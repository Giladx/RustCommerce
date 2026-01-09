use axum::extract::Path;
use axum::{http::StatusCode, Extension, Json};
use chrono::Utc;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::dto::*;
use crate::api::repository::{
    product_attribute_repository, product_feature_repository, product_repository, store_repository,
};
use crate::domain::cart::{Cart, CartItem};
use crate::domain::discount::discount::DiscountStacking;
use crate::domain::discount::{Discount, DiscountRule};
use crate::domain::money::{Currency, Money, VatRate};
use crate::domain::product::{Product, ProductAttribute, ProductFeature};
use crate::domain::store::store::Store;
use crate::utils::slugify;

/// Health check
pub async fn health() -> &'static str {
    "OK"
}

pub async fn create_product(
    Extension(pool): Extension<PgPool>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let slug = slugify(&req.title);
    let product = Product {
        id: Uuid::new_v4(),
        store_id: req.store_id,
        category_id: req.category_id,
        title: req.title,
        slug,
        description: req.description,
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
    Extension(pool): Extension<PgPool>,
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
    Extension(pool): Extension<PgPool>,
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
    Extension(pool): Extension<PgPool>,
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
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match product_repository::delete_product(&pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_store(
    Extension(pool): Extension<PgPool>,
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
