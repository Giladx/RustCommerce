use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub inventory_qty: i32,
    pub is_active: bool,
}
