use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, io::ErrorKind};
use tokio::net::TcpListener;

mod admin;
mod api;
mod domain;
mod infra;
mod mw;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .expect("Invalid DATABASE_URL");

    let api_routes = Router::new()
        .route("/health", get(api::routes::health))
        // Products
        .route("/products", post(api::routes::create_product))
        .route("/products", get(api::routes::list_products))
        .route("/products/:id", get(api::routes::get_product))
        .route("/products/:id", put(api::routes::update_product))
        .route("/products/:id", delete(api::routes::delete_product))
        // Stores
        .route("/stores", post(api::routes::create_store))
        .route("/stores", get(api::routes::list_stores_route))
        // Categories
        .route("/categories", get(api::routes::list_categories_route))
        .route("/categories", post(api::routes::create_category_route))
        .route("/categories/:id", put(api::routes::update_category_route))
        // Customers
        .route("/customers", get(api::routes::list_customers_route))
        .route("/customers", post(api::routes::create_customer_route))
        .route("/customers/:id", get(api::routes::get_customer_route))
        .route("/customers/:id", put(api::routes::update_customer_route))
        .route("/customers/:id", delete(api::routes::delete_customer_route))
        // Product Attributes
        .route("/attributes", get(api::routes::list_attributes_route))
        .route("/attributes", post(api::routes::create_attribute_route))
        .route("/attributes/:id", put(api::routes::update_attribute_route))
        .route("/attributes/:id", delete(api::routes::delete_attribute_route))
        // Product Features
        .route("/features", get(api::routes::list_features_route))
        .route("/features", post(api::routes::create_feature_route))
        .route("/features/:id", get(api::routes::get_feature_route))
        .route("/features/:id", put(api::routes::update_feature_route))
        .route("/features/:id", delete(api::routes::delete_feature_route))
        // Brands
        .route("/brands", get(api::routes::list_brands_route))
        .route("/brands", post(api::routes::create_brand_route))
        // Shopping Carts
        .route("/carts", post(api::routes::create_cart_route))
        .route("/carts", get(api::routes::list_carts_route))
        .route("/carts/:id", get(api::routes::get_cart_route))
        .route("/carts/:id", put(api::routes::update_cart_route))
        .route("/carts/:id", delete(api::routes::delete_cart_route))
        .route("/carts/:id/items", post(api::routes::add_to_cart_route))
        // Checkout
        .route("/checkout", post(api::routes::checkout_route))
        // Orders
        .route("/orders", get(api::routes::list_orders_route))
        .route("/orders/:id", get(api::routes::get_order_route))
        // Order Status Management
        .route("/order-statuses", get(api::routes::list_order_statuses_route))
        .route("/order-statuses/:id", get(api::routes::get_order_status_route))
        .route("/order-statuses/:id/transitions", get(api::routes::get_order_status_transitions_route))
        .route("/orders/:id/status", post(api::routes::change_order_status_route))
        .route("/orders/:id/status-history", get(api::routes::get_order_status_history_route))
        // Invoices
        .route("/invoices", get(api::routes::list_invoices_route))
        .route("/invoices", post(api::routes::create_invoice_route))
        .route("/invoices/:id", get(api::routes::get_invoice_route))
        .route("/invoices/:id", put(api::routes::update_invoice_route))
        .route("/invoices/:id", delete(api::routes::delete_invoice_route))
        // Credit Slips
        .route("/credit-slips", get(api::routes::list_credit_slips_route))
        .route("/credit-slips", post(api::routes::create_credit_slip_route))
        .route("/credit-slips/:id", get(api::routes::get_credit_slip_route))
        .route("/credit-slips/:id", put(api::routes::update_credit_slip_route))
        .route("/credit-slips/:id", delete(api::routes::delete_credit_slip_route))
        // Delivery Slips
        .route("/delivery-slips", get(api::routes::list_delivery_slips_route))
        .route("/delivery-slips", post(api::routes::create_delivery_slip_route))
        .route("/delivery-slips/:id", get(api::routes::get_delivery_slip_route))
        .route("/delivery-slips/:id", put(api::routes::update_delivery_slip_route))
        .route("/delivery-slips/:id", delete(api::routes::delete_delivery_slip_route))
        // Gift Cards
        .route("/gift-cards", get(api::routes::list_gift_cards_route))
        .route("/gift-cards", post(api::routes::create_gift_card_route))
        .route("/gift-cards/:id", get(api::routes::get_gift_card_route))
        .route("/gift-cards/:id", put(api::routes::update_gift_card_route))
        .route("/gift-cards/:id", delete(api::routes::delete_gift_card_route))
        // Cart Calculations
        .route("/cart/calculate", post(api::routes::calculate_cart))
        .route_layer(axum::middleware::from_fn(mw::auth::auth_middleware))
        .with_state(pool.clone());

    let app = Router::new()
        .nest("/api", api_routes)
        .merge(admin::router())
        .layer(Extension(pool));

    let addr = "127.0.0.1:3000";
    println!("🚀 RustCommerce running on http://{}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) if err.kind() == ErrorKind::PermissionDenied => {
            eprintln!(
                "⚠️  Network sandbox prevented binding to {addr}. Run this binary locally to serve HTTP."
            );
            return;
        }
        Err(err) => panic!("failed to bind {addr}: {err}"),
    };

    axum::serve(listener, app).await.unwrap();
}
