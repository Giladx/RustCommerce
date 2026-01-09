use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub store_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: Vec<ProductAttributeInput>,
    #[serde(default)]
    pub features: Vec<ProductFeatureInput>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub category_id: Option<Uuid>,
    pub slug: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProductResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStoreRequest {
    pub name: String,
    pub domain: Option<String>,
    pub currency: String,
    pub timezone: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct StoreResponse {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CalculateCartRequest {
    pub store_id: Uuid,
    pub currency: String,
    pub vat_rate: f64,
    pub items: Vec<CartItemRequest>,
    pub discounts: Vec<DiscountRequest>, // flat vector
}

#[derive(Debug, Deserialize)]
pub struct CartItemRequest {
    pub product_id: String,
    pub unit_price: f64,
    pub quantity: u32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DiscountRequest {
    Percentage {
        rate: f64,
        priority: i32,
    },
    Fixed {
        amount: f64,
        priority: i32,
    },
    Threshold {
        min: f64,
        reward: f64,
        priority: i32,
    },
}

#[derive(Debug, Serialize)]
pub struct CartTotalsResponse {
    pub net: f64,
    pub vat: f64,
    pub gross: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProductAttributeInput {
    pub name: String,
    pub value: String,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProductFeatureInput {
    pub label: String,
    pub value: String,
}
