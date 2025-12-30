use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub created_at: NaiveDateTime,
}
