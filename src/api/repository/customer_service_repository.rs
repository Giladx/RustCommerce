use crate::domain::customer_service::*;
use sqlx::PgPool;
use uuid::Uuid;

// ===== Chat Messages =====

pub async fn list_chat_messages(pool: &PgPool) -> Result<Vec<ChatMessage>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM chat_messages ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(pool)
    .await
}

pub async fn list_chat_messages_by_customer(
    pool: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<ChatMessage>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM chat_messages WHERE customer_id = $1 ORDER BY created_at ASC"
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_chat_message(
    pool: &PgPool,
    message: &ChatMessage,
) -> Result<ChatMessage, sqlx::Error> {
    sqlx::query_as::<_, ChatMessage>(
        r#"INSERT INTO chat_messages (id, customer_id, message, sender, created_at)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#
    )
    .bind(message.id)
    .bind(message.customer_id)
    .bind(&message.message)
    .bind(&message.sender)
    .bind(message.created_at)
    .fetch_one(pool)
    .await
}

// ===== Order Messages =====

pub async fn list_order_messages(pool: &PgPool) -> Result<Vec<OrderMessage>, sqlx::Error> {
    sqlx::query_as::<_, OrderMessage>(
        "SELECT * FROM order_messages ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_order_message(
    pool: &PgPool,
    message: &OrderMessage,
) -> Result<OrderMessage, sqlx::Error> {
    sqlx::query_as::<_, OrderMessage>(
        r#"INSERT INTO order_messages (id, order_id, message, is_internal, created_at)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#
    )
    .bind(message.id)
    .bind(message.order_id)
    .bind(&message.message)
    .bind(message.is_internal)
    .bind(message.created_at)
    .fetch_one(pool)
    .await
}

pub async fn reply_to_order_message(
    pool: &PgPool,
    id: Uuid,
    reply: String,
) -> Result<OrderMessage, sqlx::Error> {
    sqlx::query_as::<_, OrderMessage>(
        r#"UPDATE order_messages 
           SET reply = $1, replied_at = NOW()
           WHERE id = $2
           RETURNING *"#
    )
    .bind(reply)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_order_message(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM order_messages WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ===== Merchandise Returns =====

pub async fn list_merchandise_returns(pool: &PgPool) -> Result<Vec<MerchandiseReturn>, sqlx::Error> {
    sqlx::query_as::<_, MerchandiseReturn>(
        "SELECT * FROM merchandise_returns ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_merchandise_return(
    pool: &PgPool,
    ret: &MerchandiseReturn,
) -> Result<MerchandiseReturn, sqlx::Error> {
    sqlx::query_as::<_, MerchandiseReturn>(
        r#"INSERT INTO merchandise_returns (id, order_id, reason, items, status, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#
    )
    .bind(ret.id)
    .bind(ret.order_id)
    .bind(&ret.reason)
    .bind(&ret.items)
    .bind(&ret.status)
    .bind(ret.created_at)
    .bind(ret.updated_at)
    .fetch_one(pool)
    .await
}

pub async fn update_merchandise_return_status(
    pool: &PgPool,
    id: Uuid,
    status: String,
) -> Result<MerchandiseReturn, sqlx::Error> {
    sqlx::query_as::<_, MerchandiseReturn>(
        r#"UPDATE merchandise_returns 
           SET status = $1, updated_at = NOW()
           WHERE id = $2
           RETURNING *"#
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_merchandise_return(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM merchandise_returns WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
