use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Brand {
    pub id: Uuid,
    pub numeric_id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: bool,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductImage {
    pub id: Uuid,
    pub product_id: Uuid,
    pub url: String,
    pub alt_text: Option<String>,
    pub position: i32,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductCombination {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: Option<String>,
    pub quantity: i32,
    pub price: Option<rust_decimal::Decimal>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
