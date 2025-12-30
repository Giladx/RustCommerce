use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::product::Product;

pub async fn insert_product(
    pool: &PgPool,
    product: &Product,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO products (id, store_id, title, slug, description, is_active)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        product.id,
        product.store_id,
        product.title,
        product.slug,
        product.description,
        product.is_active
    )
    .execute(pool)
    .await?;

    Ok(())
}
