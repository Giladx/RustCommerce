use crate::domain::store::store::Store;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_store(pool: &PgPool, store: &Store) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO stores (id, name, domain, currency, timezone, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(store.id)
    .bind(&store.name)
    .bind(&store.domain)
    .bind(&store.currency)
    .bind(&store.timezone)
    .bind(store.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_stores(pool: &PgPool) -> Result<Vec<Store>, sqlx::Error> {
    sqlx::query_as::<_, Store>(
        r#"
        SELECT
            id,
            numeric_id,
            name,
            domain,
            currency,
            timezone,
            created_at::timestamptz AS created_at
        FROM stores
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_store(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM stores
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_store(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    domain: &Option<String>,
    currency: &str,
    timezone: &str,
) -> Result<Store, sqlx::Error> {
    sqlx::query_as::<_, Store>(
        r#"
        UPDATE stores
        SET
            name = $1,
            domain = $2,
            currency = $3,
            timezone = $4
        WHERE id = $5
        RETURNING
            id,
            name,
            domain,
            currency,
            timezone,
            created_at::timestamptz AS created_at
        "#,
    )
    .bind(name)
    .bind(domain)
    .bind(currency)
    .bind(timezone)
    .bind(id)
    .fetch_one(pool)
    .await
}
