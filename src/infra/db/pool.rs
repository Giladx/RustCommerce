use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .expect("Failed to create DB pool")
}
