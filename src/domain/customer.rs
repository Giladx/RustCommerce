use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub numeric_id: i32,
    pub store_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub billing_address: Option<serde_json::Value>,
    pub delivery_address: Option<serde_json::Value>,
    pub is_guest: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
