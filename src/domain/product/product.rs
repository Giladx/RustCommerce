use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Product {
    pub id: Uuid,
    pub store_id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
