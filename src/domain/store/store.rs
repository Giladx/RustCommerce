use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Store {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
}
