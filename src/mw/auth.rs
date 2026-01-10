use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use base64::prelude::*;
use sqlx::PgPool;

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get pool from extensions
    let pool = req
        .extensions()
        .get::<PgPool>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if Auth is enabled globally
    let auth_enabled_row = sqlx::query!("SELECT value FROM system_settings WHERE key = 'api_auth_enabled'")
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_enabled = auth_enabled_row
        .map(|r| r.value == "true")
        .unwrap_or(false);

    if !auth_enabled {
        return Ok(next.run(req).await);
    }

    // Get Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            // Get pool from extensions
            if let Some(pool) = req.extensions().get::<PgPool>() {
                // Check if token exists and is active
                let result = sqlx::query!(
                    "SELECT id FROM api_tokens WHERE token = $1 AND is_active = true",
                    token
                )
                .fetch_optional(pool)
                .await;

                if let Ok(Some(_)) = result {
                    // Update last_used_at
                    let _ = sqlx::query!(
                        "UPDATE api_tokens SET last_used_at = NOW() WHERE token = $1",
                        token
                    )
                    .execute(pool)
                    .await;

                    return Ok(next.run(req).await);
                }
            }
        } else if let Some(basic) = auth.strip_prefix("Basic ") {
            // Support Basic auth: username can be anything, password is the token
            if let Ok(decoded) = BASE64_STANDARD.decode(basic) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    if let Some((_username, token)) = credentials.split_once(':') {
                        if let Some(pool) = req.extensions().get::<PgPool>() {
                            let result = sqlx::query!(
                                "SELECT id FROM api_tokens WHERE token = $1 AND is_active = true",
                                token
                            )
                            .fetch_optional(pool)
                            .await;

                            if let Ok(Some(_)) = result {
                                let _ = sqlx::query!(
                                    "UPDATE api_tokens SET last_used_at = NOW() WHERE token = $1",
                                    token
                                )
                                .execute(pool)
                                .await;

                                return Ok(next.run(req).await);
                            }
                        }
                    }
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
