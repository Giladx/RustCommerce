use axum::{Router, routing::{get, post}, Extension};
use sqlx::PgPool;
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::env;

mod api;
mod domain;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let pool = PgPool::connect(&database_url).await.expect("DB connect failed");

    let app = Router::new()
        .route("/health", get(api::routes::health))
        .route("/products", post(api::routes::create_product))
        .layer(Extension(pool));

    let addr = "127.0.0.1:3000";
    println!("🚀 RustCommerce running on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
