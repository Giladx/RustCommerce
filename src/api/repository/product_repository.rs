use crate::domain::product::Product;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_product(pool: &PgPool, product: &Product) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO products (id, store_id, category_id, brand_id, title, slug, description, price, compare_price, cost_price, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(product.id)
    .bind(product.store_id)
    .bind(product.category_id)
    .bind(product.brand_id)
    .bind(&product.title)
    .bind(&product.slug)
    .bind(&product.description)
    .bind(product.price)
    .bind(product.compare_price)
    .bind(product.cost_price)
    .bind(product.is_active)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, numeric_id, store_id, category_id, brand_id, title, slug, description, price, compare_price, cost_price, is_active, created_at
        FROM products
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_product_by_id(pool: &PgPool, id: Uuid) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, numeric_id, store_id, category_id, brand_id, title, slug, description, price, compare_price, cost_price, is_active, created_at
        FROM products
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_product(
    pool: &PgPool,
    id: Uuid,
    req: &crate::api::dto::UpdateProductRequest,
) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        UPDATE products
        SET title = COALESCE($1, title),
            description = COALESCE($2, description),
            is_active = COALESCE($3, is_active),
            category_id = COALESCE($4, category_id),
            slug = COALESCE($5, slug)
        WHERE id = $6
        RETURNING id, store_id, category_id, brand_id, title, slug, description, price, compare_price, cost_price, is_active, created_at
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.is_active)
    .bind(req.category_id)
    .bind(&req.slug)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_product(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM products
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
