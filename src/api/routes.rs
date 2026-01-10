use axum::extract::{Path, State};
use axum::{http::StatusCode, Json};
use chrono::Utc;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::dto::*;
use crate::api::repository::{
    category_repository, checkout_repository, product_attribute_repository,
    product_feature_repository, product_repository, store_repository,
};
use crate::domain::brand::Brand;
use crate::domain::cart::{Cart, CartItem};
use crate::domain::category::Category;
use crate::domain::checkout::{
    CreditSlip, DeliverySlip, GiftCard, Invoice, Order, OrderStatus, OrderStatusHistory,
    OrderStatusTransition, ShoppingCart,
};
use crate::domain::discount::discount::DiscountStacking;
use crate::domain::discount::{Discount, DiscountRule};
use crate::domain::money::{Currency, Money, VatRate};
use crate::domain::product::{Product, ProductAttribute, ProductFeature};
use crate::domain::store::store::Store;
use crate::utils::slugify;

// Error handling wrapper
pub struct AppError(sqlx::Error);

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError(err)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", self.0),
        )
            .into_response()
    }
}

/// Health check
pub async fn health() -> &'static str {
    "OK"
}

pub async fn create_product(
    State(pool): State<PgPool>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let slug = slugify(&req.title);
    let product = Product {
        id: Uuid::new_v4(),
        store_id: req.store_id,
        category_id: req.category_id,
        brand_id: None,
        title: req.title,
        slug,
        description: req.description,
        price: None,
        compare_price: None,
        cost_price: None,
        is_active: true,
        created_at: Utc::now(),
    };

    match product_repository::insert_product(&pool, &product).await {
        Ok(_) => {
            for (idx, attr) in req.attributes.into_iter().enumerate() {
                let attribute = ProductAttribute {
                    id: Uuid::new_v4(),
                    product_id: product.id,
                    name: attr.name,
                    value: attr.value,
                    position: attr.position.unwrap_or(idx as i32),
                    created_at: Utc::now(),
                };

                product_attribute_repository::insert_attribute(&pool, &attribute)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }

            for feature in req.features.into_iter() {
                let feature = ProductFeature {
                    id: Uuid::new_v4(),
                    product_id: product.id,
                    label: feature.label,
                    value: feature.value,
                    created_at: Utc::now(),
                };

                product_feature_repository::insert_feature(&pool, &feature)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }

            let response = ProductResponse {
                id: product.id,
                store_id: product.store_id,
                category_id: product.category_id,
                title: product.title.clone(),
                slug: product.slug.clone(),
                description: product.description.clone(),
                is_active: product.is_active,
                created_at: product.created_at,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_products(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ProductResponse>>, StatusCode> {
    match product_repository::list_products(&pool).await {
        Ok(products) => {
            let response = products
                .into_iter()
                .map(|product| ProductResponse {
                    id: product.id,
                    store_id: product.store_id,
                    category_id: product.category_id,
                    title: product.title,
                    slug: product.slug,
                    description: product.description,
                    is_active: product.is_active,
                    created_at: product.created_at,
                })
                .collect();
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, StatusCode> {
    match product_repository::get_product_by_id(&pool, id).await {
        Ok(product) => {
            let response = ProductResponse {
                id: product.id,
                store_id: product.store_id,
                category_id: product.category_id,
                title: product.title,
                slug: product.slug,
                description: product.description,
                is_active: product.is_active,
                created_at: product.created_at,
            };
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    match product_repository::update_product(&pool, id, &req).await {
        Ok(product) => {
            let response = ProductResponse {
                id: product.id,
                store_id: product.store_id,
                category_id: product.category_id,
                title: product.title,
                slug: product.slug,
                description: product.description,
                is_active: product.is_active,
                created_at: product.created_at,
            };
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match product_repository::delete_product(&pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_store(
    State(pool): State<PgPool>,
    Json(req): Json<CreateStoreRequest>,
) -> Result<Json<StoreResponse>, StatusCode> {
    let store = Store {
        id: Uuid::new_v4(),
        name: req.name,
        domain: req.domain,
        currency: req.currency,
        timezone: req.timezone,
        created_at: Utc::now(),
    };

    match store_repository::insert_store(&pool, &store).await {
        Ok(_) => {
            let response = StoreResponse {
                id: store.id,
                name: store.name.clone(),
                domain: store.domain.clone(),
                currency: store.currency.clone(),
                timezone: store.timezone.clone(),
                created_at: store.created_at,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Convert API priority (i32) into domain-safe priority (u32)
fn parse_priority(priority: i32) -> Result<u32, StatusCode> {
    if priority < 0 {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(priority as u32)
    }
}

pub async fn calculate_cart(
    Json(req): Json<CalculateCartRequest>,
) -> Result<Json<CartTotalsResponse>, StatusCode> {
    let currency = match req.currency.as_str() {
        "ILS" => Currency::ILS,
        "EUR" => Currency::EUR,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let vat_rate = VatRate::new(Decimal::from_f64(req.vat_rate).ok_or(StatusCode::BAD_REQUEST)?);

    let mut cart = Cart::new(currency, vat_rate);

    // ---- items ----
    for item in req.items.into_iter() {
        let price = Decimal::from_f64(item.unit_price).ok_or(StatusCode::BAD_REQUEST)?;

        cart.add_item(CartItem {
            product_id: item.product_id,
            quantity: item.quantity,
            unit_price: Money::new(price, currency),
        });
    }

    // ---- discounts ----
    for discount in req.discounts.into_iter() {
        let rule = match discount {
            DiscountRequest::Percentage { rate, priority } => DiscountRule {
                discount: Discount::Percentage {
                    rate: Decimal::from_f64(rate).ok_or(StatusCode::BAD_REQUEST)?,
                },
                priority: parse_priority(priority)?,
                stacking: DiscountStacking::Exclusive,
            },

            DiscountRequest::Fixed { amount, priority } => DiscountRule {
                discount: Discount::FixedAmount {
                    amount: Money::new(
                        Decimal::from_f64(amount).ok_or(StatusCode::BAD_REQUEST)?,
                        currency,
                    ),
                },
                priority: parse_priority(priority)?,
                stacking: DiscountStacking::Exclusive,
            },

            DiscountRequest::Threshold {
                min,
                reward,
                priority,
            } => DiscountRule {
                discount: Discount::Threshold {
                    min_spend: Money::new(
                        Decimal::from_f64(min).ok_or(StatusCode::BAD_REQUEST)?,
                        currency,
                    ),
                    reward: Money::new(
                        Decimal::from_f64(reward).ok_or(StatusCode::BAD_REQUEST)?,
                        currency,
                    ),
                },
                priority: parse_priority(priority)?,
                stacking: DiscountStacking::Exclusive,
            },
        };

        cart.add_discount(rule);
    }

    let totals = cart.calculate_totals();

    Ok(Json(CartTotalsResponse {
        net: totals.net.amount().to_f64().unwrap(),
        vat: totals.vat.amount().to_f64().unwrap(),
        gross: totals.gross.amount().to_f64().unwrap(),
    }))
}

// ===== Shopping Cart Routes =====

#[derive(Deserialize)]
pub struct CreateCartRequest {
    pub store_id: Uuid,
    pub session_id: Option<String>,
}

pub async fn create_cart_route(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateCartRequest>,
) -> Result<Json<ShoppingCart>, AppError> {
    let cart = checkout_repository::create_cart(&pool, payload.store_id, payload.session_id).await?;
    Ok(Json(cart))
}

pub async fn get_cart_route(
    State(pool): State<PgPool>,
    Path(cart_id): Path<Uuid>,
) -> Result<Json<ShoppingCart>, AppError> {
    let cart = checkout_repository::get_cart(&pool, cart_id).await?;
    Ok(Json(cart))
}

#[derive(Deserialize)]
pub struct AddToCartRequest {
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}

pub async fn add_to_cart_route(
    State(pool): State<PgPool>,
    Path(cart_id): Path<Uuid>,
    Json(payload): Json<AddToCartRequest>,
) -> Result<Json<ShoppingCart>, AppError> {
    let price = rust_decimal::Decimal::from_f64(payload.unit_price).unwrap_or_default();
    checkout_repository::add_cart_item(&pool, cart_id, payload.product_id, payload.quantity, price).await?;
    let cart = checkout_repository::get_cart(&pool, cart_id).await?;
    Ok(Json(cart))
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub store_id: Uuid,
    pub customer_email: String,
    pub customer_name: String,
    pub total_amount: f64,
}

pub async fn checkout_route(
    State(pool): State<PgPool>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<Json<Order>, AppError> {
    let total = rust_decimal::Decimal::from_f64(payload.total_amount).unwrap_or_default();
    let order = checkout_repository::create_order(
        &pool,
        payload.store_id,
        payload.customer_email,
        payload.customer_name,
        total,
    )
    .await?;
    Ok(Json(order))
}

// ===== Order Routes =====

pub async fn list_orders_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Order>>, AppError> {
    let orders = checkout_repository::list_orders(&pool, None).await?;
    Ok(Json(orders))
}

pub async fn get_order_route(
    State(pool): State<PgPool>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Order>, AppError> {
    let order = checkout_repository::get_order(&pool, order_id).await?;
    Ok(Json(order))
}

// ===== Order Status Routes =====

pub async fn list_order_statuses_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<OrderStatus>>, AppError> {
    let statuses = checkout_repository::list_order_statuses(&pool).await?;
    Ok(Json(statuses))
}

pub async fn get_order_status_route(
    State(pool): State<PgPool>,
    Path(status_id): Path<Uuid>,
) -> Result<Json<OrderStatus>, AppError> {
    let status = checkout_repository::get_order_status(&pool, status_id).await?;
    Ok(Json(status))
}

pub async fn get_order_status_transitions_route(
    State(pool): State<PgPool>,
    Path(status_id): Path<Uuid>,
) -> Result<Json<Vec<OrderStatusTransition>>, AppError> {
    let transitions = checkout_repository::get_allowed_transitions(&pool, status_id).await?;
    Ok(Json(transitions))
}

#[derive(Deserialize)]
pub struct ChangeOrderStatusRequest {
    pub new_status_id: Uuid,
    pub notes: Option<String>,
}

pub async fn change_order_status_route(
    State(pool): State<PgPool>,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<ChangeOrderStatusRequest>,
) -> Result<Json<Order>, AppError> {
    let order = checkout_repository::change_order_status(
        &pool,
        order_id,
        payload.new_status_id,
        payload.notes,
    )
    .await?;
    Ok(Json(order))
}

pub async fn get_order_status_history_route(
    State(pool): State<PgPool>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Vec<OrderStatusHistory>>, AppError> {
    let history = checkout_repository::get_order_status_history(&pool, order_id).await?;
    Ok(Json(history))
}

// ===== List/Lookup Routes =====

pub async fn list_stores_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Store>>, AppError> {
    let stores = sqlx::query_as::<_, Store>("SELECT * FROM stores ORDER BY name")
        .fetch_all(&pool)
        .await?;
    Ok(Json(stores))
}

pub async fn list_categories_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Category>>, AppError> {
    let categories = category_repository::list_categories(&pool).await?;
    Ok(Json(categories))
}

pub async fn list_invoices_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Invoice>>, AppError> {
    let invoices = checkout_repository::list_invoices(&pool, None).await?;
    Ok(Json(invoices))
}

pub async fn list_credit_slips_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<CreditSlip>>, AppError> {
    let slips = checkout_repository::list_credit_slips(&pool, None).await?;
    Ok(Json(slips))
}

pub async fn list_delivery_slips_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<DeliverySlip>>, AppError> {
    let slips = checkout_repository::list_delivery_slips(&pool, None).await?;
    Ok(Json(slips))
}

pub async fn list_gift_cards_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<GiftCard>>, AppError> {
    let cards = checkout_repository::list_gift_cards(&pool, None).await?;
    Ok(Json(cards))
}

pub async fn list_carts_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ShoppingCart>>, AppError> {
    let carts = sqlx::query_as::<_, ShoppingCart>(
        "SELECT * FROM shopping_carts ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(carts))
}

// ===== Brand Routes =====

pub async fn list_brands_route(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Brand>>, AppError> {
    let brands = sqlx::query_as::<_, Brand>("SELECT * FROM brands ORDER BY name")
        .fetch_all(&pool)
        .await?;
    Ok(Json(brands))
}

#[derive(Deserialize)]
pub struct CreateBrandRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_brand_route(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBrandRequest>,
) -> Result<Json<Brand>, AppError> {
    let brand = sqlx::query_as::<_, Brand>(
        "INSERT INTO brands (name, description) VALUES ($1, $2) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .fetch_one(&pool)
    .await?;
    Ok(Json(brand))
}
