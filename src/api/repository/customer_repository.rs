use crate::domain::customer::Customer;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_customer(pool: &PgPool, customer: &Customer) -> Result<Customer, sqlx::Error> {
    sqlx::query_as!(
        Customer,
        r#"
        INSERT INTO customers (id, store_id, email, full_name, phone, password_hash, billing_address, delivery_address, is_guest, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
        customer.id,
        customer.store_id,
        customer.email,
        customer.full_name,
        customer.phone,
        customer.password_hash,
        customer.billing_address,
        customer.delivery_address,
        customer.is_guest,
        customer.created_at,
        customer.updated_at
    )
    .fetch_one(pool)
    .await
}

pub async fn update_customer(
    pool: &PgPool,
    id: Uuid,
    full_name: String,
    email: String,
    phone: Option<String>,
    is_guest: bool,
) -> Result<Customer, sqlx::Error> {
    sqlx::query_as!(
        Customer,
        r#"
        UPDATE customers
        SET full_name = $1, email = $2, phone = $3, is_guest = $4, updated_at = NOW()
        WHERE id = $5
        RETURNING *
        "#,
        full_name,
        email,
        phone,
        is_guest,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_customer(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM customers WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_customers(pool: &PgPool) -> Result<Vec<Customer>, sqlx::Error> {
    sqlx::query_as!(
        Customer,
        "SELECT * FROM customers ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn get_customer(pool: &PgPool, id: Uuid) -> Result<Customer, sqlx::Error> {
    sqlx::query_as!(
        Customer,
        "SELECT * FROM customers WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
}
