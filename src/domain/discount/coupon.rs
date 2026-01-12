use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub store_id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String, // 'percentage', 'fixed_cart', 'fixed_product'
    pub discount_value: rust_decimal::Decimal,
    pub min_purchase_amount: Option<rust_decimal::Decimal>,
    pub max_discount_amount: Option<rust_decimal::Decimal>,
    pub is_active: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub usage_limit_per_coupon: Option<i32>,
    pub usage_limit_per_customer: Option<i32>,
    pub current_usage_count: i32,
    pub free_shipping: bool,
    pub exclude_sale_items: bool,
    pub individual_use: bool,
    pub allowed_emails: Option<String>, // JSON
    pub excluded_product_ids: Option<String>, // JSON
    pub allowed_product_ids: Option<String>, // JSON
    pub excluded_category_ids: Option<String>, // JSON
    pub allowed_category_ids: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponUsage {
    pub id: Uuid,
    pub coupon_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_email: String,
    pub discount_amount: rust_decimal::Decimal,
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiscountRule {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String, // 'catalog' or 'cart'
    pub discount_type: String,
    pub discount_value: rust_decimal::Decimal,
    pub is_active: bool,
    pub priority: i32,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub uses_per_customer: Option<i32>,
    pub total_usage_limit: Option<i32>,
    pub current_usage_count: i32,
    pub stop_rules_processing: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiscountCondition {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub condition_type: String,
    pub operator: String,
    pub condition_value: String, // JSON
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiscountAction {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub action_type: String,
    pub action_value: Option<rust_decimal::Decimal>,
    pub max_discount_amount: Option<rust_decimal::Decimal>,
    pub apply_to: Option<String>,
    pub target_products: Option<String>, // JSON
    pub target_categories: Option<String>, // JSON
    pub buy_quantity: Option<i32>,
    pub get_quantity: Option<i32>,
    pub get_discount_percentage: Option<rust_decimal::Decimal>,
    pub created_at: DateTime<Utc>,
}
