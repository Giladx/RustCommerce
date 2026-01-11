use crate::domain::checkout::*;
use sqlx::PgPool;
use uuid::Uuid;

// Shopping Carts
pub async fn create_cart(
    pool: &PgPool,
    store_id: Uuid,
    session_id: Option<String>,
) -> Result<ShoppingCart, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCart,
        r#"
        INSERT INTO shopping_carts (store_id, session_id, status, currency)
        VALUES ($1, $2, 'active', 'USD')
        RETURNING *
        "#,
        store_id,
        session_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_cart(pool: &PgPool, cart_id: Uuid) -> Result<ShoppingCart, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCart,
        r#"SELECT * FROM shopping_carts WHERE id = $1"#,
        cart_id
    )
    .fetch_one(pool)
    .await
}

pub async fn update_cart_customer(
    pool: &PgPool,
    cart_id: Uuid,
    customer_email: Option<String>,
    customer_name: Option<String>,
) -> Result<ShoppingCart, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCart,
        r#"
        UPDATE shopping_carts
        SET customer_email = COALESCE($2, customer_email),
            customer_name = COALESCE($3, customer_name),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        cart_id,
        customer_email,
        customer_name
    )
    .fetch_one(pool)
    .await
}

pub async fn list_carts(pool: &PgPool, store_id: Uuid) -> Result<Vec<ShoppingCart>, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCart,
        r#"SELECT * FROM shopping_carts WHERE store_id = $1 ORDER BY created_at DESC"#,
        store_id
    )
    .fetch_all(pool)
    .await
}

pub async fn find_cart_by_customer_name(
    pool: &PgPool,
    store_id: Uuid,
    customer_name: &str,
) -> Result<Option<ShoppingCart>, sqlx::Error> {
    let pattern = format!("%{}%", customer_name);
    sqlx::query_as!(
        ShoppingCart,
        r#"
        SELECT * FROM shopping_carts
        WHERE store_id = $1 AND customer_name ILIKE $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        store_id,
        pattern
    )
    .fetch_optional(pool)
    .await
}

pub async fn add_cart_item(
    pool: &PgPool,
    cart_id: Uuid,
    product_id: Uuid,
    quantity: i32,
    unit_price: rust_decimal::Decimal,
) -> Result<CartItem, sqlx::Error> {
    let line_total = unit_price * rust_decimal::Decimal::from(quantity);
    
    sqlx::query_as!(
        CartItem,
        r#"
        INSERT INTO cart_items (cart_id, product_id, quantity, unit_price, line_total)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        cart_id,
        product_id,
        quantity,
        unit_price,
        line_total
    )
    .fetch_one(pool)
    .await
}

pub async fn get_cart_items(pool: &PgPool, cart_id: Uuid) -> Result<Vec<CartItem>, sqlx::Error> {
    sqlx::query_as!(
        CartItem,
        r#"SELECT * FROM cart_items WHERE cart_id = $1"#,
        cart_id
    )
    .fetch_all(pool)
    .await
}

// Orders
pub async fn create_order(
    pool: &PgPool,
    store_id: Uuid,
    customer_email: String,
    customer_name: String,
    total_amount: rust_decimal::Decimal,
) -> Result<Order, sqlx::Error> {
    let order_number = format!("ORD-{}", Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase());
    
    // Get default status (Pending)
    let default_status = sqlx::query!(
        "SELECT id, name FROM order_statuses WHERE is_default = true LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;
    
    let (status_id, status_name) = if let Some(status) = default_status {
        (Some(status.id), status.name)
    } else {
        // Fallback to finding 'pending' status
        let pending = sqlx::query!(
            "SELECT id, name FROM order_statuses WHERE code = 'pending' LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;
        
        if let Some(p) = pending {
            (Some(p.id), p.name)
        } else {
            (None, "pending".to_string())
        }
    };
    
    sqlx::query_as!(
        Order,
        r#"
        INSERT INTO orders 
        (order_number, store_id, customer_email, customer_name, status, status_id, payment_status, currency, subtotal, total_amount)
        VALUES ($1, $2, $3, $4, $5, $6, 'unpaid', 'USD', $7, $7)
        RETURNING *
        "#,
        order_number,
        store_id,
        customer_email,
        customer_name,
        status_name,
        status_id,
        total_amount
    )
    .fetch_one(pool)
    .await
}

pub async fn create_order_from_cart(
    pool: &PgPool,
    cart_id: Uuid,
    store_id: Uuid,
    customer_email: String,
    customer_name: String,
    total_amount: rust_decimal::Decimal,
) -> Result<Order, sqlx::Error> {
    let order_number = format!(
        "ORD-{}",
        Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap()
            .to_uppercase()
    );

    let default_status = sqlx::query!(
        "SELECT id, name FROM order_statuses WHERE is_default = true LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let (status_id, status_name) = if let Some(status) = default_status {
        (Some(status.id), status.name)
    } else {
        let pending = sqlx::query!(
            "SELECT id, name FROM order_statuses WHERE code = 'pending' LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;

        if let Some(p) = pending {
            (Some(p.id), p.name)
        } else {
            (None, "pending".to_string())
        }
    };

    sqlx::query_as!(
        Order,
        r#"
        INSERT INTO orders
        (order_number, store_id, cart_id, customer_email, customer_name, status, status_id, payment_status, currency, subtotal, total_amount)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'unpaid', 'USD', $8, $8)
        RETURNING *
        "#,
        order_number,
        store_id,
        cart_id,
        customer_email,
        customer_name,
        status_name,
        status_id,
        total_amount
    )
    .fetch_one(pool)
    .await
}

pub async fn mark_cart_converted(pool: &PgPool, cart_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE shopping_carts
        SET status = 'converted',
            converted_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        cart_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_order(pool: &PgPool, order_id: Uuid) -> Result<Order, sqlx::Error> {
    sqlx::query_as!(
        Order,
        r#"SELECT * FROM orders WHERE id = $1"#,
        order_id
    )
    .fetch_one(pool)
    .await
}

pub async fn list_orders(pool: &PgPool, store_id: Option<Uuid>) -> Result<Vec<Order>, sqlx::Error> {
    match store_id {
        Some(sid) => {
            sqlx::query_as!(
                Order,
                r#"SELECT * FROM orders WHERE store_id = $1 ORDER BY created_at DESC"#,
                sid
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                Order,
                r#"SELECT * FROM orders ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn update_order_status(
    pool: &PgPool,
    order_id: Uuid,
    status: String,
) -> Result<Order, sqlx::Error> {
    sqlx::query_as!(
        Order,
        r#"
        UPDATE orders 
        SET status = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        order_id,
        status
    )
    .fetch_one(pool)
    .await
}

pub async fn update_order(
    pool: &PgPool,
    order_id: Uuid,
    customer_email: Option<String>,
    customer_name: Option<String>,
    customer_phone: Option<String>,
    status: Option<String>,
    payment_status: Option<String>,
    notes: Option<String>,
    billing_address: Option<serde_json::Value>,
    shipping_address: Option<serde_json::Value>,
) -> Result<Order, sqlx::Error> {
    let current_order = get_order(pool, order_id).await?;
    
    sqlx::query_as!(
        Order,
        r#"
        UPDATE orders 
        SET 
            customer_email = COALESCE($2, customer_email),
            customer_name = COALESCE($3, customer_name),
            customer_phone = COALESCE($4, customer_phone),
            status = COALESCE($5, status),
            payment_status = COALESCE($6, payment_status),
            notes = COALESCE($7, notes),
            billing_address = COALESCE($8, billing_address),
            shipping_address = COALESCE($9, shipping_address),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        order_id,
        customer_email.or(Some(current_order.customer_email)),
        customer_name.or(Some(current_order.customer_name)),
        customer_phone.or_else(|| current_order.customer_phone.clone()),
        status.or(Some(current_order.status)),
        payment_status.or(Some(current_order.payment_status)),
        notes,
        billing_address,
        shipping_address
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_order(pool: &PgPool, order_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM orders WHERE id = $1",
        order_id
    )
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected() > 0)
}

// Invoices
pub async fn create_invoice(
    pool: &PgPool,
    order_id: Uuid,
    store_id: Uuid,
) -> Result<Invoice, sqlx::Error> {
    let invoice_number = format!("INV-{}", Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase());
    
    let order = get_order(pool, order_id).await?;
    
    sqlx::query_as!(
        Invoice,
        r#"
        INSERT INTO invoices 
        (invoice_number, order_id, store_id, customer_email, customer_name, status, currency, subtotal, tax_amount, total_amount)
        VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7, $8, $9)
        RETURNING *
        "#,
        invoice_number,
        order_id,
        store_id,
        order.customer_email,
        order.customer_name,
        order.currency,
        order.subtotal,
        order.tax_amount,
        order.total_amount
    )
    .fetch_one(pool)
    .await
}

pub async fn list_invoices(pool: &PgPool, store_id: Option<Uuid>) -> Result<Vec<Invoice>, sqlx::Error> {
    match store_id {
        Some(sid) => {
            sqlx::query_as!(
                Invoice,
                r#"SELECT * FROM invoices WHERE store_id = $1 ORDER BY created_at DESC"#,
                sid
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                Invoice,
                r#"SELECT * FROM invoices ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

// Credit Slips
pub async fn list_credit_slips(pool: &PgPool, store_id: Option<Uuid>) -> Result<Vec<CreditSlip>, sqlx::Error> {
    match store_id {
        Some(sid) => {
            sqlx::query_as!(
                CreditSlip,
                r#"SELECT * FROM credit_slips WHERE store_id = $1 ORDER BY created_at DESC"#,
                sid
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                CreditSlip,
                r#"SELECT * FROM credit_slips ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

// Delivery Slips
pub async fn list_delivery_slips(pool: &PgPool, store_id: Option<Uuid>) -> Result<Vec<DeliverySlip>, sqlx::Error> {
    match store_id {
        Some(sid) => {
            sqlx::query_as!(
                DeliverySlip,
                r#"SELECT * FROM delivery_slips WHERE store_id = $1 ORDER BY created_at DESC"#,
                sid
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                DeliverySlip,
                r#"SELECT * FROM delivery_slips ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

// Gift Cards
pub async fn create_gift_card(
    pool: &PgPool,
    store_id: Uuid,
    amount: rust_decimal::Decimal,
    recipient_email: Option<String>,
) -> Result<GiftCard, sqlx::Error> {
    let code = format!("GIFT-{}", Uuid::new_v4().to_string().split('-').take(2).collect::<Vec<_>>().join("").to_uppercase());
    
    sqlx::query_as!(
        GiftCard,
        r#"
        INSERT INTO gift_cards 
        (code, store_id, initial_amount, current_balance, currency, status, recipient_email)
        VALUES ($1, $2, $3, $3, 'USD', 'active', $4)
        RETURNING *
        "#,
        code,
        store_id,
        amount,
        recipient_email
    )
    .fetch_one(pool)
    .await
}

pub async fn list_gift_cards(pool: &PgPool, store_id: Option<Uuid>) -> Result<Vec<GiftCard>, sqlx::Error> {
    match store_id {
        Some(sid) => {
            sqlx::query_as!(
                GiftCard,
                r#"SELECT * FROM gift_cards WHERE store_id = $1 ORDER BY created_at DESC"#,
                sid
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                GiftCard,
                r#"SELECT * FROM gift_cards ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

// Order Statuses
pub async fn list_order_statuses(pool: &PgPool) -> Result<Vec<OrderStatus>, sqlx::Error> {
    sqlx::query_as!(
        OrderStatus,
        r#"SELECT * FROM order_statuses ORDER BY position ASC"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_order_status(pool: &PgPool, status_id: Uuid) -> Result<OrderStatus, sqlx::Error> {
    sqlx::query_as!(
        OrderStatus,
        r#"SELECT * FROM order_statuses WHERE id = $1"#,
        status_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_order_status_by_code(pool: &PgPool, code: &str) -> Result<OrderStatus, sqlx::Error> {
    sqlx::query_as!(
        OrderStatus,
        r#"SELECT * FROM order_statuses WHERE code = $1"#,
        code
    )
    .fetch_one(pool)
    .await
}

pub async fn get_allowed_transitions(pool: &PgPool, from_status_id: Uuid) -> Result<Vec<OrderStatusTransition>, sqlx::Error> {
    sqlx::query_as!(
        OrderStatusTransition,
        r#"SELECT * FROM order_status_transitions WHERE from_status_id = $1"#,
        from_status_id
    )
    .fetch_all(pool)
    .await
}

pub async fn change_order_status(
    pool: &PgPool,
    order_id: Uuid,
    new_status_id: Uuid,
    notes: Option<String>,
) -> Result<Order, sqlx::Error> {
    // Get current order
    let current_order = get_order(pool, order_id).await?;
    
    // Get the new status details
    let new_status = get_order_status(pool, new_status_id).await?;
    
    // TODO: Validate transition is allowed (check order_status_transitions table)
    
    // Record in history
    sqlx::query!(
        r#"
        INSERT INTO order_status_history (order_id, from_status, to_status, notes)
        VALUES ($1, $2, $3, $4)
        "#,
        order_id,
        current_order.status,
        new_status.name,
        notes
    )
    .execute(pool)
    .await?;
    
    // Update order status
    sqlx::query_as!(
        Order,
        r#"
        UPDATE orders 
        SET status = $2, status_id = $3, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        order_id,
        new_status.name,
        new_status_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_order_status_history(pool: &PgPool, order_id: Uuid) -> Result<Vec<OrderStatusHistory>, sqlx::Error> {
    sqlx::query_as!(
        OrderStatusHistory,
        r#"SELECT * FROM order_status_history WHERE order_id = $1 ORDER BY created_at DESC"#,
        order_id
    )
    .fetch_all(pool)
    .await
}
