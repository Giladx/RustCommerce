use axum::{http::StatusCode, Json};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::api::dto::*;
use crate::domain::cart::{Cart, CartItem};
use crate::domain::discount::{Discount, DiscountRule};
use crate::domain::money::{Currency, Money, VatRate};
use crate::domain::discount::discount::DiscountStacking;

/// Health check
pub async fn health() -> &'static str {
    "OK"
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
    Json(req): Json<CartRequest>,
) -> Result<Json<CartResponse>, StatusCode> {
    let currency = match req.currency.as_str() {
        "ILS" => Currency::ILS,
        "EUR" => Currency::EUR,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let vat_rate = VatRate::new(
        Decimal::from_f64(req.vat_rate)
            .ok_or(StatusCode::BAD_REQUEST)?,
    );

    let mut cart = Cart::new(currency, vat_rate);

    // ---- items ----
    for item in req.items.into_iter() {
        let price = Decimal::from_f64(item.unit_price)
            .ok_or(StatusCode::BAD_REQUEST)?;

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
                    rate: Decimal::from_f64(rate)
                        .ok_or(StatusCode::BAD_REQUEST)?,
                },
                priority: parse_priority(priority)?,
                stacking: DiscountStacking::Exclusive,
            },

            DiscountRequest::Fixed { amount, priority } => DiscountRule {
                discount: Discount::FixedAmount {
                    amount: Money::new(
                        Decimal::from_f64(amount)
                            .ok_or(StatusCode::BAD_REQUEST)?,
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
                        Decimal::from_f64(min)
                            .ok_or(StatusCode::BAD_REQUEST)?,
                        currency,
                    ),
                    reward: Money::new(
                        Decimal::from_f64(reward)
                            .ok_or(StatusCode::BAD_REQUEST)?,
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

    Ok(Json(CartResponse {
        net: totals.net.amount().to_f64().unwrap(),
        vat: totals.vat.amount().to_f64().unwrap(),
        gross: totals.gross.amount().to_f64().unwrap(),
    }))
}
