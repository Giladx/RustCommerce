use sqlx::PgPool;
use crate::domain::store::Store;

pub async fn get_store_by_domain(
    pool: &PgPool,
    domain: &str,
) -> Result<Store, sqlx::Error> {
    sqlx::query_as!(
        Store,
        r#"
        SELECT
            id,
            name,
            domain,
            currency,
            timezone,
            created_at
        FROM stores
        WHERE domain = $1
        "#,
        domain
    )
    .fetch_one(pool)
    .await
}
