use crate::domain::product::ProductAttribute;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_attribute(
    pool: &PgPool,
    attribute: &ProductAttribute,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO product_attributes (id, product_id, name, value, position, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(attribute.id)
    .bind(attribute.product_id)
    .bind(&attribute.name)
    .bind(&attribute.value)
    .bind(attribute.position)
    .bind(attribute.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_attributes(pool: &PgPool) -> Result<Vec<ProductAttribute>, sqlx::Error> {
    sqlx::query_as::<_, ProductAttribute>(
        r#"
        SELECT id, product_id, name, value, position, created_at
        FROM product_attributes
        ORDER BY position ASC, created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_attribute(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM product_attributes WHERE id = $1
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
        DELETE FROM product_attributes WHERE product_id = $1
        "#,
    )
    .bind(product_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_attribute(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    value: &str,
    position: i32,
) -> Result<ProductAttribute, sqlx::Error> {
    sqlx::query_as::<_, ProductAttribute>(
        r#"
        UPDATE product_attributes
        SET
            name = $1,
            value = $2,
            position = $3
        WHERE id = $4
        RETURNING id, product_id, name, value, position, created_at
        "#,
    )
    .bind(name)
    .bind(value)
    .bind(position)
    .bind(id)
    .fetch_one(pool)
    .await
}
