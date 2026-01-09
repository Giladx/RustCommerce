use crate::domain::store::store::Store;
use sqlx::PgPool;

pub async fn get_store_by_domain(pool: &PgPool, domain: &str) -> Result<Store, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id,
            name,
            domain,
            currency,
            timezone,
            created_at::timestamptz AS created_at
        FROM stores
        WHERE domain = $1
        "#,
    )
    .bind(domain)
    .fetch_one(pool)
    .await
}
