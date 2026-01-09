use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProductAttribute {
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub value: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}
