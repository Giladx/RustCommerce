use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub message: String,
    pub sender: String, // "customer" or "support"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderMessage {
    pub id: Uuid,
    pub order_id: Uuid,
    pub message: String,
    pub is_internal: bool,
    pub reply: Option<String>,
    pub created_at: DateTime<Utc>,
    pub replied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchandiseReturn {
    pub id: Uuid,
    pub order_id: Uuid,
    pub reason: String,
    pub items: serde_json::Value, // JSON array of returned items
    pub status: String, // "pending", "approved", "rejected", "completed"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
