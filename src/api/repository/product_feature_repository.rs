use crate::domain::product::ProductFeature;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_feature(pool: &PgPool, feature: &ProductFeature) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO product_features (id, product_id, label, value, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(feature.id)
    .bind(feature.product_id)
    .bind(&feature.label)
    .bind(&feature.value)
    .bind(feature.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_features(pool: &PgPool) -> Result<Vec<ProductFeature>, sqlx::Error> {
    sqlx::query_as::<_, ProductFeature>(
        r#"
        SELECT id, product_id, label, value, created_at
        FROM product_features
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_feature(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM product_features WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_by_product(pool: &PgPool, product_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM product_features WHERE product_id = $1
        "#,
    )
    .bind(product_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_feature(
    pool: &PgPool,
    id: Uuid,
    label: &str,
    value: &str,
) -> Result<ProductFeature, sqlx::Error> {
    sqlx::query_as::<_, ProductFeature>(
        r#"
        UPDATE product_features
        SET
            label = $1,
            value = $2
        WHERE id = $3
        RETURNING id, product_id, label, value, created_at
        "#,
    )
    .bind(label)
    .bind(value)
    .bind(id)
    .fetch_one(pool)
    .await
}
