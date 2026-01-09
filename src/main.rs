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
        .route("/products", post(api::routes::create_product))
        .route("/products", get(api::routes::list_products))
        .route("/products/:id", get(api::routes::get_product))
        .route("/products/:id", put(api::routes::update_product))
        .route("/products/:id", delete(api::routes::delete_product))
        .route("/stores", post(api::routes::create_store));

    let app = Router::new()
        .merge(api_routes)
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
