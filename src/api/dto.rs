use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
