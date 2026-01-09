use crate::domain::category::Category;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_category(pool: &PgPool, category: &Category) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO categories (id, store_id, name, slug, description, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(category.id)
    .bind(category.store_id)
    .bind(&category.name)
    .bind(&category.slug)
    .bind(&category.description)
    .bind(category.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_categories(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, store_id, name, slug, description, created_at
        FROM categories
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_category(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE products
        SET category_id = NULL
        WHERE category_id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    let result = sqlx::query(
        r#"
        DELETE FROM categories
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    store_id: Uuid,
    name: &str,
    slug: &str,
    description: &Option<String>,
) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories
        SET
            store_id = $1,
            name = $2,
            slug = $3,
            description = $4
        WHERE id = $5
        RETURNING id, store_id, name, slug, description, created_at
        "#,
    )
    .bind(store_id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(id)
    .fetch_one(pool)
    .await
}
