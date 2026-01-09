use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProductFeature {
    pub id: Uuid,
    pub product_id: Uuid,
    pub label: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}
