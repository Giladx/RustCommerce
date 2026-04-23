use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OTP verification attempt stored in database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OtpVerification {
    pub id: Uuid,
    pub numeric_id: i32,
    pub phone: String,
    pub code: String,
    pub app_id: i32,
    pub max_tries: i32,
    pub tries_count: i32,
    pub valid_until: DateTime<Utc>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// 019 SMS API request for sending OTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOtpRequest {
    pub send_otp: SendOtpData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOtpData {
    pub user: OtpUser,
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<i32>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tries: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_time: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpUser {
    pub username: String,
}

/// 019 SMS API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpResponse {
    pub sms: OtpResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpResponseData {
    pub status: i32,
    pub code: String,
    pub message: String,
}

/// Verify OTP request from customer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone: String,
    pub code: String,
    pub app_id: Option<i32>,
}

impl OtpVerification {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.valid_until
    }

    pub fn has_tries_remaining(&self) -> bool {
        self.tries_count < self.max_tries
    }

    pub fn can_verify(&self) -> bool {
        !self.is_expired() && self.has_tries_remaining() && !self.verified
    }
}
