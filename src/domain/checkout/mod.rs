use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShoppingCart {
    pub id: Uuid,
    pub numeric_id: i32,
    pub store_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub session_id: Option<String>,
    pub status: String,
    pub currency: String,
    pub subtotal: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub discount_amount: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub converted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub numeric_id: i32,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: rust_decimal::Decimal,
    pub line_total: rust_decimal::Decimal,
    pub product_snapshot: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub numeric_id: i32,
    pub order_number: String,
    pub store_id: Uuid,
    pub cart_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub customer_email: String,
    pub customer_name: String,
    pub customer_phone: Option<String>,
    pub status: String,
    pub status_id: Option<Uuid>,
    pub payment_status: String,
    pub currency: String,
    pub subtotal: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub shipping_amount: rust_decimal::Decimal,
    pub discount_amount: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub notes: Option<String>,
    pub billing_address: Option<serde_json::Value>,
    pub shipping_address: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub numeric_id: i32,
    pub order_id: Uuid,
    pub product_id: Option<Uuid>,
    pub product_title: String,
    pub product_sku: Option<String>,
    pub quantity: i32,
    pub unit_price: rust_decimal::Decimal,
    pub line_total: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub product_snapshot: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderStatus {
    pub id: Uuid,
    pub numeric_id: i32,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_default: Option<bool>,
    pub is_final: Option<bool>,
    pub position: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderStatusTransition {
    pub id: Uuid,
    pub numeric_id: i32,
    pub from_status_id: Uuid,
    pub to_status_id: Uuid,
    pub requires_confirmation: Option<bool>,
    pub confirmation_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderStatusHistory {
    pub id: Uuid,
    pub numeric_id: i32,
    pub order_id: Uuid,
    pub from_status: Option<String>,
    pub to_status: String,
    pub changed_by: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub numeric_id: i32,
    pub invoice_number: String,
    pub order_id: Uuid,
    pub store_id: Uuid,
    pub customer_email: String,
    pub customer_name: String,
    pub status: String,
    pub currency: String,
    pub subtotal: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub paid_amount: rust_decimal::Decimal,
    pub due_date: Option<chrono::NaiveDate>,
    pub billing_address: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeliverySlip {
    pub id: Uuid,
    pub numeric_id: i32,
    pub slip_number: String,
    pub order_id: Uuid,
    pub store_id: Uuid,
    pub status: String,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub shipping_address: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub estimated_delivery: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CreditSlip {
    pub id: Uuid,
    pub numeric_id: i32,
    pub slip_number: String,
    pub order_id: Uuid,
    pub store_id: Uuid,
    pub reason: Option<String>,
    pub status: String,
    pub currency: String,
    pub refund_amount: rust_decimal::Decimal,
    pub refund_method: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GiftCard {
    pub id: Uuid,
    pub numeric_id: i32,
    pub code: String,
    pub store_id: Uuid,
    pub initial_amount: rust_decimal::Decimal,
    pub current_balance: rust_decimal::Decimal,
    pub currency: String,
    pub status: String,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub sender_name: Option<String>,
    pub message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub redeemed_at: Option<DateTime<Utc>>,
}
