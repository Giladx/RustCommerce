use crate::domain::otp::{OtpResponse, OtpVerification, SendOtpData, SendOtpRequest, OtpUser, VerifyOtpRequest};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

const OTP_API_URL: &str = "https://019sms.co.il/api";

#[derive(Debug, Clone)]
pub struct OtpConfig {
    pub username: String,
    pub source: String,
    pub default_app_id: i32,
    pub default_max_tries: i32,
    pub default_valid_minutes: i32,
    pub default_text: String,
}

impl Default for OtpConfig {
    fn default() -> Self {
        Self {
            username: std::env::var("OTP_USERNAME").unwrap_or_else(|_| "username".to_string()),
            source: std::env::var("OTP_SOURCE").unwrap_or_else(|_| "***".to_string()),
            default_app_id: 1,
            default_max_tries: 3,
            default_valid_minutes: 5,
            default_text: "קוד האימות שלך הוא [code]".to_string(),
        }
    }
}

/// Send OTP via 019 SMS API
pub async fn send_otp(
    pool: &PgPool,
    phone: &str,
    app_id: Option<i32>,
    config: &OtpConfig,
) -> Result<OtpVerification, String> {
    // Normalize phone number (remove leading 0 if present)
    let normalized_phone = phone.trim_start_matches('0');
    let full_phone = if normalized_phone.starts_with("5") {
        format!("0{}", normalized_phone)
    } else {
        normalized_phone.to_string()
    };

    // Check for recent OTP (prevent spam)
    if let Ok(recent) = get_recent_otp(pool, &full_phone, app_id.unwrap_or(config.default_app_id)).await {
        if !recent.is_expired() && !recent.verified {
            return Err("OTP already sent. Please wait before requesting a new code.".to_string());
        }
    }

    // Build request
    let request = SendOtpRequest {
        send_otp: SendOtpData {
            user: OtpUser {
                username: config.username.clone(),
            },
            phone: full_phone.clone(),
            app_id: Some(app_id.unwrap_or(config.default_app_id)),
            source: config.source.clone(),
            max_tries: Some(config.default_max_tries),
            valid_time: Some(config.default_valid_minutes),
            text: Some(config.default_text.clone()),
        },
    };

    // Send to 019 SMS API
    let client = reqwest::Client::new();
    let response = client
        .post(OTP_API_URL)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send OTP: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("OTP API returned error: {}", response.status()));
    }

    let otp_response: OtpResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse OTP response: {}", e))?;

    // Check if OTP was sent successfully
    if otp_response.sms.status != 0 {
        return Err(format!("OTP sending failed with status: {}", otp_response.sms.status));
    }

    // Store OTP verification in database
    let verification = OtpVerification {
        id: Uuid::new_v4(),
        numeric_id: 0, // Auto-generated
        phone: full_phone,
        code: otp_response.sms.code,
        app_id: app_id.unwrap_or(config.default_app_id),
        max_tries: config.default_max_tries,
        tries_count: 0,
        valid_until: Utc::now() + Duration::minutes(config.default_valid_minutes as i64),
        verified: false,
        created_at: Utc::now(),
        verified_at: None,
    };

    save_otp_verification(pool, &verification).await?;

    Ok(verification)
}

/// Verify OTP code
pub async fn verify_otp(
    pool: &PgPool,
    request: &VerifyOtpRequest,
) -> Result<OtpVerification, String> {
    // Normalize phone
    let normalized_phone = request.phone.trim_start_matches('0');
    let full_phone = if normalized_phone.starts_with("5") {
        format!("0{}", normalized_phone)
    } else {
        normalized_phone.to_string()
    };

    // Get latest unverified OTP for this phone
    let mut verification = get_latest_otp(pool, &full_phone, request.app_id.unwrap_or(1))
        .await
        .map_err(|_| "No OTP verification found for this phone number")?;

    // Check if OTP can be verified
    if verification.verified {
        return Err("This OTP code has already been verified".to_string());
    }

    if verification.is_expired() {
        return Err("OTP code has expired. Please request a new code.".to_string());
    }

    if !verification.has_tries_remaining() {
        return Err("Maximum verification attempts exceeded. Please request a new code.".to_string());
    }

    // Increment tries count
    verification.tries_count += 1;
    update_tries_count(pool, verification.id, verification.tries_count).await?;

    // Verify code
    if verification.code != request.code {
        if verification.tries_count >= verification.max_tries {
            return Err("Invalid code. Maximum attempts exceeded.".to_string());
        }
        return Err(format!(
            "Invalid code. {} attempts remaining.",
            verification.max_tries - verification.tries_count
        ));
    }

    // Mark as verified
    verification.verified = true;
    verification.verified_at = Some(Utc::now());
    mark_as_verified(pool, verification.id).await?;

    Ok(verification)
}

/// Save OTP verification to database
async fn save_otp_verification(pool: &PgPool, verification: &OtpVerification) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO otp_verifications (id, phone, code, app_id, max_tries, tries_count, valid_until, verified, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(verification.id)
    .bind(&verification.phone)
    .bind(&verification.code)
    .bind(verification.app_id)
    .bind(verification.max_tries)
    .bind(verification.tries_count)
    .bind(verification.valid_until)
    .bind(verification.verified)
    .bind(verification.created_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to save OTP verification: {}", e))?;

    Ok(())
}

/// Get latest OTP for phone and app
async fn get_latest_otp(pool: &PgPool, phone: &str, app_id: i32) -> Result<OtpVerification, sqlx::Error> {
    sqlx::query_as::<_, OtpVerification>(
        r#"
        SELECT id, numeric_id, phone, code, app_id, max_tries, tries_count, valid_until, verified, created_at, verified_at
        FROM otp_verifications
        WHERE phone = $1 AND app_id = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(phone)
    .bind(app_id)
    .fetch_one(pool)
    .await
}

/// Get recent OTP (within last 2 minutes) to prevent spam
async fn get_recent_otp(pool: &PgPool, phone: &str, app_id: i32) -> Result<OtpVerification, sqlx::Error> {
    let two_minutes_ago = Utc::now() - Duration::minutes(2);
    
    sqlx::query_as::<_, OtpVerification>(
        r#"
        SELECT id, numeric_id, phone, code, app_id, max_tries, tries_count, valid_until, verified, created_at, verified_at
        FROM otp_verifications
        WHERE phone = $1 AND app_id = $2 AND created_at > $3
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(phone)
    .bind(app_id)
    .bind(two_minutes_ago)
    .fetch_one(pool)
    .await
}

/// Update tries count
async fn update_tries_count(pool: &PgPool, id: Uuid, tries: i32) -> Result<(), String> {
    sqlx::query("UPDATE otp_verifications SET tries_count = $1 WHERE id = $2")
        .bind(tries)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update tries count: {}", e))?;

    Ok(())
}

/// Mark OTP as verified
async fn mark_as_verified(pool: &PgPool, id: Uuid) -> Result<(), String> {
    sqlx::query("UPDATE otp_verifications SET verified = TRUE, verified_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to mark OTP as verified: {}", e))?;

    Ok(())
}

/// Clean up expired OTP codes (run periodically)
pub async fn cleanup_expired_otps(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM otp_verifications WHERE valid_until < NOW() AND created_at < NOW() - INTERVAL '7 days'")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
