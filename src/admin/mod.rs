use std::collections::HashMap;

mod auth;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::{
    api::{
        dto::UpdateProductRequest,
        repository::{
            category_repository, checkout_repository, customer_repository, customer_service_repository,
            product_attribute_repository, product_feature_repository, product_repository,
            store_repository,
        },
    },
    domain::{
        brand::Brand,
        category::Category,
        checkout::{
            CreditSlip, DeliverySlip, GiftCard, Invoice, Order, OrderStatus, ShoppingCart,
        },
        customer::Customer,
        customer_service::{ChatMessage, MerchandiseReturn, OrderMessage},
        product::{Product, ProductAttribute, ProductFeature},
        store::store::Store,
    },
    utils::slugify,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AdminNav {
    Dashboard,
    Catalog,
    Orders,
    Customers,
    CustomerService,
    ApiTokens,
}

#[derive(Debug, sqlx::FromRow)]
struct AdminApiToken {
    id: Uuid,
    name: String,
    token: String,
    description: Option<String>,
    is_active: bool,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CustomerForm {
    store_id: Uuid,
    full_name: String,
    email: String,
    phone: Option<String>,
    #[serde(default)]
    is_guest: bool,
}

#[derive(Debug, Deserialize)]
struct TokenForm {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingsForm {
    api_auth_enabled: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StoreForm {
    name: String,
    domain: Option<String>,
    currency: String,
    timezone: String,
}

#[derive(Debug, Deserialize)]
struct CategoryForm {
    store_id: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProductForm {
    store_id: Uuid,
    category_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    #[serde(default)]
    is_active: Option<bool>,
    #[serde(default)]
    attributes: Option<String>,
    #[serde(default)]
    features: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateProductForm {
    category_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    #[serde(default)]
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AttributeForm {
    product_id: Uuid,
    name: String,
    value: String,
    position: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct FeatureForm {
    product_id: Uuid,
    label: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct BrandForm {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OrderCreateForm {
    store_id: Option<Uuid>,
    mode: Option<String>,
    customer_email: Option<String>,
    customer_name: Option<String>,
    existing_cart_id: Option<String>,
    existing_customer_name: Option<String>,
    item_product_id: Option<Vec<String>>,
    item_quantity: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
struct OrderStatusForm {
    new_status_id: Uuid,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InvoiceForm {
    order_id: Uuid,
    store_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct InvoiceStatusForm {
    status: String,
}

#[derive(Debug, Deserialize)]
struct CreditSlipForm {
    order_id: Uuid,
    slip_number: String,
    amount: f64,
}

#[derive(Debug, Deserialize)]
struct DeliverySlipForm {
    order_id: Uuid,
    slip_number: String,
    tracking_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GiftCardForm {
    id: Option<Uuid>,
    code: String,
    initial_amount: f64,
    current_balance: f64,
    recipient_name: Option<String>,
    recipient_email: Option<String>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct ChatMessageForm {
    customer_id: Uuid,
    message: String,
    sender: String, // "customer" or "support"
}

#[derive(Debug, Deserialize)]
struct OrderMessageForm {
    order_id: Uuid,
    message: String,
    is_internal: bool,
}

#[derive(Debug, Deserialize)]
struct OrderMessageReplyForm {
    reply: String,
}

#[derive(Debug, Deserialize)]
struct MerchandiseReturnForm {
    order_id: Uuid,
    reason: String,
    items: String, // JSON array of items
    status: String,
}

pub fn router() -> Router {
    let protected_routes = Router::new()
        .route("/", get(dashboard_page))
        .route("/admin", get(dashboard_page))
        .route("/admin/docs", get(api_docs))
        // API Tokens & Settings
        .route("/admin/api-tokens", get(api_tokens_page).post(create_api_token))
        .route("/admin/api-tokens/:id/toggle", post(toggle_api_token))
        .route("/admin/settings/auth", post(update_api_auth_settings))
        // Customers
        .route("/admin/customers", get(customers_page).post(create_customer))
        .route("/admin/customers/:id/update", post(update_customer))
        .route("/admin/customers/:id/delete", post(delete_customer))
        // Stores
        .route("/admin/stores", get(stores_page).post(create_store))
        .route("/admin/stores/:id/update", post(update_store))
        .route("/admin/stores/:id/delete", post(delete_store))
        // Catalog
        .route("/admin/catalog", get(catalog_page))
        // Categories
        .route("/admin/categories", get(categories_page).post(create_category))
        .route("/admin/categories/:id/update", post(update_category))
        .route("/admin/categories/:id/delete", post(delete_category))
        // Products
        .route("/admin/products", get(products_page).post(create_product))
        .route("/admin/products/:id/update", post(update_product))
        .route("/admin/products/:id/delete", post(delete_product))
        // Brands
        .route("/admin/brands", get(brands_page).post(create_brand))
        .route("/admin/brands/:id/update", post(update_brand))
        .route("/admin/brands/:id/delete", post(delete_brand))
        // Orders
        .route("/admin/orders", get(orders_page).post(create_order))
        .route("/admin/orders/:id/status", post(update_order_status))
        .route("/admin/invoices", get(invoices_page).post(create_invoice))
        .route("/admin/invoices/:id/update", post(update_invoice))
        .route("/admin/invoices/:id/delete", post(delete_invoice))
        .route("/admin/credit-slips", get(credit_slips_page).post(create_credit_slip))
        .route("/admin/credit-slips/:id/update", post(update_credit_slip))
        .route("/admin/credit-slips/:id/delete", post(delete_credit_slip))
        .route("/admin/delivery-slips", get(delivery_slips_page).post(create_delivery_slip))
        .route("/admin/delivery-slips/:id/update", post(update_delivery_slip))
        .route("/admin/delivery-slips/:id/delete", post(delete_delivery_slip))
        .route("/admin/shopping-carts", get(shopping_carts_page))
        .route("/admin/shopping-carts/:id/delete", post(delete_shopping_cart))
        .route("/admin/gift-cards", get(gift_cards_page))
        .route("/admin/gift-cards/create", post(create_gift_card))
        .route("/admin/gift-cards/update", post(update_gift_card))
        .route("/admin/gift-cards/delete", post(delete_gift_card))
        // Customer Service
        .route("/admin/customer-service/chat", get(customer_service_chat_page).post(send_chat_message))
        .route("/admin/customer-service/order-messages", get(order_messages_page).post(create_order_message))
        .route("/admin/customer-service/order-messages/:id/reply", post(reply_order_message))
        .route("/admin/customer-service/order-messages/:id/delete", post(delete_order_message_handler))
        .route("/admin/customer-service/returns", get(merchandise_returns_page).post(create_merchandise_return))
        .route("/admin/customer-service/returns/:id/update", post(update_merchandise_return))
        .route("/admin/customer-service/returns/:id/delete", post(delete_merchandise_return_handler))
        // Attributes & Features
        .route(
            "/admin/product-attributes",
            get(attributes_page).post(create_attribute),
        )
        .route(
            "/admin/product-attributes/:id/update",
            post(update_attribute),
        )
        .route(
            "/admin/product-attributes/:id/delete",
            post(delete_attribute),
        )
        .route(
            "/admin/product-features",
            get(features_page).post(create_feature),
        )
        .route("/admin/product-features/:id/update", post(update_feature))
        .route("/admin/product-features/:id/delete", post(delete_feature))
        .layer(axum::middleware::from_fn(auth::admin_auth_middleware));

    let public_routes = Router::new()
        .route("/admin/login", get(auth::login_page).post(auth::login_handler))
        .route("/admin/logout", get(auth::logout_handler));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}

async fn dashboard_page(
    Extension(pool): Extension<PgPool>,
) -> Html<String> {
    // 1. Fetch Key Metrics
    let total_revenue = sqlx::query!(
        r#"SELECT SUM(total_amount) as total 
           FROM orders 
           WHERE status NOT IN ('cancelled', 'refunded', 'pending')"#
    )
    .fetch_one(&pool)
    .await
    .map(|r| r.total.unwrap_or_default())
    .unwrap_or_default();

    let total_orders = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM orders WHERE status NOT IN ('cancelled', 'refunded', 'pending')"#
    )
    .fetch_one(&pool)
    .await
    .map(|r| r.count.unwrap_or(0))
    .unwrap_or(0);

    let avg_order_value = if total_orders > 0 {
        total_revenue / Decimal::from(total_orders)
    } else {
        Decimal::ZERO
    };

    let customers_count = sqlx::query!("SELECT COUNT(*) as count FROM customers")
        .fetch_one(&pool)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // 2. Fetch Recent Orders for the list
    let recent_orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders ORDER BY created_at DESC LIMIT 5"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let content = format!(
        r#"
        <!-- Financial Widgets -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div class="bg-white dark:bg-dark-panel rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-dark-border transition-all hover:shadow-md">
                <div class="flex justify-between items-start">
                    <div>
                        <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Total Revenue</p>
                        <h3 class="text-2xl font-bold text-gray-900 dark:text-white mt-2">${:.2}</h3>
                    </div>
                    <div class="p-2 bg-green-50 dark:bg-green-900/20 rounded-lg">
                        <span class="material-icons text-green-600 dark:text-green-400">attach_money</span>
                    </div>
                </div>
            </div>
            
            <div class="bg-white dark:bg-dark-panel rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-dark-border transition-all hover:shadow-md">
                <div class="flex justify-between items-start">
                    <div>
                        <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Total Orders</p>
                        <h3 class="text-2xl font-bold text-gray-900 dark:text-white mt-2">{}</h3>
                    </div>
                    <div class="p-2 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                        <span class="material-icons text-blue-600 dark:text-blue-400">shopping_bag</span>
                    </div>
                </div>
            </div>

            <div class="bg-white dark:bg-dark-panel rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-dark-border transition-all hover:shadow-md">
                <div class="flex justify-between items-start">
                    <div>
                        <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Avg. Order Value</p>
                        <h3 class="text-2xl font-bold text-gray-900 dark:text-white mt-2">${:.2}</h3>
                    </div>
                    <div class="p-2 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                        <span class="material-icons text-purple-600 dark:text-purple-400">analytics</span>
                    </div>
                </div>
            </div>

            <div class="bg-white dark:bg-dark-panel rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-dark-border transition-all hover:shadow-md">
                <div class="flex justify-between items-start">
                    <div>
                        <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Total Customers</p>
                        <h3 class="text-2xl font-bold text-gray-900 dark:text-white mt-2">{}</h3>
                    </div>
                    <div class="p-2 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                        <span class="material-icons text-orange-600 dark:text-orange-400">group</span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Recent Activity Section -->
        <div class="bg-white dark:bg-dark-panel rounded-2xl shadow-sm border border-gray-100 dark:border-dark-border overflow-hidden">
            <div class="px-6 py-4 border-b border-gray-100 dark:border-dark-border flex justify-between items-center">
                <h3 class="font-semibold text-gray-900 dark:text-white">Recent Orders</h3>
                <a href="/admin/orders" class="text-sm text-blue-600 dark:text-blue-400 hover:underline">View all</a>
            </div>
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50/50 dark:bg-slate-800/50">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Order #</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Customer</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Total</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Date</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-100 dark:divide-gray-800">
                        {}
                    </tbody>
                </table>
            </div>
        </div>
        "#,
        total_revenue,
        total_orders,
        avg_order_value,
        customers_count,
        render_recent_orders_rows(&recent_orders)
    );
    
    Html(render_page(
        AdminNav::Dashboard,
        "Dashboard",
        "Overview of your store performance",
        &content,
    ))
}

fn render_recent_orders_rows(orders: &[Order]) -> String {
    if orders.is_empty() {
        return r#"<tr><td colspan="5" class="px-6 py-8 text-center text-gray-500 dark:text-gray-400 italic">No recent orders found</td></tr>"#.to_string();
    }

    orders.iter().map(|o| {
        let status_class = order_status_badge_class("", &o.status);
        format!(
            r#"<tr class="hover:bg-gray-50/50 dark:hover:bg-slate-800/50 transition-colors">
                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-600 dark:text-gray-300">{}</td>
                <td class="px-6 py-4 whitespace-nowrap"><span class="px-2.5 py-0.5 rounded-full text-xs font-medium {}">{}</span></td>
                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">${}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">{}</td>
            </tr>"#,
            html_escape(&o.order_number),
            html_escape(&o.customer_email),
            status_class,
            html_escape(&o.status),
            o.total_amount,
            o.created_at.format("%b %d, %Y")
        )
    }).collect::<Vec<_>>().join("")
}

// ===== Individual Page Handlers =====

async fn stores_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Stores",
        "Manage your store configurations",
        &render_stores_content(&stores),
    ))
}

async fn categories_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let categories = category_repository::list_categories(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Categories",
        "Organize your product catalog",
        &render_categories_content(&stores, &categories),
    ))
}

async fn products_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let categories = category_repository::list_categories(&pool).await.unwrap_or_default();
    let products = product_repository::list_products(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Products",
        "Manage your product inventory",
        &render_products_content(&stores, &categories, &products),
    ))
}

async fn brands_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let brands = sqlx::query_as::<_, Brand>("SELECT * FROM brands ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Brands",
        "Manage product brands",
        &render_brands_content(&brands),
    ))
}

async fn catalog_page() -> Redirect {
    Redirect::to("/admin/categories")
}

async fn attributes_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let products = product_repository::list_products(&pool).await.unwrap_or_default();
    let attributes = product_attribute_repository::list_attributes(&pool)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Attributes",
        "Create and edit product attributes",
        &render_attribute_section(&attributes, &products),
    ))
}

async fn features_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let products = product_repository::list_products(&pool).await.unwrap_or_default();
    let features = product_feature_repository::list_features(&pool)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Features",
        "Attach descriptive feature values per product",
        &render_feature_section(&features, &products),
    ))
}

async fn orders_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let statuses = checkout_repository::list_order_statuses(&pool).await.unwrap_or_default();
    let products = product_repository::list_products(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Orders",
        "View and manage customer orders",
        &render_orders_content(&orders, &stores, &statuses, &products),
    ))
}

async fn create_order(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<OrderCreateForm>,
) -> impl IntoResponse {
    match create_order_from_form(&pool, form).await {
        Ok(_) => Redirect::to("/admin/orders").into_response(),
        Err(message) => {
            eprintln!("Failed to create order: {message}");
            (
                StatusCode::BAD_REQUEST,
                Html(render_error(&message)),
            )
                .into_response()
        }
    }
}

async fn update_order_status(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<OrderStatusForm>,
) -> impl IntoResponse {
    match checkout_repository::change_order_status(&pool, id, form.new_status_id, form.notes.clone())
        .await
    {
        Ok(_) => Redirect::to("/admin/orders").into_response(),
        Err(err) => {
            eprintln!("Failed to update order status: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update order status")),
            )
                .into_response()
        }
    }
}

async fn create_order_from_form(
    pool: &PgPool,
    form: OrderCreateForm,
) -> Result<Order, String> {
    let mode = form.mode.as_deref().unwrap_or("new");
    let store_id = form
        .store_id
        .ok_or_else(|| "Store is required.".to_string())?;

    if mode == "existing" {
        let cart_id = form
            .existing_cart_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| Uuid::parse_str(value).map_err(|_| "Invalid cart ID.".to_string()))
            .transpose()?;

        let cart = if let Some(cart_id) = cart_id {
            let cart = checkout_repository::get_cart(pool, cart_id)
                .await
                .map_err(|_| "Cart not found.".to_string())?;
            if cart.store_id != store_id {
                return Err("Cart does not belong to the selected store.".to_string());
            }
            cart
        } else {
            let customer_name = form
                .existing_customer_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Provide a cart ID or customer name.".to_string())?;
            checkout_repository::find_cart_by_customer_name(pool, store_id, customer_name)
                .await
                .map_err(|_| "Unable to search carts.".to_string())?
                .ok_or_else(|| "No cart found for that customer name.".to_string())?
        };

        let cart_items = checkout_repository::get_cart_items(pool, cart.id)
            .await
            .map_err(|_| "Unable to load cart items.".to_string())?;
        if cart_items.is_empty() {
            return Err("Selected cart has no items.".to_string());
        }

        let customer_email = optional_text(cart.customer_email.clone())
            .ok_or_else(|| "Cart is missing customer email.".to_string())?;
        let customer_name = optional_text(cart.customer_name.clone())
            .ok_or_else(|| "Cart is missing customer name.".to_string())?;

        let total = cart_items
            .iter()
            .fold(Decimal::from_i32(0).unwrap_or_default(), |acc, item| acc + item.line_total);

        let order = checkout_repository::create_order_from_cart(
            pool,
            cart.id,
            cart.store_id,
            customer_email,
            customer_name,
            total,
        )
        .await
        .map_err(|_| "Unable to create order from cart.".to_string())?;

        checkout_repository::mark_cart_converted(pool, cart.id)
            .await
            .map_err(|_| "Unable to finalize cart.".to_string())?;

        return Ok(order);
    }

    let customer_email = optional_text(form.customer_email)
        .ok_or_else(|| "Customer email is required.".to_string())?;
    let customer_name = optional_text(form.customer_name)
        .ok_or_else(|| "Customer name is required.".to_string())?;

    let items = parse_cart_items(form.item_product_id, form.item_quantity)?;
    if items.is_empty() {
        return Err("Add at least one product to the cart.".to_string());
    }

    let cart = checkout_repository::create_cart(pool, store_id, None)
        .await
        .map_err(|_| "Unable to create cart.".to_string())?;
    checkout_repository::update_cart_customer(
        pool,
        cart.id,
        Some(customer_email.clone()),
        Some(customer_name.clone()),
    )
    .await
    .map_err(|_| "Unable to set cart customer info.".to_string())?;

    let mut total = Decimal::from_i32(0).unwrap_or_default();
    for item in items {
        let product = product_repository::get_product_by_id(pool, item.product_id)
            .await
            .map_err(|_| "Unable to load product.".to_string())?;
        let unit_price = product.price.unwrap_or_else(|| Decimal::from_i32(0).unwrap_or_default());
        let quantity = Decimal::from_i32(item.quantity).unwrap_or_default();
        total += unit_price * quantity;
        checkout_repository::add_cart_item(
            pool,
            cart.id,
            item.product_id,
            item.quantity,
            unit_price,
        )
        .await
        .map_err(|_| "Unable to add item to cart.".to_string())?;
    }

    let order = checkout_repository::create_order_from_cart(
        pool,
        cart.id,
        store_id,
        customer_email,
        customer_name,
        total,
    )
    .await
    .map_err(|_| "Unable to create order.".to_string())?;

    checkout_repository::mark_cart_converted(pool, cart.id)
        .await
        .map_err(|_| "Unable to finalize cart.".to_string())?;

    Ok(order)
}

#[derive(Debug)]
struct CartItemInput {
    product_id: Uuid,
    quantity: i32,
}

fn parse_cart_items(
    product_ids: Option<Vec<String>>,
    quantities: Option<Vec<i32>>,
) -> Result<Vec<CartItemInput>, String> {
    let product_ids = product_ids.unwrap_or_default();
    let quantities = quantities.unwrap_or_default();
    let mut items = Vec::new();

    for (index, raw_id) in product_ids.into_iter().enumerate() {
        let trimmed = raw_id.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let product_id =
            Uuid::parse_str(&trimmed).map_err(|_| "Invalid product selection.".to_string())?;
        let quantity = quantities.get(index).copied().unwrap_or(1).max(1);
        items.push(CartItemInput { product_id, quantity });
    }

    Ok(items)
}

async fn invoices_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let invoices = checkout_repository::list_invoices(&pool, None).await.unwrap_or_default();
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Invoices",
        "Billing documents and payments",
        &render_invoices_content(&invoices, &orders, &stores),
    ))
}

async fn credit_slips_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let slips = checkout_repository::list_credit_slips(&pool, None).await.unwrap_or_default();
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Credit Slips",
        "Refunds and credits",
        &render_credit_slips_content(&slips, &orders),
    ))
}

async fn delivery_slips_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let slips = checkout_repository::list_delivery_slips(&pool, None).await.unwrap_or_default();
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Delivery Slips",
        "Shipping and delivery tracking",
        &render_delivery_slips_content(&slips, &orders),
    ))
}

async fn shopping_carts_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let carts = sqlx::query_as::<_, ShoppingCart>(
        "SELECT * FROM shopping_carts ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Shopping Carts",
        "Active and abandoned carts",
        &render_shopping_carts_content(&carts),
    ))
}

async fn gift_cards_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let cards = checkout_repository::list_gift_cards(&pool, None).await.unwrap_or_default();
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Gift Cards",
        "Manage gift card codes and balances",
        &render_gift_cards_content(&cards, &stores),
    ))
}

// ===== Order-Related CRUD Handlers =====

async fn create_invoice(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<InvoiceForm>,
) -> impl IntoResponse {
    match checkout_repository::create_invoice(&pool, form.order_id, form.store_id).await {
        Ok(_) => Redirect::to("/admin/invoices").into_response(),
        Err(err) => {
            eprintln!("Failed to create invoice: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create invoice")),
            )
                .into_response()
        }
    }
}

async fn update_invoice(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<InvoiceStatusForm>,
) -> impl IntoResponse {
    let query = "UPDATE invoices SET status = $1 WHERE id = $2";
    match sqlx::query(query)
        .bind(&form.status)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/invoices").into_response(),
        Err(err) => {
            eprintln!("Failed to update invoice: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update invoice")),
            )
                .into_response()
        }
    }
}

async fn delete_invoice(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM invoices WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/invoices").into_response(),
        Err(err) => {
            eprintln!("Failed to delete invoice: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete invoice")),
            )
                .into_response()
        }
    }
}

async fn create_credit_slip(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<CreditSlipForm>,
) -> impl IntoResponse {
    let amount = rust_decimal::Decimal::from_f64(form.amount).unwrap_or_default();
    let query = "INSERT INTO credit_slips (order_id, slip_number, amount) VALUES ($1, $2, $3)";
    match sqlx::query(query)
        .bind(form.order_id)
        .bind(&form.slip_number)
        .bind(amount)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/credit-slips").into_response(),
        Err(err) => {
            eprintln!("Failed to create credit slip: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create credit slip")),
            )
                .into_response()
        }
    }
}

async fn update_credit_slip(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<CreditSlipForm>,
) -> impl IntoResponse {
    let amount = rust_decimal::Decimal::from_f64(form.amount).unwrap_or_default();
    let query = "UPDATE credit_slips SET slip_number = $1, amount = $2 WHERE id = $3";
    match sqlx::query(query)
        .bind(&form.slip_number)
        .bind(amount)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/credit-slips").into_response(),
        Err(err) => {
            eprintln!("Failed to update credit slip: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update credit slip")),
            )
                .into_response()
        }
    }
}

async fn delete_credit_slip(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM credit_slips WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/credit-slips").into_response(),
        Err(err) => {
            eprintln!("Failed to delete credit slip: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete credit slip")),
            )
                .into_response()
        }
    }
}

async fn create_delivery_slip(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<DeliverySlipForm>,
) -> impl IntoResponse {
    let query = "INSERT INTO delivery_slips (order_id, slip_number, tracking_number) VALUES ($1, $2, $3)";
    match sqlx::query(query)
        .bind(form.order_id)
        .bind(&form.slip_number)
        .bind(form.tracking_number)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/delivery-slips").into_response(),
        Err(err) => {
            eprintln!("Failed to create delivery slip: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create delivery slip")),
            )
                .into_response()
        }
    }
}

async fn update_delivery_slip(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<DeliverySlipForm>,
) -> impl IntoResponse {
    let query = "UPDATE delivery_slips SET slip_number = $1, tracking_number = $2 WHERE id = $3";
    match sqlx::query(query)
        .bind(&form.slip_number)
        .bind(form.tracking_number)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/delivery-slips").into_response(),
        Err(err) => {
            eprintln!("Failed to update delivery slip: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update delivery slip")),
            )
                .into_response()
        }
    }
}

async fn delete_delivery_slip(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM delivery_slips WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/delivery-slips").into_response(),
        Err(err) => {
            eprintln!("Failed to delete delivery slip: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete delivery slip")),
            )
                .into_response()
        }
    }
}

async fn delete_shopping_cart(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM shopping_carts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/shopping-carts").into_response(),
        Err(err) => {
            eprintln!("Failed to delete cart: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete cart")),
            )
                .into_response()
        }
    }
}

async fn create_gift_card(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<GiftCardForm>,
) -> impl IntoResponse {
    let initial_amount = rust_decimal::Decimal::from_f64(form.initial_amount).unwrap_or_default();
    let current_balance = rust_decimal::Decimal::from_f64(form.current_balance).unwrap_or_default();
    
    // Get the default store (or handle multi-store properly)
    let store_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM stores LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(Uuid::nil());
    
    let query = r#"
        INSERT INTO gift_cards (store_id, code, initial_amount, current_balance, currency, status, recipient_name, recipient_email)
        VALUES ($1, $2, $3, $4, 'USD', $5, $6, $7)
    "#;
    
    match sqlx::query(query)
        .bind(store_id)
        .bind(&form.code)
        .bind(initial_amount)
        .bind(current_balance)
        .bind(&form.status)
        .bind(optional_text(form.recipient_name))
        .bind(optional_text(form.recipient_email))
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/gift-cards").into_response(),
        Err(err) => {
            eprintln!("Failed to create gift card: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create gift card")),
            )
                .into_response()
        }
    }
}

async fn update_gift_card(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<GiftCardForm>,
) -> impl IntoResponse {
    let id = form.id.unwrap_or(Uuid::nil());
    let initial_amount = rust_decimal::Decimal::from_f64(form.initial_amount).unwrap_or_default();
    let current_balance = rust_decimal::Decimal::from_f64(form.current_balance).unwrap_or_default();
    
    let query = r#"
        UPDATE gift_cards 
        SET code = $1, initial_amount = $2, current_balance = $3, status = $4, 
            recipient_name = $5, recipient_email = $6
        WHERE id = $7
    "#;
    
    match sqlx::query(query)
        .bind(&form.code)
        .bind(initial_amount)
        .bind(current_balance)
        .bind(&form.status)
        .bind(optional_text(form.recipient_name))
        .bind(optional_text(form.recipient_email))
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/gift-cards").into_response(),
        Err(err) => {
            eprintln!("Failed to update gift card: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update gift card")),
            )
                .into_response()
        }
    }
}

async fn delete_gift_card(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = form.get("id").and_then(|s| Uuid::parse_str(s).ok()).unwrap_or(Uuid::nil());
    
    match sqlx::query("DELETE FROM gift_cards WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => Redirect::to("/admin/gift-cards").into_response(),
        Err(err) => {
            eprintln!("Failed to delete gift card: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete gift card")),
            )
                .into_response()
        }
    }
}

// ===== Customer Service Handlers =====

async fn customer_service_chat_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let messages = customer_service_repository::list_chat_messages(&pool)
        .await
        .unwrap_or_default();
    let customers = customer_repository::list_customers(&pool)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::CustomerService,
        "Customer Service Chat",
        "Live chat conversations with customers",
        &render_chat_content(&messages, &customers),
    ))
}

async fn send_chat_message(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<ChatMessageForm>,
) -> impl IntoResponse {
    let message = ChatMessage {
        id: Uuid::new_v4(),
        customer_id: form.customer_id,
        message: form.message,
        sender: form.sender,
        created_at: Utc::now(),
    };

    match customer_service_repository::insert_chat_message(&pool, &message).await {
        Ok(_) => Redirect::to("/admin/customer-service/chat").into_response(),
        Err(err) => {
            eprintln!("Failed to send message: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to send message")),
            )
                .into_response()
        }
    }
}

async fn order_messages_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let messages = customer_service_repository::list_order_messages(&pool)
        .await
        .unwrap_or_default();
    let orders = checkout_repository::list_orders(&pool, None)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::CustomerService,
        "Order Messages",
        "Customer questions and internal notes about orders",
        &render_order_messages_content(&messages, &orders),
    ))
}

async fn create_order_message(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<OrderMessageForm>,
) -> impl IntoResponse {
    let message = OrderMessage {
        id: Uuid::new_v4(),
        order_id: form.order_id,
        message: form.message,
        is_internal: form.is_internal,
        reply: None,
        created_at: Utc::now(),
        replied_at: None,
    };

    match customer_service_repository::insert_order_message(&pool, &message).await {
        Ok(_) => Redirect::to("/admin/customer-service/order-messages").into_response(),
        Err(err) => {
            eprintln!("Failed to create order message: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create message")),
            )
                .into_response()
        }
    }
}

async fn reply_order_message(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<OrderMessageReplyForm>,
) -> impl IntoResponse {
    match customer_service_repository::reply_to_order_message(&pool, id, form.reply).await {
        Ok(_) => Redirect::to("/admin/customer-service/order-messages").into_response(),
        Err(err) => {
            eprintln!("Failed to reply to message: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to reply")),
            )
                .into_response()
        }
    }
}

async fn delete_order_message_handler(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match customer_service_repository::delete_order_message(&pool, id).await {
        Ok(_) => Redirect::to("/admin/customer-service/order-messages").into_response(),
        Err(err) => {
            eprintln!("Failed to delete message: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete message")),
            )
                .into_response()
        }
    }
}

async fn merchandise_returns_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let returns = customer_service_repository::list_merchandise_returns(&pool)
        .await
        .unwrap_or_default();
    let orders = checkout_repository::list_orders(&pool, None)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::CustomerService,
        "Merchandise Returns",
        "Manage product returns and stock restocking",
        &render_merchandise_returns_content(&returns, &orders),
    ))
}

async fn create_merchandise_return(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<MerchandiseReturnForm>,
) -> impl IntoResponse {
    let items: serde_json::Value = serde_json::from_str(&form.items).unwrap_or(json!([]));
    let ret = MerchandiseReturn {
        id: Uuid::new_v4(),
        order_id: form.order_id,
        reason: form.reason,
        items,
        status: form.status,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match customer_service_repository::insert_merchandise_return(&pool, &ret).await {
        Ok(_) => Redirect::to("/admin/customer-service/returns").into_response(),
        Err(err) => {
            eprintln!("Failed to create return: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create return")),
            )
                .into_response()
        }
    }
}

async fn update_merchandise_return(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<MerchandiseReturnForm>,
) -> impl IntoResponse {
    match customer_service_repository::update_merchandise_return_status(&pool, id, form.status)
        .await
    {
        Ok(_) => Redirect::to("/admin/customer-service/returns").into_response(),
        Err(err) => {
            eprintln!("Failed to update return: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update return")),
            )
                .into_response()
        }
    }
}

async fn delete_merchandise_return_handler(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match customer_service_repository::delete_merchandise_return(&pool, id).await {
        Ok(_) => Redirect::to("/admin/customer-service/returns").into_response(),
        Err(err) => {
            eprintln!("Failed to delete return: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete return")),
            )
                .into_response()
        }
    }
}

// ===== Customers Handlers =====

async fn customers_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let customers = customer_repository::list_customers(&pool)
        .await
        .unwrap_or_default();
    let stores = store_repository::list_stores(&pool)
        .await
        .unwrap_or_default();

    Html(render_page(
        AdminNav::Customers,
        "Customers",
        "View and manage your customer base",
        &render_customers_content(&customers, &stores),
    ))
}

async fn create_customer(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<CustomerForm>,
) -> impl IntoResponse {
    let customer = Customer {
        id: Uuid::new_v4(),
        store_id: form.store_id,
        email: form.email.trim().to_owned(),
        full_name: form.full_name.trim().to_owned(),
        phone: optional_text(form.phone),
        password_hash: None,
        billing_address: None,
        delivery_address: None,
        is_guest: form.is_guest,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match customer_repository::insert_customer(&pool, &customer).await {
        Ok(_) => Redirect::to("/admin/customers").into_response(),
        Err(err) => {
            eprintln!("Failed to create customer: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create customer")),
            )
                .into_response()
        }
    }
}

async fn update_customer(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<CustomerForm>,
) -> impl IntoResponse {
    match customer_repository::update_customer(
        &pool,
        id,
        form.full_name.trim().to_owned(),
        form.email.trim().to_owned(),
        optional_text(form.phone),
        form.is_guest,
    )
    .await
    {
        Ok(_) => Redirect::to("/admin/customers").into_response(),
        Err(err) => {
            eprintln!("Failed to update customer: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update customer")),
            )
                .into_response()
        }
    }
}

async fn delete_customer(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match customer_repository::delete_customer(&pool, id).await {
        Ok(_) => Redirect::to("/admin/customers").into_response(),
        Err(err) => {
            eprintln!("Failed to delete customer: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete customer")),
            )
                .into_response()
        }
    }
}

fn render_customers_content(customers: &[Customer], stores: &[Store]) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    
    let header = format!(r#"
        <div class="flex items-center justify-between mb-6">
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white">Customers</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Manage registered users and guest accounts</p>
            </div>
            <button type="button" data-modal-target="customer" data-modal-mode="create" data-modal-action="/admin/customers" data-modal-title="Add Customer" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-2">
                <span class="material-icons text-lg">person_add</span>
                New Customer
            </button>
        </div>

        <!-- Filter Bar -->
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm border border-gray-200 dark:border-dark-border p-4 mb-4">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="md:col-span-1">
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Search</label>
                    <input type="text" id="customerSearch" placeholder="Name, email, phone..." class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Type</label>
                    <select id="typeFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Types</option>
                        <option value="registered">Registered</option>
                        <option value="guest">Guest</option>
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Store</label>
                    <select id="storeFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Stores</option>
                        {store_options}
                    </select>
                </div>
                <div class="flex items-end">
                    <button onclick="clearCustomerFilters()" class="w-full px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-slate-800 rounded-lg hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors">
                        Clear Filters
                    </button>
                </div>
            </div>
            <div class="mt-3 text-xs text-gray-500 dark:text-gray-400">
                Showing <span id="customerCount">{total}</span> customers
            </div>
        </div>

        <!-- Bulk Actions Bar -->
        <div id="bulkActionsBar" class="hidden bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 mb-4">
            <div class="flex items-center justify-between">
                <span class="text-sm font-medium text-blue-900 dark:text-blue-100">
                    <span id="selectedCount">0</span> customers selected
                </span>
                <div class="flex gap-2">
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Export Selected
                    </button>
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Send Email
                    </button>
                </div>
            </div>
        </div>
    "#,
        total = customers.len(),
        store_options = stores.iter()
            .map(|s| format!(r#"<option value="{}">{}</option>"#, s.id, html_escape(&s.name)))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let rows = if customers.is_empty() {
        r#"<tr><td colspan="10" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No customers found.</td></tr>"#.to_string()
    } else {
        customers
            .iter()
            .enumerate()
            .map(|(idx, customer)| {
                let type_badge = if customer.is_guest {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400">Guest</span>"#
                } else {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400">Registered</span>"#
                };
                
                let store_name = store_lookup.get(&customer.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                
                // Mock data for orders and spending
                let order_count = ((idx * 7) % 20) + 1;
                let total_spent = ((idx * 13) % 100) * 10 + 50;
                
                // Status badge based on activity
                let status_badge = if order_count > 10 {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Active</span>"#
                } else if order_count > 3 {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400">Regular</span>"#
                } else {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400">New</span>"#
                };
                
                let date_str = customer.created_at.format("%b %d, %Y").to_string();
                
                let payload = json!({
                    "store_id": customer.store_id,
                    "full_name": customer.full_name,
                    "email": customer.email,
                    "phone": customer.phone.clone().unwrap_or_default(),
                    "is_guest": customer.is_guest,
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="customer-row border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50 transition-colors group"
                        data-name="{name_lower}"
                        data-email="{email_lower}"
                        data-phone="{phone_lower}"
                        data-type="{type_lower}"
                        data-store="{store_id}">
                        <td class="px-4 py-3">
                            <input type="checkbox" class="customer-checkbox rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                        </td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white font-semibold text-sm">
                                    {initials}
                                </div>
                                <div>
                                    <div class="font-semibold text-gray-900 dark:text-white text-sm">{name}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">{email}</div>
                                </div>
                            </div>
                        </td>
                        <td class="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">{phone}</td>
                        <td class="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">{store}</td>
                        <td class="px-4 py-3">{type_badge}</td>
                        <td class="px-4 py-3">{status_badge}</td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-medium text-gray-900 dark:text-white">{order_count}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">orders</div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-semibold text-gray-900 dark:text-white">${total_spent}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">lifetime</div>
                        </td>
                        <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{date}</td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button type="button" data-modal-target="customer" data-modal-mode="edit" data-modal-action="/admin/customers/{id}/update" data-modal-title="Edit Customer" data-modal-payload="{payload}"
                                    class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors" title="Edit">
                                    <span class="material-icons text-lg">edit</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors" title="View Orders">
                                    <span class="material-icons text-lg">shopping_bag</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-purple-600 dark:hover:text-purple-400 hover:bg-purple-50 dark:hover:bg-purple-900/20 rounded transition-colors" title="Send Email">
                                    <span class="material-icons text-lg">email</span>
                                </button>
                                <form method="post" action="/admin/customers/{id}/delete" style="display:inline;" onsubmit="return confirm('Delete this customer? This cannot be undone.');">
                                    <button type="submit" class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors" title="Delete">
                                        <span class="material-icons text-lg">delete</span>
                                    </button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    name = html_escape(&customer.full_name),
                    name_lower = customer.full_name.to_lowercase(),
                    email = html_escape(&customer.email),
                    email_lower = customer.email.to_lowercase(),
                    phone = customer.phone.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    phone_lower = customer.phone.as_deref().unwrap_or("").to_lowercase(),
                    store = store_name,
                    store_id = customer.store_id,
                    type_badge = type_badge,
                    type_lower = if customer.is_guest { "guest" } else { "registered" },
                    status_badge = status_badge,
                    order_count = order_count,
                    total_spent = total_spent,
                    date = date_str,
                    initials = customer.full_name.chars()
                        .filter(|c| c.is_uppercase())
                        .take(2)
                        .collect::<String>(),
                    id = customer.id,
                    payload = payload_attr
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"{header}
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50 dark:bg-slate-800/50 border-b border-gray-200 dark:border-dark-border">
                        <tr>
                            <th class="px-4 py-3 text-left w-12">
                                <input type="checkbox" id="selectAllCustomers" class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                            </th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Customer</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Phone</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Store</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Type</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Orders</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Total Spent</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Joined</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            
            <!-- Pagination -->
            <div class="bg-gray-50 dark:bg-slate-800/30 px-4 py-3 border-t border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        Showing <span class="font-medium">1</span> to <span class="font-medium">{count}</span> of <span class="font-medium">{count}</span> customers
                    </div>
                    <div class="flex gap-2">
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Previous</button>
                        <button class="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 border border-blue-600 rounded-lg">1</button>
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Next</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
        // Customer filtering
        function filterCustomers() {{
            const search = document.getElementById('customerSearch').value.toLowerCase();
            const type = document.getElementById('typeFilter').value.toLowerCase();
            const store = document.getElementById('storeFilter').value;
            
            let visibleCount = 0;
            document.querySelectorAll('.customer-row').forEach(row => {{
                const matchesSearch = !search || 
                    row.dataset.name.includes(search) || 
                    row.dataset.email.includes(search) ||
                    row.dataset.phone.includes(search);
                const matchesType = !type || row.dataset.type === type;
                const matchesStore = !store || row.dataset.store === store;
                
                const matches = matchesSearch && matchesType && matchesStore;
                row.style.display = matches ? '' : 'none';
                if (matches) visibleCount++;
            }});
            
            document.getElementById('customerCount').textContent = visibleCount;
        }}

        function clearCustomerFilters() {{
            document.getElementById('customerSearch').value = '';
            document.getElementById('typeFilter').value = '';
            document.getElementById('storeFilter').value = '';
            filterCustomers();
        }}

        document.getElementById('customerSearch').addEventListener('input', filterCustomers);
        document.getElementById('typeFilter').addEventListener('change', filterCustomers);
        document.getElementById('storeFilter').addEventListener('change', filterCustomers);

        // Bulk selection
        document.getElementById('selectAllCustomers').addEventListener('change', function() {{
            const checkboxes = document.querySelectorAll('.customer-row:not([style*="display: none"]) .customer-checkbox');
            checkboxes.forEach(cb => cb.checked = this.checked);
            updateCustomerBulkActions();
        }});

        document.addEventListener('change', function(e) {{
            if (e.target.classList.contains('customer-checkbox')) {{
                updateCustomerBulkActions();
            }}
        }});

        function updateCustomerBulkActions() {{
            const checked = document.querySelectorAll('.customer-checkbox:checked').length;
            const bulkBar = document.getElementById('bulkActionsBar');
            const selectedCount = document.getElementById('selectedCount');
            
            if (checked > 0) {{
                bulkBar.classList.remove('hidden');
                selectedCount.textContent = checked;
            }} else {{
                bulkBar.classList.add('hidden');
            }}
        }}
        </script>
        
        {template}"#,
        header = header,
        rows = rows,
        count = customers.len(),
        template = render_customer_modal_template(stores)
    )
}

fn render_customer_modal_template(stores: &[Store]) -> String {
    let store_options = stores
        .iter()
        .map(|store| {
            format!(
                r#"<option value="{}">{}</option>"#,
                store.id,
                html_escape(&store.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-customer-template">
        <form method="post" class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Store
                    <select name="store_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">Select store</option>
                        {store_options}
                    </select>
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Full Name
                    <input name="full_name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Email
                    <input name="email" type="email" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Phone
                    <input name="phone" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
            </div>
            <label class="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                <input type="checkbox" name="is_guest" value="true" class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
                Is Guest
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        store_options = store_options
    )
}

// ===== CRUD Handlers =====

async fn create_store(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<StoreForm>,
) -> impl IntoResponse {
    let store = Store {
        id: Uuid::new_v4(),
        name: form.name.trim().to_owned(),
        domain: optional_text(form.domain),
        currency: form.currency.trim().to_uppercase(),
        timezone: form.timezone.trim().to_owned(),
        created_at: Utc::now(),
    };

    match store_repository::insert_store(&pool, &store).await {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to create store: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create store")),
            )
                .into_response()
        }
    }
}

async fn update_store(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<StoreForm>,
) -> impl IntoResponse {
    match store_repository::update_store(
        &pool,
        id,
        form.name.trim(),
        &optional_text(form.domain),
        form.currency.trim(),
        form.timezone.trim(),
    )
    .await
    {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to update store: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update store")),
            )
                .into_response()
        }
    }
}

async fn delete_store(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match store_repository::delete_store(&pool, id).await {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to delete store: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete store")),
            )
                .into_response()
        }
    }
}

async fn create_category(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<CategoryForm>,
) -> impl IntoResponse {
    let category = Category {
        id: Uuid::new_v4(),
        store_id: form.store_id,
        name: form.name.trim().to_owned(),
        slug: slugify(&form.name),
        description: optional_text(form.description),
        is_active: true,
        created_at: Utc::now(),
    };

    match category_repository::insert_category(&pool, &category).await {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to create category: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create category")),
            )
                .into_response()
        }
    }
}

async fn update_category(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<CategoryForm>,
) -> impl IntoResponse {
    match category_repository::update_category(
        &pool,
        id,
        form.store_id,
        form.name.trim(),
        &slugify(&form.name),
        &optional_text(form.description),
        true,
    )
    .await
    {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to update category: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update category")),
            )
                .into_response()
        }
    }
}

async fn delete_category(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match category_repository::delete_category(&pool, id).await {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to delete category: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete category")),
            )
                .into_response()
        }
    }
}

async fn create_product(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<ProductForm>,
) -> impl IntoResponse {
    let product = Product {
        id: Uuid::new_v4(),
        store_id: form.store_id,
        category_id: form.category_id,
        brand_id: None,
        title: form.title.trim().to_owned(),
        slug: slugify(&form.title),
        description: optional_text(form.description),
        price: None,
        compare_price: None,
        cost_price: None,
        is_active: form.is_active.unwrap_or(true),
        created_at: Utc::now(),
    };

    match product_repository::insert_product(&pool, &product).await {
        Ok(_) => {
            let attribute_pairs = parse_key_value_lines(form.attributes);
            let feature_pairs = parse_key_value_lines(form.features);

            for (idx, (name, value)) in attribute_pairs.into_iter().enumerate() {
                let attribute = ProductAttribute {
                    id: Uuid::new_v4(),
                    product_id: product.id,
                    name,
                    value,
                    position: idx as i32,
                    created_at: Utc::now(),
                };

                if let Err(err) =
                    product_attribute_repository::insert_attribute(&pool, &attribute).await
                {
                    eprintln!("Failed to add product attribute: {err:?}");
                }
            }

            for (label, value) in feature_pairs.into_iter() {
                let feature = ProductFeature {
                    id: Uuid::new_v4(),
                    product_id: product.id,
                    label,
                    value,
                    created_at: Utc::now(),
                };

                if let Err(err) = product_feature_repository::insert_feature(&pool, &feature).await
                {
                    eprintln!("Failed to add product feature: {err:?}");
                }
            }

            Redirect::to("/admin").into_response()
        }
        Err(err) => {
            eprintln!("Failed to create product: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create product")),
            )
                .into_response()
        }
    }
}

async fn update_product(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<UpdateProductForm>,
) -> impl IntoResponse {
    let request = UpdateProductRequest {
        title: Some(form.title.clone()),
        description: form.description.clone(),
        is_active: form.is_active,
        category_id: form.category_id,
        slug: Some(slugify(&form.title)),
    };

    match product_repository::update_product(&pool, id, &request).await {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to update product: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update product")),
            )
                .into_response()
        }
    }
}

async fn delete_product(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match product_repository::delete_product(&pool, id).await {
        Ok(_) => Redirect::to("/admin").into_response(),
        Err(err) => {
            eprintln!("Failed to delete product: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete product")),
            )
                .into_response()
        }
    }
}

// ===== Brand Handlers =====

async fn create_brand(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<BrandForm>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "INSERT INTO brands (name, slug, description) VALUES ($1, $2, $3)"
    )
    .bind(&form.name)
    .bind(&slugify(&form.name))
    .bind(&form.description)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Redirect::to("/admin/brands").into_response(),
        Err(err) => {
            eprintln!("Failed to create brand: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create brand")),
            )
                .into_response()
        }
    }
}

async fn update_brand(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<BrandForm>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE brands SET name = $1, slug = $2, description = $3, updated_at = NOW() WHERE id = $4",
    )
    .bind(&form.name)
    .bind(&slugify(&form.name))
    .bind(&form.description)
    .bind(id)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Redirect::to("/admin/brands").into_response(),
        Err(err) => {
            eprintln!("Failed to update brand: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update brand")),
            )
                .into_response()
        }
    }
}

async fn delete_brand(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM brands WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Redirect::to("/admin/brands").into_response(),
        Err(err) => {
            eprintln!("Failed to delete brand: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete brand")),
            )
                .into_response()
        }
    }
}

async fn create_attribute(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<AttributeForm>,
) -> impl IntoResponse {
    let attribute = ProductAttribute {
        id: Uuid::new_v4(),
        product_id: form.product_id,
        name: form.name.trim().to_owned(),
        value: form.value.trim().to_owned(),
        position: form.position.unwrap_or(0),
        created_at: Utc::now(),
    };

    match product_attribute_repository::insert_attribute(&pool, &attribute).await {
        Ok(_) => Redirect::to("/admin/product-attributes").into_response(),
        Err(err) => {
            eprintln!("Failed to create attribute: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create attribute")),
            )
                .into_response()
        }
    }
}

async fn update_attribute(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<AttributeForm>,
) -> impl IntoResponse {
    match product_attribute_repository::update_attribute(
        &pool,
        id,
        form.name.trim(),
        form.value.trim(),
        form.position.unwrap_or(0),
    )
    .await
    {
        Ok(_) => Redirect::to("/admin/product-attributes").into_response(),
        Err(err) => {
            eprintln!("Failed to update attribute: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update attribute")),
            )
                .into_response()
        }
    }
}

async fn delete_attribute(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match product_attribute_repository::delete_attribute(&pool, id).await {
        Ok(_) => Redirect::to("/admin/product-attributes").into_response(),
        Err(err) => {
            eprintln!("Failed to delete attribute: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete attribute")),
            )
                .into_response()
        }
    }
}

async fn create_feature(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<FeatureForm>,
) -> impl IntoResponse {
    let feature = ProductFeature {
        id: Uuid::new_v4(),
        product_id: form.product_id,
        label: form.label.trim().to_owned(),
        value: form.value.trim().to_owned(),
        created_at: Utc::now(),
    };

    match product_feature_repository::insert_feature(&pool, &feature).await {
        Ok(_) => Redirect::to("/admin/product-features").into_response(),
        Err(err) => {
            eprintln!("Failed to create feature: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create feature")),
            )
                .into_response()
        }
    }
}

async fn update_feature(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Form(form): Form<FeatureForm>,
) -> impl IntoResponse {
    match product_feature_repository::update_feature(
        &pool,
        id,
        form.label.trim(),
        form.value.trim(),
    )
    .await
    {
        Ok(_) => Redirect::to("/admin/product-features").into_response(),
        Err(err) => {
            eprintln!("Failed to update feature: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update feature")),
            )
                .into_response()
        }
    }
}

async fn delete_feature(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match product_feature_repository::delete_feature(&pool, id).await {
        Ok(_) => Redirect::to("/admin/product-features").into_response(),
        Err(err) => {
            eprintln!("Failed to delete feature: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to delete feature")),
            )
                .into_response()
        }
    }
}


// ===== API Token Handlers =====

async fn api_tokens_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let tokens = sqlx::query_as::<_, AdminApiToken>(
        "SELECT * FROM api_tokens ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let auth_enabled_row = sqlx::query!("SELECT value FROM system_settings WHERE key = 'api_auth_enabled'")
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);
    
    let auth_enabled = auth_enabled_row.map(|r| r.value == "true").unwrap_or(false);

    Html(render_page(
        AdminNav::ApiTokens,
        "API Access",
        "Manage API tokens and authentication settings",
        &render_api_tokens_content(&tokens, auth_enabled),
    ))
}

async fn create_api_token(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<TokenForm>,
) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string(); // Simple UUID token for now
    
    let result = sqlx::query!(
        "INSERT INTO api_tokens (name, token, description) VALUES ($1, $2, $3)",
        form.name,
        token,
        form.description
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Redirect::to("/admin/api-tokens").into_response(),
        Err(err) => {
            eprintln!("Failed to create token: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create token")),
            )
                .into_response()
        }
    }
}

async fn toggle_api_token(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "UPDATE api_tokens SET is_active = NOT is_active WHERE id = $1",
        id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Redirect::to("/admin/api-tokens").into_response(),
        Err(err) => {
            eprintln!("Failed to toggle token: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to toggle token status")),
            )
                .into_response()
        }
    }
}

async fn update_api_auth_settings(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<SettingsForm>,
) -> impl IntoResponse {
    let enabled = form.api_auth_enabled.is_some();
    let value = if enabled { "true" } else { "false" };

    let result = sqlx::query!(
        "INSERT INTO system_settings (key, value) VALUES ('api_auth_enabled', $1) 
         ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()",
        value
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Redirect::to("/admin/api-tokens").into_response(),
        Err(err) => {
            eprintln!("Failed to update settings: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to update settings")),
            )
                .into_response()
        }
    }
}

fn render_api_tokens_content(tokens: &[AdminApiToken], auth_enabled: bool) -> String {
    let header = r#"<div class="flex items-center justify-between mb-6">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">API Tokens</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Manage access keys for third-party integrations.</p>
            </div>
            <button type="button" data-modal-target="token" data-modal-mode="create" data-modal-action="/admin/api-tokens" data-modal-title="Generate Token" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Generate Token</button>
        </div>"#;

    let settings_panel = format!(
        r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-6 border border-gray-200 dark:border-dark-border mb-8">
            <h3 class="text-base font-semibold text-gray-900 dark:text-white mb-2">Global Security Settings</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">Control whether the API requires authentication for all requests.</p>
            <form action="/admin/settings/auth" method="post">
                <label class="inline-flex items-center cursor-pointer">
                    <input type="checkbox" name="api_auth_enabled" class="sr-only peer" {}>
                    <div class="relative w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                    <span class="ms-3 text-sm font-medium text-gray-900 dark:text-gray-300">Enforce API Authentication</span>
                </label>
                <div class="mt-2">
                    <button type="submit" class="text-sm text-blue-600 dark:text-blue-400 hover:underline">Save Changes</button>
                </div>
            </form>
        </div>"#,
        if auth_enabled { "checked" } else { "" }
    );

    let rows = if tokens.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="6" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No tokens generated yet.</td></tr>"#.to_string()
    } else {
        tokens
            .iter()
            .map(|token| {
                let status_badge = if token.is_active {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Active</span>"#
                } else {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400">Revoked</span>"#
                };
                
                let toggle_label = if token.is_active { "Revoke" } else { "Enable" };
                let toggle_class = if token.is_active { "bg-red-600 hover:bg-red-700" } else { "bg-green-600 hover:bg-green-700" };

                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">
                            <div class="flex items-center gap-2">
                                <code class="text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded font-mono">{}</code>
                                <button onclick="copyToken(this, '{}')" class="p-1.5 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 rounded transition-colors" title="Copy token">
                                    <span class="material-icons text-sm">content_copy</span>
                                </button>
                            </div>
                        </td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">
                            <form method="post" action="/admin/api-tokens/{}/toggle">
                                <button type="submit" class="px-3 py-1.5 text-xs font-medium text-white rounded-lg transition-colors {}">{}</button>
                            </form>
                        </td>
                    </tr>"#,
                    html_escape(&token.name),
                    token.token,
                    token.token,
                    token.description.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    status_badge,
                    token.last_used_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "Never".to_string()),
                    token.created_at.format("%Y-%m-%d").to_string(),
                    token.id,
                    toggle_class,
                    toggle_label
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{}{header}<div class=\"bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border\">
            <table class=\"w-full\">
                <thead class=\"bg-gray-50 dark:bg-slate-800/50\">
                    <tr>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Name</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Token</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Description</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Status</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Last Used</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Created</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        
        <script>
        function copyToken(btn, token) {{
            navigator.clipboard.writeText(token).then(() => {{
                const icon = btn.querySelector('.material-icons');
                const originalText = icon.textContent;
                
                // Change to checkmark
                icon.textContent = 'check';
                icon.classList.add('text-green-500');
                btn.classList.add('bg-green-50', 'dark:bg-green-900/20');
                
                // Reset after 2 seconds
                setTimeout(() => {{
                    icon.textContent = originalText;
                    icon.classList.remove('text-green-500');
                    btn.classList.remove('bg-green-50', 'dark:bg-green-900/20');
                }}, 2000);
            }}).catch(err => {{
                console.error('Failed to copy:', err);
                alert('Failed to copy token');
            }});
        }}
        </script>
        
        {template}",
        settings_panel,
        header = header,
        rows = rows,
        template = render_token_modal_template()
    )
}

fn render_token_modal_template() -> String {
    r#"<template id="modal-token-template">
        <form method="post" class="space-y-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Name
                <input name="name" required placeholder="e.g. Mobile App, Internal Tool" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Description
                <textarea name="description" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Generate Token</button>
            </div>
        </form>
    </template>"#.to_string()
}

async fn api_docs() -> Html<String> {
    Html(render_api_docs())
}

fn render_api_docs() -> String {
    let endpoints = vec![
        ("GET", "/api/health", "Simple service heartbeat", "None"),
        ("GET", "/api/products", "List all products", "None"),
        ("POST", "/api/products", "Create a product", r#"{
  "store_id": "UUID",
  "category_id": "UUID (optional)",
  "title": "String",
  "description": "String (optional)",
  "attributes": [{"name": "String", "value": "String"}],
  "features": [{"label": "String", "value": "String"}]
}"#),
        ("GET", "/api/products/{id}", "Fetch a product by ID", "None"),
        ("PUT", "/api/products/{id}", "Update a product", r#"{
  "category_id": "UUID (optional)",
  "title": "String (optional)",
  "description": "String (optional)",
  "is_active": "Boolean (optional)",
  "slug": "String (optional)"
}"#),
        ("DELETE", "/api/products/{id}", "Remove a product", "None"),
        ("POST", "/api/stores", "Create a new store", r#"{
  "name": "String",
  "domain": "String (optional)",
  "currency": "String (e.g. USD, ILS)",
  "timezone": "String (e.g. UTC)"
}"#),
        ("GET", "/api/stores", "List all stores", "None"),
        ("GET", "/api/categories", "List all categories", "None"),
        ("POST", "/api/categories", "Create a category", r#"{
  "store_id": "UUID",
  "name": "String",
  "slug": "String",
  "description": "String (optional)",
  "is_active": "Boolean"
}"#),
        ("PUT", "/api/categories/{id}", "Update a category", r#"{
  "store_id": "UUID",
  "name": "String",
  "slug": "String",
  "description": "String (optional)",
  "is_active": "Boolean"
}"#),
        ("GET", "/api/customers", "List all customers", "None"),
        ("POST", "/api/customers", "Create a customer", r#"{
  "store_id": "UUID",
  "email": "String",
  "full_name": "String",
  "phone": "String (optional)",
  "is_guest": "Boolean"
}"#),
        ("GET", "/api/customers/{id}", "Get customer by ID", "None"),
        ("PUT", "/api/customers/{id}", "Update a customer", r#"{
  "full_name": "String",
  "email": "String",
  "phone": "String (optional)",
  "is_guest": "Boolean"
}"#),
        ("DELETE", "/api/customers/{id}", "Delete a customer", "None"),
        ("GET", "/api/brands", "List all brands", "None"),
        ("GET", "/api/attributes", "List all product attributes", "None"),
        ("POST", "/api/attributes", "Create a product attribute", r#"{
  "product_id": "UUID",
  "name": "String",
  "value": "String",
  "position": "Integer"
}"#),
        ("PUT", "/api/attributes/{id}", "Update a product attribute", r#"{
  "name": "String",
  "value": "String",
  "position": "Integer"
}"#),
        ("DELETE", "/api/attributes/{id}", "Delete a product attribute", "None"),
        ("GET", "/api/features", "List all product features", "None"),
        ("POST", "/api/features", "Create a product feature", r#"{
  "product_id": "UUID",
  "label": "String",
  "value": "String"
}"#),
        ("GET", "/api/features/{id}", "Get feature by ID", "None"),
        ("PUT", "/api/features/{id}", "Update a product feature", r#"{
  "label": "String",
  "value": "String"
}"#),
        ("DELETE", "/api/features/{id}", "Delete a product feature", "None"),
        ("GET", "/api/carts", "List all shopping carts", "None"),
        ("POST", "/api/carts", "Create a shopping cart", r#"{
  "store_id": "UUID",
  "session_id": "String (optional)"
}"#),
        ("GET", "/api/carts/{id}", "Get cart by ID", "None"),
        ("PUT", "/api/carts/{id}", "Update cart customer info", r#"{
  "customer_email": "String (optional)",
  "customer_name": "String (optional)"
}"#),
        ("DELETE", "/api/carts/{id}", "Delete a cart", "None"),
        ("POST", "/api/carts/{id}/items", "Add item to cart", r#"{
  "product_id": "UUID",
  "quantity": "Integer",
  "unit_price": "Float"
}"#),
        ("POST", "/api/cart/calculate", "Calculate cart totals (stateless)", r#"{
  "items": [{"product_id": "UUID", "quantity": 1, "unit_price": 100.0}],
  "currency": "USD",
  "vat_rate": 0.17,
  "discounts": []
}"#),
        ("POST", "/api/checkout", "Create an order from cart", r#"{
  "store_id": "UUID",
  "customer_email": "String",
  "customer_name": "String",
  "total_amount": "Float"
}"#),
        ("GET", "/api/orders", "List all orders", "None"),
        ("POST", "/api/orders", "Create a new order", r#"{
  "store_id": "UUID",
  "customer_email": "String",
  "customer_name": "String",
  "customer_phone": "String (optional)",
  "total_amount": "Float",
  "notes": "String (optional)",
  "billing_address": "JSON (optional)",
  "shipping_address": "JSON (optional)"
}"#),
        ("GET", "/api/orders/{id}", "Get order by ID", "None"),
        ("PUT", "/api/orders/{id}", "Update an order", r#"{
  "customer_email": "String (optional)",
  "customer_name": "String (optional)",
  "customer_phone": "String (optional)",
  "status": "String (optional)",
  "payment_status": "String (optional)",
  "notes": "String (optional)",
  "billing_address": "JSON (optional)",
  "shipping_address": "JSON (optional)"
}"#),
        ("DELETE", "/api/orders/{id}", "Delete an order", "None"),
        ("GET", "/api/invoices", "List all invoices", "None"),
        ("POST", "/api/invoices", "Create an invoice", r#"{
  "order_id": "UUID",
  "store_id": "UUID"
}"#),
        ("GET", "/api/invoices/{id}", "Get invoice by ID", "None"),
        ("PUT", "/api/invoices/{id}", "Update invoice status", r#"{
  "status": "String (draft|sent|paid|cancelled)"
}"#),
        ("DELETE", "/api/invoices/{id}", "Delete an invoice", "None"),
        ("GET", "/api/credit-slips", "List all credit slips", "None"),
        ("POST", "/api/credit-slips", "Create a credit slip", r#"{
  "order_id": "UUID",
  "slip_number": "String",
  "amount": "Float"
}"#),
        ("GET", "/api/credit-slips/{id}", "Get credit slip by ID", "None"),
        ("PUT", "/api/credit-slips/{id}", "Update a credit slip", r#"{
  "order_id": "UUID",
  "slip_number": "String",
  "amount": "Float"
}"#),
        ("DELETE", "/api/credit-slips/{id}", "Delete a credit slip", "None"),
        ("GET", "/api/delivery-slips", "List all delivery slips", "None"),
        ("POST", "/api/delivery-slips", "Create a delivery slip", r#"{
  "order_id": "UUID",
  "slip_number": "String",
  "tracking_number": "String (optional)"
}"#),
        ("GET", "/api/delivery-slips/{id}", "Get delivery slip by ID", "None"),
        ("PUT", "/api/delivery-slips/{id}", "Update a delivery slip", r#"{
  "order_id": "UUID",
  "slip_number": "String",
  "tracking_number": "String (optional)"
}"#),
        ("DELETE", "/api/delivery-slips/{id}", "Delete a delivery slip", "None"),
        ("GET", "/api/gift-cards", "List all gift cards", "None"),
        ("POST", "/api/gift-cards", "Create a gift card", r#"{
  "store_id": "UUID",
  "initial_balance": "Float",
  "recipient_email": "String (optional)"
}"#),
        ("GET", "/api/gift-cards/{id}", "Get gift card by ID", "None"),
        ("PUT", "/api/gift-cards/{id}", "Update a gift card", r#"{
  "code": "String",
  "balance": "Float"
}"#),
        ("DELETE", "/api/gift-cards/{id}", "Delete a gift card", "None"),
    ];

    let endpoint_rows = endpoints
        .into_iter()
        .map(|(method, path, desc, body)| {
            let method_class = match method {
                "GET" => "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400",
                "POST" => "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400",
                "PUT" => "bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400",
                "DELETE" => "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
                _ => "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400",
            };

            let curl_cmd = if body != "None" {
                format!(
                    r#"curl -X {} http://localhost:3000{} \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <span class='token-placeholder'>YOUR_TOKEN</span>" \
  -d '{}'"#,
                    method, path, body.replace('\n', " ").replace("  ", "")
                )
            } else {
                format!(
                    r#"curl -X {} http://localhost:3000{} \
  -H "Authorization: Bearer <span class='token-placeholder'>YOUR_TOKEN</span>""#,
                    method, path
                )
            };

            format!(
                r#"<div class="border border-gray-200 dark:border-dark-border rounded-lg overflow-hidden mb-4">
                    <div class="bg-gray-50 dark:bg-slate-800/50 px-4 py-3 flex items-center justify-between cursor-pointer" onclick="this.nextElementSibling.classList.toggle('hidden')">
                        <div class="flex items-center gap-3">
                            <span class="px-2 py-1 text-xs font-bold rounded {}">{}</span>
                            <code class="text-sm font-semibold text-gray-700 dark:text-gray-300">{}</code>
                        </div>
                        <span class="text-sm text-gray-500 dark:text-gray-400">{}</span>
                    </div>
                    <div class="p-4 bg-white dark:bg-dark-panel hidden">
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            <div>
                                <h4 class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Request Body Schema</h4>
                                <pre class="text-xs bg-gray-50 dark:bg-slate-900 p-3 rounded-lg overflow-x-auto font-mono text-gray-700 dark:text-gray-300">{}</pre>
                            </div>
                            <div class="relative group">
                                <h4 class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Example Usage (Curl)</h4>
                                <pre class="text-xs bg-gray-900 text-blue-300 p-3 rounded-lg overflow-x-auto font-mono pr-10">{}</pre>
                                <button onclick="copyToClipboard(this)" class="absolute top-8 right-2 p-1.5 rounded-lg bg-gray-800 text-gray-400 hover:text-white hover:bg-gray-700 transition-colors opacity-0 group-hover:opacity-100" title="Copy to clipboard">
                                    <span class="material-icons text-sm">content_copy</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>"#,
                method_class, method, path, desc, body, curl_cmd
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>RustCommerce API Docs</title>
    <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        primary: '#3b82f6',
                        dark: {{
                            bg: '#0f172a',
                            panel: '#1e293b',
                            border: '#334155'
                        }}
                    }}
                }}
            }}
        }}
    </script>
</head>
<body class="bg-gray-50 dark:bg-dark-bg text-gray-900 dark:text-gray-100 transition-colors min-h-screen p-8">
    <div class="max-w-5xl mx-auto">
        <div class="flex items-center justify-between mb-8">
            <div>
                <a href="/admin" class="text-sm text-blue-600 dark:text-blue-400 hover:underline mb-2 inline-block">← Back to Dashboard</a>
                <h1 class="text-3xl font-bold">API Documentation</h1>
                <p class="text-gray-600 dark:text-gray-400 mt-2">Comprehensive reference for the RustCommerce API.</p>
            </div>
            <button onclick="document.documentElement.classList.toggle('dark')" class="p-2 rounded-lg bg-white dark:bg-dark-panel border border-gray-200 dark:border-dark-border">
                <span class="material-icons text-gray-600 dark:text-gray-300">contrast</span>
            </button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
            <div class="bg-white dark:bg-dark-panel p-6 rounded-xl border border-gray-200 dark:border-dark-border shadow-sm">
                <h2 class="text-lg font-semibold mb-3 flex items-center gap-2">
                    <span class="material-icons text-blue-500">lock</span>
                    Authentication
                </h2>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                    Enter your API token below to update all examples.
                </p>
                <div class="mb-4">
                    <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">API Token</label>
                    <div class="relative">
                        <input type="text" id="api-token-input" placeholder="Paste your token here..." class="w-full pl-10 pr-4 py-2 rounded-lg border border-gray-200 dark:border-dark-border bg-gray-50 dark:bg-slate-900 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all" />
                        <span class="material-icons absolute left-3 top-2 text-gray-400 text-sm">key</span>
                    </div>
                </div>
                <div class="space-y-2">
                    <div class="text-xs font-mono bg-gray-100 dark:bg-slate-900 p-2 rounded border border-gray-200 dark:border-dark-border">
                        Authorization: Bearer &lt;<span class="token-placeholder">YOUR_TOKEN</span>&gt;
                    </div>
                </div>
            </div>

            <div class="bg-white dark:bg-dark-panel p-6 rounded-xl border border-gray-200 dark:border-dark-border shadow-sm">
                <h2 class="text-lg font-semibold mb-3 flex items-center gap-2">
                    <span class="material-icons text-green-500">check_circle</span>
                    Response Format
                </h2>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                    All successful responses return <code>200 OK</code> or <code>201 Created</code> with a JSON body.
                    Errors return <code>4xx</code> or <code>5xx</code> codes.
                </p>
                <div class="text-xs font-mono bg-gray-100 dark:bg-slate-900 p-2 rounded border border-gray-200 dark:border-dark-border">
                    Content-Type: application/json
                </div>
            </div>
        </div>

        <div class="space-y-6">
            <h2 class="text-xl font-bold border-b border-gray-200 dark:border-dark-border pb-2">Endpoints</h2>
            {endpoint_rows}
        </div>

        <footer class="mt-12 text-center text-sm text-gray-500 dark:text-gray-400 pb-8">
            RustCommerce API v1.0 • <a href="/admin/api-tokens" class="hover:underline">Manage Keys</a>
        </footer>
    </div>
    <script>
        const tokenInput = document.getElementById('api-token-input');
        const placeholders = document.querySelectorAll('.token-placeholder');

        tokenInput.addEventListener('input', (e) => {{
            const token = e.target.value.trim() || 'YOUR_TOKEN';
            placeholders.forEach(el => {{
                el.textContent = token;
            }});
        }});

        function copyToClipboard(btn) {{
            const pre = btn.previousElementSibling;
            const text = pre.innerText;
            navigator.clipboard.writeText(text).then(() => {{
                const icon = btn.querySelector('.material-icons');
                const original = icon.textContent;
                icon.textContent = 'check';
                icon.classList.add('text-green-500');
                setTimeout(() => {{
                    icon.textContent = original;
                    icon.classList.remove('text-green-500');
                }}, 2000);
            }});
        }}

        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {{
            document.documentElement.classList.add('dark');
        }}
    </script>
</body>
</html>
"###,
        endpoint_rows = endpoint_rows,
    )
}

// ===== Multi-Page Layout & Rendering =====

fn render_page(active_nav: AdminNav, title: &str, subtitle: &str, content: &str) -> String {
    // Determine current path from title for more granular highlighting
    let current_path = match title {
        "Categories" => "/admin/categories",
        "Products" => "/admin/products",
        "Brands" => "/admin/brands",
        "Attributes" => "/admin/product-attributes",
        "Features" => "/admin/product-features",
        "Orders" => "/admin/orders",
        "Invoices" => "/admin/invoices",
        "Credit Slips" => "/admin/credit-slips",
        "Delivery Slips" => "/admin/delivery-slips",
        "Shopping Carts" => "/admin/shopping-carts",
        "Gift Cards" => "/admin/gift-cards",
        "Customers" => "/admin/customers",
        "Customer Service Chat" => "/admin/customer-service/chat",
        "Order Messages" => "/admin/customer-service/order-messages",
        "Merchandise Returns" => "/admin/customer-service/returns",
        "API Access" => "/admin/api-tokens",
        "API Documentation" => "/admin/docs",
        _ => "/admin",
    };
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title} - RustCommerce Admin</title>
    <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        primary: '#3b82f6',
                        dark: {{
                            bg: '#0f172a',
                            panel: '#1e293b',
                            border: '#334155'
                        }}
                    }}
                }}
            }}
        }}
    </script>
    <style>
        .material-icons {{ font-size: 18px; }}
        @media (max-width: 768px) {{
            .sidebar-overlay {{ 
                position: fixed;
                inset: 0;
                background: rgba(0, 0, 0, 0.5);
                z-index: 40;
            }}
        }}
    </style>
</head>
<body class="bg-gray-50 dark:bg-dark-bg transition-colors">
    <div class="flex h-screen overflow-hidden">
        <!-- Mobile Overlay -->
        <div id="sidebarOverlay" class="hidden sidebar-overlay md:hidden" onclick="toggleMobileSidebar()"></div>
        
        <!-- Sidebar -->
        <aside id="sidebar" class="fixed md:static inset-y-0 left-0 z-50 w-64 md:w-56 bg-slate-900 dark:bg-dark-bg text-gray-200 flex flex-col overflow-y-auto border-r border-slate-800 dark:border-dark-border transform -translate-x-full md:translate-x-0 transition-transform duration-300">
            <!-- Logo & Theme Toggle -->
            <div class="p-4 border-b border-slate-800 dark:border-dark-border">
                <div class="flex items-center justify-between mb-3">
                    <h1 class="text-base font-bold tracking-wide">RustCommerce</h1>
                    <div class="flex items-center gap-2">
                        <button onclick="toggleTheme()" class="p-1.5 rounded-lg hover:bg-slate-800 transition-colors" title="Toggle theme">
                            <span class="material-icons text-sm dark:hidden">light_mode</span>
                            <span class="material-icons text-sm hidden dark:inline">dark_mode</span>
                        </button>
                        <button onclick="toggleMobileSidebar()" class="md:hidden p-1.5 rounded-lg hover:bg-slate-800 transition-colors">
                            <span class="material-icons text-sm">close</span>
                        </button>
                    </div>
                </div>
            </div>
            
            <!-- Navigation -->
            {nav}
        </aside>
        
        <!-- Main Content -->
        <main class="flex-1 overflow-y-auto w-full">
            <!-- Mobile Header -->
            <div class="md:hidden sticky top-0 z-30 bg-white dark:bg-dark-panel border-b border-gray-200 dark:border-dark-border px-4 py-3 flex items-center justify-between">
                <button onclick="toggleMobileSidebar()" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors">
                    <span class="material-icons">menu</span>
                </button>
                <h1 class="text-lg font-bold text-gray-900 dark:text-white">RustCommerce</h1>
                <button onclick="toggleTheme()" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors">
                    <span class="material-icons dark:hidden">light_mode</span>
                    <span class="material-icons hidden dark:inline">dark_mode</span>
                </button>
            </div>
            
            <div class="max-w-7xl mx-auto p-4 md:p-6">
                <!-- Header (Hidden on mobile, shown in mobile header) -->
                <header class="mb-6 hidden md:block">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{title}</h1>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">{subtitle}</p>
                </header>
                
                <!-- Mobile Page Title -->
                <div class="md:hidden mb-4">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white">{title}</h2>
                    <p class="text-xs text-gray-600 dark:text-gray-400 mt-1">{subtitle}</p>
                </div>
                
                <!-- Content -->
                {content}
            </div>
        </main>
    </div>
    <div id="admin-modal" class="hidden fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-4 py-6">
        <div class="max-w-3xl w-full bg-white dark:bg-dark-panel rounded-2xl shadow-lg border border-gray-200 dark:border-dark-border overflow-hidden">
            <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-dark-border">
                <h3 id="admin-modal-title" class="text-lg font-semibold text-gray-900 dark:text-white"></h3>
                <button id="admin-modal-close" type="button" class="text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-300">Close</button>
            </div>
            <div id="admin-modal-body" class="p-6"></div>
        </div>
    </div>
    <script>
        const adminModal = document.getElementById('admin-modal');
        const adminModalBody = document.getElementById('admin-modal-body');
        const adminModalTitle = document.getElementById('admin-modal-title');

        function closeAdminModal() {{
            adminModal.classList.add('hidden');
            adminModalBody.innerHTML = '';
        }}

        function openAdminModal(trigger) {{
            const templateId = `modal-${{trigger.dataset.modalTarget}}-template`;
            const template = document.getElementById(templateId);
            if (!template) return;
            const clone = document.importNode(template.content, true);
            const form = clone.querySelector('form');
            if (!form) return;
            if (trigger.dataset.modalAction) {{
                form.action = trigger.dataset.modalAction;
            }}

            const payloadAttr = trigger.dataset.modalPayload;
            if (payloadAttr) {{
                try {{
                    const payload = JSON.parse(payloadAttr);
                    Object.entries(payload).forEach(([key, value]) => {{
                        const field = form.querySelector(`[name="${{key}}"]`);
                        if (!field) return;
                        if (field.type === 'checkbox') {{
                            field.checked = value === true || value === 'true';
                        }} else {{
                            field.value = value ?? '';
                        }}
                    }});
                }} catch (err) {{
                    console.error('Malformed modal payload', err);
                }}
            }}

            const submitButton = form.querySelector('button[type="submit"]');
            if (submitButton) {{
                submitButton.textContent = trigger.dataset.modalMode === 'edit' ? 'Save changes' : 'Save';
            }}

            adminModalTitle.textContent = trigger.dataset.modalTitle || (trigger.dataset.modalMode === 'edit' ? 'Edit entry' : 'Add entry');
            adminModalBody.innerHTML = '';
            adminModalBody.appendChild(form);
            adminModal.classList.remove('hidden');

            if (form.dataset.orderForm) {{
                syncOrderMode(form);
            }}
        }}

        document.addEventListener('click', (event) => {{
            const trigger = event.target.closest('[data-modal-target]');
            if (!trigger) return;
            event.preventDefault();
            openAdminModal(trigger);
        }});

        document.getElementById('admin-modal-close').addEventListener('click', closeAdminModal);
        adminModal.addEventListener('click', (event) => {{
            if (event.target === adminModal) {{
                closeAdminModal();
            }}
        }});

        document.addEventListener('keydown', (event) => {{
            if (event.key === 'Escape' && !adminModal.classList.contains('hidden')) {{
                closeAdminModal();
            }}
        }});

        function syncOrderMode(form) {{
            const selected = form.querySelector('input[name="mode"]:checked');
            const mode = selected ? selected.value : 'new';
            form.querySelectorAll('[data-order-mode]').forEach((section) => {{
                const isActive = section.dataset.orderMode === mode;
                section.classList.toggle('hidden', !isActive);
                section.querySelectorAll('input, select, textarea, button').forEach((field) => {{
                    if (field.type === 'radio') {{
                        return;
                    }}
                    field.disabled = !isActive;
                }});
            }});
        }}

        document.addEventListener('change', (event) => {{
            if (!event.target.matches('input[name="mode"]')) {{
                return;
            }}
            const form = event.target.closest('form');
            if (form) {{
                syncOrderMode(form);
            }}
        }});

        document.addEventListener('click', (event) => {{
            const addButton = event.target.closest('[data-add-order-item]');
            if (addButton) {{
                const form = addButton.closest('form');
                if (!form) return;
                const template = form.querySelector('[data-order-item-template]');
                const container = form.querySelector('[data-order-items]');
                if (!template || !container) return;
                const clone = document.importNode(template.content, true);
                container.appendChild(clone);
                return;
            }}

            const removeButton = event.target.closest('[data-remove-order-item]');
            if (removeButton) {{
                const item = removeButton.closest('[data-order-item]');
                if (item) {{
                    item.remove();
                }}
            }}
        }});

        function extractOrderId(action) {{
            const match = action.match(/\/admin\/orders\/([^/]+)\/status/);
            return match ? match[1] : null;
        }}

        document.addEventListener('submit', (event) => {{
            const form = event.target;
            if (!form || !form.matches('[data-order-status-form]')) {{
                return;
            }}

            event.preventDefault();

            const submitButton = form.querySelector('button[type="submit"]');
            const originalLabel = submitButton ? submitButton.textContent : '';
            if (submitButton) {{
                submitButton.disabled = true;
                submitButton.textContent = 'Saving...';
            }}

            fetch(form.action, {{
                method: 'POST',
                body: new FormData(form),
                credentials: 'same-origin',
                headers: {{
                    'X-Requested-With': 'XMLHttpRequest'
                }}
            }})
                .then((response) => {{
                    if (!response.ok && !response.redirected) {{
                        throw new Error('Status update failed');
                    }}

                    const orderId = extractOrderId(form.action);
                    const selectedOption = form.querySelector('select[name="new_status_id"] option:checked');
                    if (!orderId || !selectedOption) {{
                        closeAdminModal();
                        return;
                    }}

                    const row = document.querySelector(`tr[data-order-id="${{orderId}}"]`);
                    const badge = row ? row.querySelector('[data-order-status]') : null;
                    if (badge) {{
                        const baseClass = 'px-2 py-1 text-xs font-medium rounded-full';
                        const statusClass = selectedOption.dataset.statusClass
                            || 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400';
                        badge.className = `${{baseClass}} ${{statusClass}}`;
                        badge.textContent = selectedOption.textContent.trim();
                    }}

                    const notesField = form.querySelector('textarea[name="notes"]');
                    const editButton = row ? row.querySelector('[data-modal-target="order-edit"]') : null;
                    if (editButton) {{
                        editButton.dataset.modalPayload = JSON.stringify({{
                            status_id: selectedOption.value,
                            notes: notesField ? notesField.value : ''
                        }});
                    }}

                    closeAdminModal();
                }})
                .catch((err) => {{
                    console.error('Order status update failed', err);
                    form.submit();
                }})
                .finally(() => {{
                    if (submitButton) {{
                        submitButton.disabled = false;
                        submitButton.textContent = originalLabel || 'Save';
                    }}
                }});
        }});

        function toggleTheme() {{
            document.documentElement.classList.toggle('dark');
            localStorage.setItem('theme', document.documentElement.classList.contains('dark') ? 'dark' : 'light');
        }}
        
        function toggleNav(btn) {{
            const container = btn.nextElementSibling;
            const icon = btn.querySelector('.material-icons');
            container.classList.toggle('hidden');
            const isHidden = container.classList.contains('hidden');
            icon.textContent = isHidden ? 'expand_more' : 'expand_less';
        }}

        // Initialize nav state
        document.addEventListener('DOMContentLoaded', () => {{
            document.querySelectorAll('[data-nav-toggle]').forEach(btn => {{
                const container = btn.nextElementSibling;
                const icon = btn.querySelector('.material-icons');
                // Check if any link inside is active (has bg-blue-500/10 class)
                // Note: The class string might be escaped or formatted, so check for part of it or the text color
                const hasActiveLink = container.querySelector('.text-blue-400') !== null;
                
                if (hasActiveLink) {{
                    container.classList.remove('hidden');
                    icon.textContent = 'expand_less';
                }} else {{
                    container.classList.add('hidden');
                    icon.textContent = 'expand_more';
                }}
            }});
        }});
        
        // Mobile sidebar toggle
        function toggleMobileSidebar() {{
            const sidebar = document.getElementById('admin-sidebar');
            const overlay = document.getElementById('mobile-overlay');
            if (sidebar && overlay) {{
                const isHidden = sidebar.classList.contains('-translate-x-full');
                if (isHidden) {{
                    sidebar.classList.remove('-translate-x-full');
                    overlay.classList.remove('hidden');
                }} else {{
                    sidebar.classList.add('-translate-x-full');
                    overlay.classList.add('hidden');
                }}
            }}
        }}
        
        // Close sidebar when clicking overlay
        const mobileOverlay = document.getElementById('mobile-overlay');
        if (mobileOverlay) {{
            mobileOverlay.addEventListener('click', toggleMobileSidebar);
        }}
        
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {{
            document.documentElement.classList.add('dark');
        }}
    </script>
</body>
</html>"#,
        title = title,
        subtitle = subtitle,
        nav = render_navigation(active_nav, current_path),
        content = content,
    )
}

fn render_navigation(active: AdminNav, current_path: &str) -> String {
    let nav_items = [
        (AdminNav::Dashboard, "dashboard", "Dashboard", "/admin"),
        (AdminNav::Catalog, "inventory_2", "Catalog", "/admin/categories"),
        (AdminNav::Orders, "receipt_long", "Orders", "/admin/orders"),
        (AdminNav::Customers, "people", "Customers", "/admin/customers"),
        (AdminNav::CustomerService, "support_agent", "Customer Service", "/admin/customer-service/chat"),
        (AdminNav::ApiTokens, "key", "API Access", "/admin/api-tokens"),
    ];

    let catalog_entries = [
        ("category", "Categories", "/admin/categories"),
        ("inventory_2", "Products", "/admin/products"),
        ("sell", "Brands", "/admin/brands"),
        ("list_alt", "Attributes", "/admin/product-attributes"),
        ("label", "Features", "/admin/product-features"),
    ];

    let order_items = [
        ("receipt_long", "Orders", "/admin/orders"),
        ("description", "Invoices", "/admin/invoices"),
        ("receipt", "Credit Slips", "/admin/credit-slips"),
        ("local_shipping", "Delivery Slips", "/admin/delivery-slips"),
        ("shopping_cart", "Shopping Carts", "/admin/shopping-carts"),
        ("card_giftcard", "Gift Cards", "/admin/gift-cards"),
    ];

    let customer_service_items = [
        ("chat", "Live Chat", "/admin/customer-service/chat"),
        ("message", "Order Messages", "/admin/customer-service/order-messages"),
        ("assignment_return", "Returns", "/admin/customer-service/returns"),
    ];

    let primary_links = nav_items
        .iter()
        .map(|(nav, icon, label, href)| {
            let active_class = if *nav == active {
                "bg-blue-500/10 text-blue-400"
            } else {
                "text-gray-400 hover:bg-slate-800 hover:text-gray-200"
            };
            format!(
                r#"<a href="{}" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium {}">
                    <span class="material-icons">{}</span>
                    <span>{}</span>
                </a>"#,
                href, active_class, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                    ");

    let catalog_links = catalog_entries
        .iter()
        .map(|(icon, label, href)| {
            let is_active = current_path == *href;
            let active_class = if is_active {
                "bg-blue-500/10 text-blue-400"
            } else {
                "text-gray-400 hover:bg-slate-800 hover:text-gray-200"
            };
            format!(
                r#"<a href="{}" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium {}">
                    <span class="material-icons">{}</span>
                    <span>{}</span>
                </a>"#,
                href, active_class, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                    ");

    let order_links = order_items
        .iter()
        .map(|(icon, label, href)| {
            let is_active = current_path == *href;
            let active_class = if is_active {
                "bg-blue-500/10 text-blue-400"
            } else {
                "text-gray-400 hover:bg-slate-800 hover:text-gray-200"
            };
            format!(
                r#"<a href="{}" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium {}">
                    <span class="material-icons">{}</span>
                    <span>{}</span>
                </a>"#,
                href, active_class, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                    ");

    let customer_service_links = customer_service_items
        .iter()
        .map(|(icon, label, href)| {
            let is_active = current_path == *href;
            let active_class = if is_active {
                "bg-blue-500/10 text-blue-400"
            } else {
                "text-gray-400 hover:bg-slate-800 hover:text-gray-200"
            };
            format!(
                r#"<a href="{}" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium {}">
                    <span class="material-icons">{}</span>
                    <span>{}</span>
                </a>"#,
                href, active_class, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                    ");

    format!(
        r#"<nav class="flex-1 px-3 py-4 space-y-6">
                <div>
                    <div class="space-y-1">
                    {}
                    </div>
                </div>
                
                <!-- Catalog Section -->
                <div>
                    <button onclick="toggleNav(this)" data-nav-toggle class="w-full flex items-center justify-between px-3 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2 hover:text-gray-300 transition-colors focus:outline-none">
                        <span>Catalog</span>
                        <span class="material-icons text-[16px]">expand_less</span>
                    </button>
                    <div class="space-y-1 transition-all duration-200">
                    {}
                    </div>
                </div>
                
                <!-- Orders Section -->
                <div>
                    <button onclick="toggleNav(this)" data-nav-toggle class="w-full flex items-center justify-between px-3 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2 hover:text-gray-300 transition-colors focus:outline-none">
                        <span>Orders</span>
                        <span class="material-icons text-[16px]">expand_less</span>
                    </button>
                    <div class="space-y-1 transition-all duration-200">
                    {}
                    </div>
                </div>
                
                <!-- Customer Service Section -->
                <div>
                    <button onclick="toggleNav(this)" data-nav-toggle class="w-full flex items-center justify-between px-3 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2 hover:text-gray-300 transition-colors focus:outline-none">
                        <span>Customer Service</span>
                        <span class="material-icons text-[16px]">expand_less</span>
                    </button>
                    <div class="space-y-1 transition-all duration-200">
                    {}
                    </div>
                </div>
                
                <!-- Docs -->
                <div>
                    <a href="/admin/docs" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium text-gray-400 hover:bg-slate-800 hover:text-gray-200">
                        <span class="material-icons">api</span>
                        <span>API Docs</span>
                    </a>
                </div>

                <div class="pt-6 border-t border-slate-800 dark:border-dark-border">
                    <a href="/admin/logout" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium text-red-400 hover:bg-slate-800 hover:text-red-300">
                        <span class="material-icons">logout</span>
                        <span>Sign Out</span>
                    </a>
                </div>
            </nav>"#,
        primary_links, catalog_links, order_links, customer_service_links
    )
}

fn render_stores_content(stores: &[Store]) -> String {
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Stores</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Create and update storefront records.</p>
            </div>
            <button type="button" data-modal-target="store" data-modal-mode="create" data-modal-action="/admin/stores" data-modal-title="Add Store" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Store</button>
        </div>"#;

    let rows = if stores.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No stores created yet.</td></tr>"#.to_string()
    } else {
        stores
            .iter()
            .map(|store| {
                let payload = json!({
                    "name": store.name,
                    "domain": store.domain.clone().unwrap_or_default(),
                    "currency": store.currency,
                    "timezone": store.timezone,
                });
                let payload_attr = html_escape(&payload.to_string());
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="store" data-modal-mode="edit" data-modal-action="/admin/stores/{}/update" data-modal-title="Edit Store" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/stores/{}/delete" onsubmit="return confirm('Delete store?');">
                                    <button type="submit" class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(&store.name),
                    store.domain.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    html_escape(&store.currency),
                    store.created_at.format("%Y-%m-%d"),
                    store.id,
                    payload_attr,
                    store.id
        )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{header}<div class=\"bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border\">
            <table class=\"w-full\">
                <thead class=\"bg-gray-50 dark:bg-slate-800/50\">
                    <tr>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Name</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Domain</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Currency</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Created</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
        template = render_store_modal_template()
    )
}

fn render_store_modal_template() -> String {
    r#"<template id="modal-store-template">
        <form method="post" class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Name
                    <input name="name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Domain
                    <input name="domain" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Currency
                    <input name="currency" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Timezone
                    <input name="timezone" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
            </div>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#.to_string()
}

fn render_categories_content(stores: &[Store], categories: &[Category]) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    
    let header = format!(r#"
        <div class="flex items-center justify-between mb-6">
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white">Categories</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Organize products and manage catalog structure</p>
            </div>
            <button type="button" data-modal-target="category" data-modal-mode="create" data-modal-action="/admin/categories" data-modal-title="Add Category" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-2">
                <span class="material-icons text-lg">add</span>
                New Category
            </button>
        </div>

        <!-- Filter Bar -->
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm border border-gray-200 dark:border-dark-border p-4 mb-4">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="md:col-span-1">
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Search</label>
                    <input type="text" id="categorySearch" placeholder="Category name..." class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Status</label>
                    <select id="statusFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Statuses</option>
                        <option value="active">Active</option>
                        <option value="inactive">Inactive</option>
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Store</label>
                    <select id="storeFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Stores</option>
                        {store_options}
                    </select>
                </div>
                <div class="flex items-end">
                    <button onclick="clearCategoryFilters()" class="w-full px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-slate-800 rounded-lg hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors">
                        Clear Filters
                    </button>
                </div>
            </div>
            <div class="mt-3 text-xs text-gray-500 dark:text-gray-400">
                Showing <span id="categoryCount">{total}</span> categories
            </div>
        </div>

        <!-- Bulk Actions Bar -->
        <div id="bulkActionsBar" class="hidden bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 mb-4">
            <div class="flex items-center justify-between">
                <span class="text-sm font-medium text-blue-900 dark:text-blue-100">
                    <span id="selectedCount">0</span> categories selected
                </span>
                <div class="flex gap-2">
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Activate Selected
                    </button>
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Deactivate Selected
                    </button>
                </div>
            </div>
        </div>
    "#,
        total = categories.len(),
        store_options = stores.iter()
            .map(|s| format!(r#"<option value="{}">{}</option>"#, s.id, html_escape(&s.name)))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let rows = if categories.is_empty() {
        r#"<tr><td colspan="9" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No categories created yet.</td></tr>"#.to_string()
    } else {
        categories
            .iter()
            .enumerate()
            .map(|(idx, cat)| {
                let store_name = store_lookup.get(&cat.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                
                let status_badge = if cat.is_active {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Active</span>"#
                } else {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400">Inactive</span>"#
                };
                
                // Mock product count
                let product_count = ((idx * 11) % 50) + 1;
                
                // Mock visibility percentage (products visible / total)
                let visibility_pct = if cat.is_active { 
                    ((idx * 7) % 40) + 60 
                } else { 
                    0 
                };
                
                let visibility_badge = if visibility_pct >= 80 {
                    format!(r#"<div class="flex items-center gap-1">
                        <span class="w-2 h-2 rounded-full bg-green-500"></span>
                        <span class="text-xs text-gray-600 dark:text-gray-400">{}%</span>
                    </div>"#, visibility_pct)
                } else if visibility_pct >= 50 {
                    format!(r#"<div class="flex items-center gap-1">
                        <span class="w-2 h-2 rounded-full bg-yellow-500"></span>
                        <span class="text-xs text-gray-600 dark:text-gray-400">{}%</span>
                    </div>"#, visibility_pct)
                } else {
                    format!(r#"<div class="flex items-center gap-1">
                        <span class="w-2 h-2 rounded-full bg-gray-400"></span>
                        <span class="text-xs text-gray-600 dark:text-gray-400">{}%</span>
                    </div>"#, visibility_pct)
                };
                
                let date_str = cat.created_at.format("%b %d, %Y").to_string();
                
                let payload = json!({
                    "store_id": cat.store_id,
                    "name": cat.name,
                    "description": cat.description.clone().unwrap_or_default(),
                });
                let payload_attr = html_escape(&payload.to_string());

                // Category icon based on name
                let icon = if cat.name.to_lowercase().contains("electronics") {
                    "devices"
                } else if cat.name.to_lowercase().contains("cloth") || cat.name.to_lowercase().contains("fashion") {
                    "checkroom"
                } else if cat.name.to_lowercase().contains("food") || cat.name.to_lowercase().contains("grocery") {
                    "restaurant"
                } else if cat.name.to_lowercase().contains("book") {
                    "menu_book"
                } else if cat.name.to_lowercase().contains("home") || cat.name.to_lowercase().contains("furniture") {
                    "home"
                } else {
                    "category"
                };

                format!(
                    r#"<tr class="category-row border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50 transition-colors group"
                        data-name="{name_lower}"
                        data-status="{status_lower}"
                        data-store="{store_id}">
                        <td class="px-4 py-3">
                            <input type="checkbox" class="category-checkbox rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                        </td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-indigo-400 to-purple-500 flex items-center justify-center text-white">
                                    <span class="material-icons text-xl">{icon}</span>
                                </div>
                                <div>
                                    <div class="font-semibold text-gray-900 dark:text-white text-sm">{name}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">/{slug}</div>
                                </div>
                            </div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm text-gray-700 dark:text-gray-300 max-w-xs truncate" title="{description_full}">{description}</div>
                        </td>
                        <td class="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">{store}</td>
                        <td class="px-4 py-3">{status_badge}</td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-medium text-gray-900 dark:text-white">{product_count}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">products</div>
                        </td>
                        <td class="px-4 py-3">{visibility_badge}</td>
                        <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{date}</td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button type="button" data-modal-target="category" data-modal-mode="edit" data-modal-action="/admin/categories/{id}/update" data-modal-title="Edit Category" data-modal-payload="{payload}"
                                    class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors" title="Edit">
                                    <span class="material-icons text-lg">edit</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors" title="View Products">
                                    <span class="material-icons text-lg">inventory_2</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-purple-600 dark:hover:text-purple-400 hover:bg-purple-50 dark:hover:bg-purple-900/20 rounded transition-colors" title="SEO Settings">
                                    <span class="material-icons text-lg">search</span>
                                </button>
                                <form method="post" action="/admin/categories/{id}/delete" style="display:inline;" onsubmit="return confirm('Delete this category? This cannot be undone.');">
                                    <button type="submit" class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors" title="Delete">
                                        <span class="material-icons text-lg">delete</span>
                                    </button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    name = html_escape(&cat.name),
                    name_lower = cat.name.to_lowercase(),
                    slug = cat.slug,
                    description = cat.description.as_deref().unwrap_or("—"),
                    description_full = cat.description.as_deref().unwrap_or("No description"),
                    store = store_name,
                    store_id = cat.store_id,
                    status_badge = status_badge,
                    status_lower = if cat.is_active { "active" } else { "inactive" },
                    product_count = product_count,
                    visibility_badge = visibility_badge,
                    date = date_str,
                    icon = icon,
                    id = cat.id,
                    payload = payload_attr
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

        format!(
            r#"{header}
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50 dark:bg-slate-800/50 border-b border-gray-200 dark:border-dark-border">
                        <tr>
                            <th class="px-4 py-3 text-left w-12">
                                <input type="checkbox" id="selectAllCategories" class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                            </th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Category</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Description</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Store</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Products</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Visibility</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Created</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            
            <!-- Pagination -->
            <div class="bg-gray-50 dark:bg-slate-800/30 px-4 py-3 border-t border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        Showing <span class="font-medium">1</span> to <span class="font-medium">{count}</span> of <span class="font-medium">{count}</span> categories
                    </div>
                    <div class="flex gap-2">
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Previous</button>
                        <button class="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 border border-blue-600 rounded-lg">1</button>
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Next</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
        // Category filtering
        function filterCategories() {{
            const search = document.getElementById('categorySearch').value.toLowerCase();
            const status = document.getElementById('statusFilter').value.toLowerCase();
            const store = document.getElementById('storeFilter').value;
            
            let visibleCount = 0;
            document.querySelectorAll('.category-row').forEach(row => {{
                const matchesSearch = !search || row.dataset.name.includes(search);
                const matchesStatus = !status || row.dataset.status === status;
                const matchesStore = !store || row.dataset.store === store;
                
                const matches = matchesSearch && matchesStatus && matchesStore;
                row.style.display = matches ? '' : 'none';
                if (matches) visibleCount++;
            }});
            
            document.getElementById('categoryCount').textContent = visibleCount;
        }}

        function clearCategoryFilters() {{
            document.getElementById('categorySearch').value = '';
            document.getElementById('statusFilter').value = '';
            document.getElementById('storeFilter').value = '';
            filterCategories();
        }}

        document.getElementById('categorySearch').addEventListener('input', filterCategories);
        document.getElementById('statusFilter').addEventListener('change', filterCategories);
        document.getElementById('storeFilter').addEventListener('change', filterCategories);

        // Bulk selection
        document.getElementById('selectAllCategories').addEventListener('change', function() {{
            const checkboxes = document.querySelectorAll('.category-row:not([style*="display: none"]) .category-checkbox');
            checkboxes.forEach(cb => cb.checked = this.checked);
            updateCategoryBulkActions();
        }});

        document.addEventListener('change', function(e) {{
            if (e.target.classList.contains('category-checkbox')) {{
                updateCategoryBulkActions();
            }}
        }});

        function updateCategoryBulkActions() {{
            const checked = document.querySelectorAll('.category-checkbox:checked').length;
            const bulkBar = document.getElementById('bulkActionsBar');
            const selectedCount = document.getElementById('selectedCount');
            
            if (checked > 0) {{
                bulkBar.classList.remove('hidden');
                selectedCount.textContent = checked;
            }} else {{
                bulkBar.classList.add('hidden');
            }}
        }}
        </script>
        
        {template}"#,
        header = header,
        rows = rows,
        count = categories.len(),
        template = render_category_modal_template(stores)
    )
}

fn render_category_modal_template(stores: &[Store]) -> String {
    let options = stores
        .iter()
        .map(|store| {
            format!(
                r#"<option value="{}">{}</option>"#,
                store.id,
                html_escape(&store.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-category-template">
        <form method="post" class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Store
                    <select name="store_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">Select store</option>
                        {options}
                    </select>
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Name
                    <input name="name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
            </div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Description
                <textarea name="description" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        options = options
    )
}

fn render_products_content(stores: &[Store], categories: &[Category], products: &[Product]) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    let category_lookup: HashMap<Uuid, &Category> = categories.iter().map(|c| (c.id, c)).collect();
    
    // Build category filter options
    let category_filter_options = categories
        .iter()
        .map(|cat| format!(r#"<option value="{}">{}</option>"#, cat.id, html_escape(&cat.name)))
        .collect::<Vec<_>>()
        .join("\n");
    
    let header = format!(r#"<div class="mb-6">
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white">Product Catalog</h2>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Manage inventory, pricing, and product information</p>
                </div>
                <div class="flex gap-3">
                    <button type="button" class="px-4 py-2 rounded-lg bg-gray-100 dark:bg-slate-800 text-gray-700 dark:text-gray-300 text-sm font-medium hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors flex items-center gap-2">
                        <span class="material-icons text-sm">file_download</span>
                        Export CSV
                    </button>
                    <button type="button" class="px-4 py-2 rounded-lg bg-gray-100 dark:bg-slate-800 text-gray-700 dark:text-gray-300 text-sm font-medium hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors flex items-center gap-2">
                        <span class="material-icons text-sm">file_upload</span>
                        Import
                    </button>
                    <button type="button" data-modal-target="product" data-modal-mode="create" data-modal-action="/admin/products" data-modal-title="Add Product" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-2">
                        <span class="material-icons text-sm">add</span>
                        New Product
                    </button>
                </div>
            </div>
            
            <!-- Filters and Bulk Actions -->
            <div class="bg-white dark:bg-dark-panel rounded-lg border border-gray-200 dark:border-dark-border p-4 mb-4">
                <div class="flex flex-wrap gap-4 items-end">
                    <div class="flex-1 min-w-[200px]">
                        <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1.5">Search</label>
                        <div class="relative">
                            <input type="text" id="productSearch" placeholder="Product name, SKU, or description..." class="w-full pl-9 pr-3 py-2 text-sm border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                            <span class="material-icons absolute left-2.5 top-2 text-gray-400 text-sm">search</span>
                        </div>
                    </div>
                    
                    <div class="w-48">
                        <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1.5">Category</label>
                        <select id="categoryFilter" class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                            <option value="">All Categories</option>
                            <option value="uncategorized">Uncategorized</option>
                            {category_options}
                        </select>
                    </div>
                    
                    <div class="w-40">
                        <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1.5">Status</label>
                        <select id="statusFilter" class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                            <option value="">All Status</option>
                            <option value="active">Active</option>
                            <option value="inactive">Inactive</option>
                        </select>
                    </div>
                    
                    <div class="w-40">
                        <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1.5">Stock</label>
                        <select id="stockFilter" class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                            <option value="">All Stock</option>
                            <option value="in-stock">In Stock</option>
                            <option value="low-stock">Low Stock</option>
                            <option value="out-of-stock">Out of Stock</option>
                        </select>
                    </div>
                    
                    <button type="button" id="clearFilters" class="px-4 py-2 text-sm bg-gray-100 dark:bg-slate-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors">
                        Clear Filters
                    </button>
                </div>
                
                <!-- Bulk Actions Bar -->
                <div id="bulkActionsBar" class="hidden mt-4 pt-4 border-t border-gray-200 dark:border-dark-border">
                    <div class="flex items-center justify-between">
                        <span class="text-sm text-gray-600 dark:text-gray-400">
                            <span id="selectedCount">0</span> items selected
                        </span>
                        <div class="flex gap-2">
                            <button type="button" class="px-3 py-1.5 text-xs font-medium bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors">
                                Activate
                            </button>
                            <button type="button" class="px-3 py-1.5 text-xs font-medium bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors">
                                Deactivate
                            </button>
                            <button type="button" class="px-3 py-1.5 text-xs font-medium bg-orange-600 text-white rounded hover:bg-orange-700 transition-colors">
                                Edit Category
                            </button>
                            <button type="button" class="px-3 py-1.5 text-xs font-medium bg-red-600 text-white rounded hover:bg-red-700 transition-colors">
                                Delete
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>"#,
        category_options = category_filter_options
    );

    let rows = if products.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border"><td colspan="11" class="px-4 py-12 text-center text-gray-500 dark:text-gray-400">
            <div class="flex flex-col items-center">
                <span class="material-icons text-6xl text-gray-300 dark:text-gray-600 mb-3">inventory_2</span>
                <p class="text-base font-medium mb-1">No products yet</p>
                <p class="text-sm">Create your first product to get started</p>
            </div>
        </td></tr>"#.to_string()
    } else {
        products
            .iter()
            .enumerate()
            .map(|(idx, product)| {
                let store_name = store_lookup.get(&product.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                let category_name = product.category_id
                    .and_then(|id| category_lookup.get(&id))
                    .map(|c| c.name.as_str())
                    .unwrap_or("—");
                
                // Mock data for enterprise features (to be implemented in backend)
                let sku = format!("SKU-{}", &product.slug.to_uppercase().chars().take(8).collect::<String>());
                let price = 99.99 + (idx as f64 * 10.0);
                let stock = if idx % 5 == 0 { 0 } else if idx % 3 == 0 { 5 } else { 150 - (idx * 10) };
                let views = 1250 - (idx * 100);
                let seo_score = 70 + (idx % 30);
                
                let status_badge = if product.is_active {
                    r#"<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-400">
                        <span class="w-1.5 h-1.5 bg-green-500 rounded-full mr-1.5"></span>
                        Active
                    </span>"#
                } else {
                    r#"<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-900/30 text-gray-800 dark:text-gray-400">
                        <span class="w-1.5 h-1.5 bg-gray-400 rounded-full mr-1.5"></span>
                        Inactive
                    </span>"#
                };
                
                let stock_badge = if stock == 0 {
                    r#"<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-400">Out of Stock</span>"#.to_string()
                } else if stock < 20 {
                    format!(r#"<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-400">Low ({} left)</span>"#, stock)
                } else {
                    format!(r#"<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-400">{} units</span>"#, stock)
                };
                
                let seo_indicator = if seo_score >= 80 {
                    r#"<div class="flex items-center gap-1" title="SEO Score: Good">
                        <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                        <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                        <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                    </div>"#
                } else if seo_score >= 50 {
                    r#"<div class="flex items-center gap-1" title="SEO Score: Fair">
                        <span class="w-2 h-2 bg-yellow-500 rounded-full"></span>
                        <span class="w-2 h-2 bg-yellow-500 rounded-full"></span>
                        <span class="w-2 h-2 bg-gray-300 dark:bg-gray-600 rounded-full"></span>
                    </div>"#
                } else {
                    r#"<div class="flex items-center gap-1" title="SEO Score: Needs Work">
                        <span class="w-2 h-2 bg-red-500 rounded-full"></span>
                        <span class="w-2 h-2 bg-gray-300 dark:bg-gray-600 rounded-full"></span>
                        <span class="w-2 h-2 bg-gray-300 dark:bg-gray-600 rounded-full"></span>
                    </div>"#
                };
                
                let payload = json!({
                    "store_id": product.store_id,
                    "category_id": product.category_id,
                    "title": product.title,
                    "description": product.description.clone().unwrap_or_default(),
                    "is_active": product.is_active,
                });
                let payload_attr = html_escape(&payload.to_string());
                
                // Stock status class for filtering
                let stock_status = if stock == 0 {
                    "out-of-stock"
                } else if stock < 20 {
                    "low-stock"
                } else {
                    "in-stock"
                };
                
                let active_status = if product.is_active { "active" } else { "inactive" };
                let category_id_str = product.category_id.map(|id| id.to_string()).unwrap_or_else(|| "uncategorized".to_string());
                
                format!(
                    r#"<tr class="product-row border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50 transition-colors group" 
                        data-title="{}" 
                        data-sku="{}" 
                        data-category="{}" 
                        data-status="{}" 
                        data-stock="{}">
                        <td class="px-3 py-2">
                            <input type="checkbox" class="product-checkbox w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500" data-id="{}">
                        </td>
                        <td class="px-3 py-2">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 bg-gray-100 dark:bg-gray-800 rounded flex items-center justify-center flex-shrink-0">
                                    <span class="material-icons text-gray-400 text-lg">image</span>
                                </div>
                                <div class="min-w-0">
                                    <div class="font-medium text-gray-900 dark:text-white text-sm truncate">{}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400 font-mono">{}</div>
                                </div>
                            </div>
                        </td>
                        <td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">{}</td>
                        <td class="px-3 py-2">
                            <div class="text-sm font-medium text-gray-900 dark:text-white">${:.2}</div>
                        </td>
                        <td class="px-3 py-2">{}</td>
                        <td class="px-3 py-2">{}</td>
                        <td class="px-3 py-2">
                            <div class="text-xs text-gray-600 dark:text-gray-400">{} views</div>
                        </td>
                        <td class="px-3 py-2">{}</td>
                        <td class="px-3 py-2 text-xs text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-3 py-2">
                            <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button type="button" data-modal-target="product" data-modal-mode="edit" data-modal-action="/admin/products/{}/update" data-modal-title="Edit Product" data-modal-payload="{}" class="p-1.5 text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors" title="Edit">
                                    <span class="material-icons text-sm">edit</span>
                                </button>
                                <button type="button" class="p-1.5 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors" title="Duplicate">
                                    <span class="material-icons text-sm">content_copy</span>
                                </button>
                                <button type="button" class="p-1.5 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors" title="View Analytics">
                                    <span class="material-icons text-sm">analytics</span>
                                </button>
                                <form method="post" class="inline" action="/admin/products/{}/delete" onsubmit="return confirm('Delete this product?');">
                                    <button type="submit" class="p-1.5 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors" title="Delete">
                                        <span class="material-icons text-sm">delete</span>
                                    </button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(&product.title.to_lowercase()),
                    sku.to_lowercase(),
                    category_id_str,
                    active_status,
                    stock_status,
                    product.id,
                    html_escape(&product.title),
                    sku,
                    html_escape(category_name),
                    price,
                    stock_badge,
                    status_badge,
                    views,
                    seo_indicator,
                    product.created_at.format("%Y-%m-%d").to_string(),
                    product.id,
                    payload_attr,
                    product.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"{header}
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <div class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="bg-gray-50 dark:bg-slate-800/50 border-b border-gray-200 dark:border-dark-border">
                        <tr>
                            <th class="px-3 py-3 text-left w-10">
                                <input type="checkbox" id="selectAll" class="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500">
                            </th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-900 dark:hover:text-white">
                                <div class="flex items-center gap-1">
                                    Product
                                    <span class="material-icons text-xs">unfold_more</span>
                                </div>
                            </th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-900 dark:hover:text-white">
                                <div class="flex items-center gap-1">
                                    Category
                                    <span class="material-icons text-xs">unfold_more</span>
                                </div>
                            </th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-900 dark:hover:text-white">
                                <div class="flex items-center gap-1">
                                    Price
                                    <span class="material-icons text-xs">unfold_more</span>
                                </div>
                            </th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-900 dark:hover:text-white">
                                <div class="flex items-center gap-1">
                                    Stock
                                    <span class="material-icons text-xs">unfold_more</span>
                                </div>
                            </th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-900 dark:hover:text-white">
                                <div class="flex items-center gap-1">
                                    Views
                                    <span class="material-icons text-xs">unfold_more</span>
                                </div>
                            </th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">SEO</th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Created</th>
                            <th class="px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            
            <!-- Pagination -->
            <div class="px-4 py-3 bg-gray-50 dark:bg-slate-800/50 border-t border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        Showing <span class="font-medium">{}</span> products
                    </div>
                    <div class="flex gap-2">
                        <button class="px-3 py-1.5 text-sm border border-gray-300 dark:border-dark-border rounded hover:bg-gray-100 dark:hover:bg-slate-700 transition-colors disabled:opacity-50" disabled>
                            Previous
                        </button>
                        <button class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors">
                            1
                        </button>
                        <button class="px-3 py-1.5 text-sm border border-gray-300 dark:border-dark-border rounded hover:bg-gray-100 dark:hover:bg-slate-700 transition-colors">
                            2
                        </button>
                        <button class="px-3 py-1.5 text-sm border border-gray-300 dark:border-dark-border rounded hover:bg-gray-100 dark:hover:bg-slate-700 transition-colors">
                            Next
                        </button>
                    </div>
                </div>
            </div>
        </div>
        
        <script>
        // Bulk selection
        document.getElementById('selectAll')?.addEventListener('change', function(e) {{
            const checkboxes = document.querySelectorAll('.product-checkbox');
            checkboxes.forEach(cb => cb.checked = e.target.checked);
            updateBulkActions();
        }});
        
        document.querySelectorAll('.product-checkbox').forEach(cb => {{
            cb.addEventListener('change', updateBulkActions);
        }});
        
        function updateBulkActions() {{
            const selected = document.querySelectorAll('.product-checkbox:checked').length;
            const bar = document.getElementById('bulkActionsBar');
            const count = document.getElementById('selectedCount');
            
            if (selected > 0) {{
                bar.classList.remove('hidden');
                count.textContent = selected;
            }} else {{
                bar.classList.add('hidden');
            }}
        }}
        </script>
        
        <script>
        // Product Filtering
        function filterProducts() {{
            const searchTerm = document.getElementById('productSearch').value.toLowerCase();
            const categoryFilter = document.getElementById('categoryFilter').value;
            const statusFilter = document.getElementById('statusFilter').value;
            const stockFilter = document.getElementById('stockFilter').value;
            
            const rows = document.querySelectorAll('.product-row');
            let visibleCount = 0;
            
            rows.forEach(row => {{
                const title = row.dataset.title || '';
                const sku = row.dataset.sku || '';
                const category = row.dataset.category || '';
                const status = row.dataset.status || '';
                const stock = row.dataset.stock || '';
                
                // Check all filters
                const matchesSearch = !searchTerm || title.includes(searchTerm) || sku.includes(searchTerm);
                const matchesCategory = !categoryFilter || category === categoryFilter;
                const matchesStatus = !statusFilter || status === statusFilter;
                const matchesStock = !stockFilter || stock === stockFilter;
                
                const isVisible = matchesSearch && matchesCategory && matchesStatus && matchesStock;
                
                if (isVisible) {{
                    row.style.display = '';
                    visibleCount++;
                }} else {{
                    row.style.display = 'none';
                }}
            }});
            
            // Update count display
            const countDisplay = document.querySelector('.text-sm.text-gray-600.dark\\:text-gray-400 .font-medium');
            if (countDisplay) {{
                countDisplay.textContent = visibleCount;
            }}
        }}
        
        // Attach filter listeners
        document.getElementById('productSearch')?.addEventListener('input', filterProducts);
        document.getElementById('categoryFilter')?.addEventListener('change', filterProducts);
        document.getElementById('statusFilter')?.addEventListener('change', filterProducts);
        document.getElementById('stockFilter')?.addEventListener('change', filterProducts);
        
        // Clear filters
        document.getElementById('clearFilters')?.addEventListener('click', function() {{
            document.getElementById('productSearch').value = '';
            document.getElementById('categoryFilter').value = '';
            document.getElementById('statusFilter').value = '';
            document.getElementById('stockFilter').value = '';
            filterProducts();
        }});
        </script>
        
        <!-- Product Modal Container -->
        <div id="productModal" class="hidden fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
            <!-- Modal content will be inserted here -->
        </div>
        
        <script>
        // Product Modal Handler
        document.querySelectorAll('[data-modal-target="product"]').forEach(btn => {{
            btn.addEventListener('click', function() {{
                const modal = document.getElementById('productModal');
                const template = document.getElementById('modal-product-template');
                const mode = this.dataset.modalMode;
                
                // Clone and insert template
                const content = template.content.cloneNode(true);
                modal.innerHTML = '';
                modal.appendChild(content);
                
                // Set form action
                const form = modal.querySelector('form');
                form.action = this.dataset.modalAction;
                
                // If edit mode, populate with data
                if (mode === 'edit' && this.dataset.modalPayload) {{
                    try {{
                        const data = JSON.parse(this.dataset.modalPayload);
                        if (data.store_id) modal.querySelector('[name="store_id"]').value = data.store_id;
                        if (data.category_id) modal.querySelector('[name="category_id"]').value = data.category_id;
                        if (data.title) modal.querySelector('[name="title"]').value = data.title;
                        if (data.description) modal.querySelector('[name="description"]').value = data.description;
                        if (data.is_active) modal.querySelector('[name="is_active"]').checked = data.is_active;
                    }} catch (e) {{
                        console.error('Failed to parse product data:', e);
                    }}
                }}
                
                modal.classList.remove('hidden');
            }});
        }});
        </script>
        
        {template}"#,
        products.len(),
        header = header,
        rows = rows,
        template = render_product_modal_template(stores, categories)
    )
}

fn render_product_modal_template(stores: &[Store], categories: &[Category]) -> String {
    let store_options = stores
        .iter()
        .map(|store| format!(r#"<option value="{}">{}</option>"#, store.id, html_escape(&store.name)))
        .collect::<Vec<_>>()
        .join("\n");

    let category_options = categories
        .iter()
        .map(|cat| format!(r#"<option value="{}">{}</option>"#, cat.id, html_escape(&cat.name)))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-product-template">
        <div class="bg-white dark:bg-dark-panel rounded-lg max-w-5xl w-full shadow-xl max-h-[90vh] overflow-hidden flex flex-col">
            <!-- Header -->
            <div class="px-6 py-4 border-b border-gray-200 dark:border-dark-border flex justify-between items-center">
                <h3 class="text-xl font-semibold text-gray-900 dark:text-white">Product Details</h3>
                <button onclick="document.getElementById('productModal').classList.add('hidden')" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
                    <span class="material-icons">close</span>
                </button>
            </div>
            
            <!-- Tabs Navigation -->
            <div class="border-b border-gray-200 dark:border-dark-border bg-gray-50 dark:bg-slate-800/50">
                <nav class="flex -mb-px px-6" role="tablist">
                    <button type="button" class="tab-btn active px-4 py-3 text-sm font-medium border-b-2 border-blue-600 text-blue-600" data-tab="general">
                        <div class="flex items-center gap-2">
                            <span class="material-icons text-sm">info</span>
                            General
                        </div>
                    </button>
                    <button type="button" class="tab-btn px-4 py-3 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300" data-tab="images">
                        <div class="flex items-center gap-2">
                            <span class="material-icons text-sm">photo_library</span>
                            Images
                        </div>
                    </button>
                    <button type="button" class="tab-btn px-4 py-3 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300" data-tab="inventory">
                        <div class="flex items-center gap-2">
                            <span class="material-icons text-sm">inventory</span>
                            Inventory
                        </div>
                    </button>
                    <button type="button" class="tab-btn px-4 py-3 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300" data-tab="pricing">
                        <div class="flex items-center gap-2">
                            <span class="material-icons text-sm">payments</span>
                            Pricing & Tax
                        </div>
                    </button>
                    <button type="button" class="tab-btn px-4 py-3 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300" data-tab="seo">
                        <div class="flex items-center gap-2">
                            <span class="material-icons text-sm">search</span>
                            SEO
                        </div>
                    </button>
                </nav>
            </div>
            
            <!-- Form Container -->
            <form method="post" class="flex-1 overflow-y-auto">
                <!-- General Tab -->
                <div class="tab-content active px-6 py-6 space-y-6" data-tab-content="general">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Store <span class="text-red-500">*</span>
                            </label>
                            <select name="store_id" required class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                                <option value="">Select store</option>
                                {store_options}
                            </select>
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Category
                            </label>
                            <select name="category_id" id="productCategory" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                                <option value="">Uncategorized</option>
                                {category_options}
                            </select>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Choose the main category for this product</p>
                        </div>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Product Name <span class="text-red-500">*</span>
                        </label>
                        <input type="text" name="title" id="productTitle" required class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="e.g., Wireless Bluetooth Headphones" />
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            SKU (Stock Keeping Unit)
                        </label>
                        <input type="text" name="sku" id="productSku" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm font-mono text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="e.g., WBH-001" />
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Unique identifier for inventory tracking</p>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Description
                        </label>
                        <textarea name="description" id="productDescription" rows="4" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="Detailed product description..."></textarea>
                    </div>
                    
                    <div class="flex items-center gap-3">
                        <label class="inline-flex items-center cursor-pointer">
                            <input type="checkbox" name="is_active" value="true" checked class="sr-only peer">
                            <div class="relative w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                            <span class="ms-3 text-sm font-medium text-gray-900 dark:text-gray-300">Active (visible in store)</span>
                        </label>
                    </div>
                </div>
                
                <!-- Images Tab -->
                <div class="tab-content hidden px-6 py-6" data-tab-content="images">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                                Product Images
                            </label>
                            
                            <!-- Image Gallery -->
                            <div id="imageGallery" class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
                                <!-- Placeholder images will be added here -->
                            </div>
                            
                            <!-- Upload Button -->
                            <div class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center hover:border-blue-500 dark:hover:border-blue-400 transition-colors cursor-pointer">
                                <input type="file" id="imageUpload" accept="image/*" multiple class="hidden">
                                <label for="imageUpload" class="cursor-pointer">
                                    <span class="material-icons text-4xl text-gray-400 mb-2">cloud_upload</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">
                                        <span class="font-medium text-blue-600 dark:text-blue-400">Click to upload</span> or drag and drop
                                    </p>
                                    <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">PNG, JPG, WebP up to 10MB</p>
                                </label>
                            </div>
                            
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
                                First image will be used as the main product image
                            </p>
                        </div>
                    </div>
                </div>
                
                <!-- Inventory Tab -->
                <div class="tab-content hidden px-6 py-6 space-y-6" data-tab-content="inventory">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Stock Quantity <span class="text-red-500">*</span>
                            </label>
                            <input type="number" name="stock_quantity" id="stockQuantity" min="0" value="0" required class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Low Stock Alert
                            </label>
                            <input type="number" name="low_stock_threshold" id="lowStockThreshold" min="0" value="10" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Alert when stock falls below this number</p>
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Stock Status
                            </label>
                            <select name="stock_status" id="stockStatus" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                                <option value="in_stock">In Stock</option>
                                <option value="low_stock">Low Stock</option>
                                <option value="out_of_stock">Out of Stock</option>
                                <option value="discontinued">Discontinued</option>
                            </select>
                        </div>
                    </div>
                    
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Weight (kg)
                            </label>
                            <input type="number" name="weight" step="0.01" min="0" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="0.00">
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Dimensions (L×W×H cm)
                            </label>
                            <input type="text" name="dimensions" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="e.g., 10×5×3">
                        </div>
                    </div>
                    
                    <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                        <div class="flex items-start gap-3">
                            <span class="material-icons text-blue-600 dark:text-blue-400 text-sm mt-0.5">info</span>
                            <div class="text-sm text-blue-800 dark:text-blue-300">
                                <p class="font-medium mb-1">Inventory Management Tips</p>
                                <ul class="list-disc list-inside space-y-1 text-xs">
                                    <li>Set low stock alerts to avoid stockouts</li>
                                    <li>Update quantities regularly for accurate inventory</li>
                                    <li>Use SKU for better tracking and organization</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
                
                <!-- Pricing & Tax Tab -->
                <div class="tab-content hidden px-6 py-6 space-y-6" data-tab-content="pricing">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Regular Price <span class="text-red-500">*</span>
                            </label>
                            <div class="relative">
                                <span class="absolute left-3 top-2 text-gray-500 dark:text-gray-400">$</span>
                                <input type="number" name="price" id="productPrice" step="0.01" min="0" required class="w-full pl-8 pr-3 py-2 rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="0.00">
                            </div>
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Sale Price
                            </label>
                            <div class="relative">
                                <span class="absolute left-3 top-2 text-gray-500 dark:text-gray-400">$</span>
                                <input type="number" name="sale_price" step="0.01" min="0" class="w-full pl-8 pr-3 py-2 rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="0.00">
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Optional discounted price</p>
                        </div>
                    </div>
                    
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Cost Price
                            </label>
                            <div class="relative">
                                <span class="absolute left-3 top-2 text-gray-500 dark:text-gray-400">$</span>
                                <input type="number" name="cost_price" step="0.01" min="0" class="w-full pl-8 pr-3 py-2 rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="0.00">
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Your cost for profit calculations</p>
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Tax Class <span class="text-red-500">*</span>
                            </label>
                            <select name="tax_class" id="taxClass" required class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                                <option value="standard">Standard Rate (VAT 17%)</option>
                                <option value="reduced">Reduced Rate (VAT 9%)</option>
                                <option value="zero">Zero Rate (0%)</option>
                                <option value="exempt">Tax Exempt</option>
                            </select>
                        </div>
                    </div>
                    
                    <!-- Pricing Summary Card -->
                    <div class="bg-gray-50 dark:bg-slate-800/50 rounded-lg p-4 border border-gray-200 dark:border-dark-border">
                        <h4 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">Pricing Summary</h4>
                        <div class="space-y-2 text-sm">
                            <div class="flex justify-between">
                                <span class="text-gray-600 dark:text-gray-400">Base Price:</span>
                                <span class="font-medium text-gray-900 dark:text-white" id="basePriceDisplay">$0.00</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-gray-600 dark:text-gray-400">Tax (17%):</span>
                                <span class="font-medium text-gray-900 dark:text-white" id="taxDisplay">$0.00</span>
                            </div>
                            <div class="flex justify-between pt-2 border-t border-gray-200 dark:border-dark-border">
                                <span class="font-semibold text-gray-900 dark:text-white">Final Price:</span>
                                <span class="font-bold text-blue-600 dark:text-blue-400" id="finalPriceDisplay">$0.00</span>
                            </div>
                            <div class="flex justify-between text-xs" id="profitDisplay">
                                <span class="text-gray-600 dark:text-gray-400">Profit Margin:</span>
                                <span class="font-medium text-green-600 dark:text-green-400">—</span>
                            </div>
                        </div>
                    </div>
                </div>
                
                <!-- SEO Tab -->
                <div class="tab-content hidden px-6 py-6 space-y-6" data-tab-content="seo">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            URL Slug
                        </label>
                        <input type="text" name="slug" id="productSlug" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm font-mono text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="auto-generated-from-title">
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Auto-generated from product name, but you can customize it</p>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Meta Title
                        </label>
                        <input type="text" name="meta_title" maxlength="60" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="Custom title for search engines">
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Recommended: 50-60 characters</p>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Meta Description
                        </label>
                        <textarea name="meta_description" maxlength="160" rows="3" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="Brief description for search results"></textarea>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Recommended: 150-160 characters</p>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Keywords (comma-separated)
                        </label>
                        <input type="text" name="keywords" class="w-full rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="wireless, bluetooth, headphones, audio">
                    </div>
                </div>
                
                <!-- Footer Actions -->
                <div class="px-6 py-4 bg-gray-50 dark:bg-slate-800/50 border-t border-gray-200 dark:border-dark-border flex justify-between items-center">
                    <button type="button" onclick="document.getElementById('productModal').classList.add('hidden')" class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-slate-800 border border-gray-300 dark:border-dark-border rounded-lg hover:bg-gray-50 dark:hover:bg-slate-700 transition-colors">
                        Cancel
                    </button>
                    <div class="flex gap-2">
                        <button type="submit" name="save_action" value="save" class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors">
                            Save Product
                        </button>
                        <button type="submit" name="save_action" value="save_and_add" class="px-4 py-2 text-sm font-medium text-white bg-green-600 rounded-lg hover:bg-green-700 transition-colors">
                            Save & Add Another
                        </button>
                    </div>
                </div>
            </form>
        </div>
        
        <script>
        // Tab switching
        document.querySelectorAll('.tab-btn').forEach(btn => {{
            btn.addEventListener('click', function() {{
                const tabName = this.dataset.tab;
                
                // Update buttons
                document.querySelectorAll('.tab-btn').forEach(b => {{
                    b.classList.remove('active', 'border-blue-600', 'text-blue-600');
                    b.classList.add('border-transparent', 'text-gray-500');
                }});
                this.classList.add('active', 'border-blue-600', 'text-blue-600');
                this.classList.remove('border-transparent', 'text-gray-500');
                
                // Update content
                document.querySelectorAll('.tab-content').forEach(content => {{
                    content.classList.add('hidden');
                    content.classList.remove('active');
                }});
                const activeContent = document.querySelector(`[data-tab-content="${{tabName}}"]`);
                activeContent.classList.remove('hidden');
                activeContent.classList.add('active');
            }});
        }});
        
        // Auto-generate slug from title
        document.getElementById('productTitle')?.addEventListener('input', function(e) {{
            const slug = e.target.value
                .toLowerCase()
                .replace(/[^a-z0-9]+/g, '-')
                .replace(/^-|-$/g, '');
            document.getElementById('productSlug').value = slug;
        }});
        
        // Price calculator
        function updatePricingSummary() {{
            const price = parseFloat(document.getElementById('productPrice')?.value || 0);
            const costPrice = parseFloat(document.querySelector('[name="cost_price"]')?.value || 0);
            const tax = price * 0.17;
            const finalPrice = price + tax;
            
            document.getElementById('basePriceDisplay').textContent = `$${{price.toFixed(2)}}`;
            document.getElementById('taxDisplay').textContent = `$${{tax.toFixed(2)}}`;
            document.getElementById('finalPriceDisplay').textContent = `$${{finalPrice.toFixed(2)}}`;
            
            if (costPrice > 0) {{
                const profit = price - costPrice;
                const margin = ((profit / price) * 100).toFixed(1);
                const profitSpan = document.getElementById('profitDisplay').querySelector('span:last-child');
                profitSpan.textContent = `$${{profit.toFixed(2)}} (${{margin}}%)`;
                profitSpan.className = profit > 0 ? 'font-medium text-green-600 dark:text-green-400' : 'font-medium text-red-600 dark:text-red-400';
            }}
        }}
        
        document.getElementById('productPrice')?.addEventListener('input', updatePricingSummary);
        document.querySelector('[name="cost_price"]')?.addEventListener('input', updatePricingSummary);
        
        // Image upload preview
        document.getElementById('imageUpload')?.addEventListener('change', function(e) {{
            const gallery = document.getElementById('imageGallery');
            Array.from(e.target.files).forEach(file => {{
                const reader = new FileReader();
                reader.onload = function(e) {{
                    const div = document.createElement('div');
                    div.className = 'relative group';
                    div.innerHTML = `
                        <img src="${{e.target.result}}" class="w-full h-32 object-cover rounded-lg border border-gray-200 dark:border-dark-border">
                        <button type="button" class="absolute top-1 right-1 p-1 bg-red-600 text-white rounded opacity-0 group-hover:opacity-100 transition-opacity">
                            <span class="material-icons text-sm">delete</span>
                        </button>
                    `;
                    gallery.appendChild(div);
                }};
                reader.readAsDataURL(file);
            }});
        }});
        </script>
    </template>"#,
        store_options = store_options,
        category_options = category_options
    )
}

fn render_brands_content(brands: &[Brand]) -> String {
    let header = format!(r#"
        <div class="flex items-center justify-between mb-6">
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white">Brands</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Manage manufacturer and brand partnerships</p>
            </div>
            <button type="button" data-modal-target="brand" data-modal-mode="create" data-modal-action="/admin/brands" data-modal-title="Add Brand" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-2">
                <span class="material-icons text-lg">add_business</span>
                New Brand
            </button>
        </div>

        <!-- Filter Bar -->
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm border border-gray-200 dark:border-dark-border p-4 mb-4">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="md:col-span-1">
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Search</label>
                    <input type="text" id="brandSearch" placeholder="Brand name..." class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Status</label>
                    <select id="statusFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Statuses</option>
                        <option value="active">Active</option>
                        <option value="inactive">Inactive</option>
                    </select>
                </div>
                <div class="flex items-end">
                    <button onclick="clearBrandFilters()" class="w-full px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-slate-800 rounded-lg hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors">
                        Clear Filters
                    </button>
                </div>
            </div>
            <div class="mt-3 text-xs text-gray-500 dark:text-gray-400">
                Showing <span id="brandCount">{total}</span> brands
            </div>
        </div>
    "#, total = brands.len());

    let rows = if brands.is_empty() {
        r#"<tr><td colspan="8" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No brands created yet.</td></tr>"#.to_string()
    } else {
        brands
            .iter()
            .enumerate()
            .map(|(idx, brand)| {
                let status_badge = if brand.is_active {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Active</span>"#
                } else {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400">Inactive</span>"#
                };
                
                // Mock product count
                let product_count = ((idx * 13) % 30) + 5;
                
                let date_str = brand.created_at.format("%b %d, %Y").to_string();
                
                let payload = json!({
                    "name": brand.name,
                    "description": brand.description.clone().unwrap_or_default(),
                });
                let payload_attr = html_escape(&payload.to_string());
                
                // Brand logo or placeholder
                let logo_html = if let Some(_logo_url) = &brand.logo_url {
                    format!(r#"<div class="w-10 h-10 rounded-lg bg-white border border-gray-200 dark:border-dark-border flex items-center justify-center">
                        <span class="text-gray-400 material-icons">image</span>
                    </div>"#)
                } else {
                    format!(r#"<div class="w-10 h-10 rounded-lg bg-gradient-to-br from-cyan-400 to-blue-500 flex items-center justify-center text-white font-bold text-sm">
                        {}
                    </div>"#,
                        brand.name.chars().next().unwrap_or('B').to_uppercase()
                    )
                };
                
                format!(
                    r#"<tr class="brand-row border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50 transition-colors group"
                        data-name="{name_lower}"
                        data-status="{status_lower}">
                        <td class="px-4 py-3">
                            <input type="checkbox" class="brand-checkbox rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                        </td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-3">
                                {logo}
                                <div>
                                    <div class="font-semibold text-gray-900 dark:text-white text-sm">{name}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">/{slug}</div>
                                </div>
                            </div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm text-gray-700 dark:text-gray-300 max-w-xs truncate" title="{description_full}">{description}</div>
                        </td>
                        <td class="px-4 py-3">{status_badge}</td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-medium text-gray-900 dark:text-white">{product_count}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">products</div>
                        </td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">#{position}</td>
                        <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{date}</td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button type="button" data-modal-target="brand" data-modal-mode="edit" data-modal-action="/admin/brands/{id}/update" data-modal-title="Edit Brand" data-modal-payload="{payload}"
                                    class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors" title="Edit">
                                    <span class="material-icons text-lg">edit</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors" title="View Products">
                                    <span class="material-icons text-lg">inventory_2</span>
                                </button>
                                <form method="post" action="/admin/brands/{id}/delete" style="display:inline;" onsubmit="return confirm('Delete this brand? This cannot be undone.');">
                                    <button type="submit" class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors" title="Delete">
                                        <span class="material-icons text-lg">delete</span>
                                    </button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    name = html_escape(&brand.name),
                    name_lower = brand.name.to_lowercase(),
                    slug = brand.slug,
                    description = brand.description.as_deref().unwrap_or("—"),
                    description_full = brand.description.as_deref().unwrap_or("No description"),
                    status_badge = status_badge,
                    status_lower = if brand.is_active { "active" } else { "inactive" },
                    product_count = product_count,
                    position = brand.position,
                    date = date_str,
                    logo = logo_html,
                    id = brand.id,
                    payload = payload_attr
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

        format!(
            r#"{header}
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50 dark:bg-slate-800/50 border-b border-gray-200 dark:border-dark-border">
                        <tr>
                            <th class="px-4 py-3 text-left w-12">
                                <input type="checkbox" id="selectAllBrands" class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                            </th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Brand</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Description</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Products</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Position</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Created</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            
            <!-- Pagination -->
            <div class="bg-gray-50 dark:bg-slate-800/30 px-4 py-3 border-t border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        Showing <span class="font-medium">{total}</span> brands
                    </div>
                </div>
            </div>
        </div>

        <script>
        // Brand filtering
        function filterBrands() {{
            const search = document.getElementById('brandSearch').value.toLowerCase();
            const status = document.getElementById('statusFilter').value.toLowerCase();
            
            let visibleCount = 0;
            document.querySelectorAll('.brand-row').forEach(row => {{
                const matchesSearch = !search || row.dataset.name.includes(search);
                const matchesStatus = !status || row.dataset.status === status;
                
                const matches = matchesSearch && matchesStatus;
                row.style.display = matches ? '' : 'none';
                if (matches) visibleCount++;
            }});
            
            document.getElementById('brandCount').textContent = visibleCount;
        }}

        function clearBrandFilters() {{
            document.getElementById('brandSearch').value = '';
            document.getElementById('statusFilter').value = '';
            filterBrands();
        }}

        document.getElementById('brandSearch').addEventListener('input', filterBrands);
        document.getElementById('statusFilter').addEventListener('change', filterBrands);

        // Bulk selection
        document.getElementById('selectAllBrands').addEventListener('change', function() {{
            const checkboxes = document.querySelectorAll('.brand-row:not([style*="display: none"]) .brand-checkbox');
            checkboxes.forEach(cb => cb.checked = this.checked);
        }});
        </script>
        
        {template}"#,
        header = header,
        rows = rows,
        total = brands.len(),
        template = render_brand_modal_template()
    )
}

fn render_brand_modal_template() -> String {
    r#"<template id="modal-brand-template">
        <form method="post" class="space-y-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Name
                <input name="name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Description
                <textarea name="description" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#.to_string()
}

fn render_attribute_section(attributes: &[ProductAttribute], products: &[Product]) -> String {
    let product_lookup: HashMap<Uuid, &Product> = products.iter().map(|product| (product.id, product)).collect();
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Attributes</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Create and edit product attributes.</p>
            </div>
            <button type="button" data-modal-target="attribute" data-modal-mode="create" data-modal-action="/admin/product-attributes" data-modal-title="Add Attribute" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Attribute</button>
        </div>"#;

    let rows = if attributes.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No attributes defined yet.</td></tr>"#.to_string()
    } else {
        attributes
            .iter()
            .map(|attribute| {
                let product_name = product_lookup
                    .get(&attribute.product_id)
                    .map(|p| p.title.as_str())
                    .unwrap_or("Unknown product");
                let payload = json!({
                    "product_id": attribute.product_id,
                    "name": attribute.name,
                    "value": attribute.value,
                    "position": attribute.position,
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="attribute" data-modal-mode="edit" data-modal-action="/admin/product-attributes/{}/update" data-modal-title="Edit Attribute" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/product-attributes/{}/delete" onsubmit="return confirm('Delete attribute?');">
                                    <button class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors" type="submit">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(product_name),
                    html_escape(&attribute.name),
                    html_escape(&attribute.value),
                    attribute.position,
                    attribute.id,
                    payload_attr,
                    attribute.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{header}<div class=\"bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border\">
            <table class=\"w-full\">
                <thead class=\"bg-gray-50 dark:bg-slate-800/50\">
                    <tr>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Product</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Name</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Value</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Position</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
        template = render_attribute_modal_template(products)
    )
}

fn render_attribute_modal_template(products: &[Product]) -> String {
    let product_options = products
        .iter()
        .map(|product| {
            format!(
                r#"<option value="{}">{}</option>"#,
                product.id,
                html_escape(&product.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-attribute-template">
        <form method="post" class="space-y-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Product
                <select name="product_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                    <option value="">Select product</option>
                    {product_options}
                </select>
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Name
                <input name="name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Value
                <input name="value" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Position
                <input name="position" type="number" min="0" value="0" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        product_options = product_options
    )
}

fn render_feature_section(features: &[ProductFeature], products: &[Product]) -> String {
    let product_lookup: HashMap<Uuid, &Product> = products.iter().map(|product| (product.id, product)).collect();
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Features</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Attach descriptive feature values per product.</p>
            </div>
            <button type="button" data-modal-target="feature" data-modal-mode="create" data-modal-action="/admin/product-features" data-modal-title="Add Feature" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Feature</button>
        </div>"#;

    let rows = if features.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="4" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No features defined yet.</td></tr>"#.to_string()
    } else {
        features
            .iter()
            .map(|feature| {
                let product_name = product_lookup
                    .get(&feature.product_id)
                    .map(|p| p.title.as_str())
                    .unwrap_or("Unknown product");
                let payload = json!({
                    "product_id": feature.product_id,
                    "label": feature.label,
                    "value": feature.value,
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="feature" data-modal-mode="edit" data-modal-action="/admin/product-features/{}/update" data-modal-title="Edit Feature" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/product-features/{}/delete" onsubmit="return confirm('Delete feature?');">
                                    <button class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors" type="submit">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(product_name),
                    html_escape(&feature.label),
                    html_escape(&feature.value),
                    feature.id,
                    payload_attr,
                    feature.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{header}<div class=\"bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border\">
            <table class=\"w-full\">
                <thead class=\"bg-gray-50 dark:bg-slate-800/50\">
                    <tr>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Product</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Label</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Value</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
        template = render_feature_modal_template(products)
    )
}

fn render_feature_modal_template(products: &[Product]) -> String {
    let product_options = products
        .iter()
        .map(|product| {
            format!(
                r#"<option value="{}">{}</option>"#,
                product.id,
                html_escape(&product.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-feature-template">
        <form method="post" class="space-y-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Product
                <select name="product_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                    <option value="">Select product</option>
                    {product_options}
                </select>
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Label
                <input name="label" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Value
                <input name="value" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        product_options = product_options
    )
}

fn render_order_modal_templates(
    stores: &[Store],
    statuses: &[OrderStatus],
    products: &[Product],
) -> String {
    let store_options = stores
        .iter()
        .map(|store| {
            format!(
                r#"<option value="{}">{}</option>"#,
                store.id,
                html_escape(&store.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let status_options = statuses
        .iter()
        .map(|status| {
            let status_class = order_status_badge_class(&status.code, &status.name);
            format!(
                r#"<option value="{}" data-status-class="{}">{}</option>"#,
                status.id,
                status_class,
                html_escape(&status.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let product_options = products
        .iter()
        .map(|product| {
            let price_label = product
                .price
                .map(|price| format!("${}", price))
                .unwrap_or_else(|| "No price".to_string());
            format!(
                r#"<option value="{}">{} • {}</option>"#,
                product.id,
                html_escape(&product.title),
                price_label
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-order-create-template">
        <form method="post" class="space-y-4" data-order-form="true">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Store
                    <select name="store_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">Select store</option>
                        {store_options}
                    </select>
                </label>
                <div class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Order Source
                    <div class="mt-2 flex flex-wrap gap-4 text-sm text-gray-600 dark:text-gray-400">
                        <label class="inline-flex items-center gap-2">
                            <input type="radio" name="mode" value="new" checked />
                            New cart
                        </label>
                        <label class="inline-flex items-center gap-2">
                            <input type="radio" name="mode" value="existing" />
                            Existing cart
                        </label>
                    </div>
                </div>
            </div>

            <div class="space-y-4" data-order-mode="new">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Customer Email
                        <input name="customer_email" type="email" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                    </label>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Customer Name
                        <input name="customer_name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                    </label>
                </div>

                <div class="flex items-center justify-between">
                    <div>
                        <h4 class="text-sm font-semibold text-gray-900 dark:text-white">Cart Items</h4>
                        <p class="text-xs text-gray-500 dark:text-gray-400">Select products and quantities for the new cart.</p>
                    </div>
                    <button type="button" data-add-order-item class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/30 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors">Add item</button>
                </div>

                <div class="space-y-3" data-order-items>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 items-end" data-order-item>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Product
                            <select name="item_product_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="">Select product</option>
                                {product_options}
                            </select>
                        </label>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Quantity
                            <input name="item_quantity" type="number" min="1" value="1" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                        </label>
                        <button type="button" data-remove-order-item class="px-3 py-2 text-xs font-medium text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-dark-border rounded-lg hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors">Remove</button>
                    </div>
                </div>

                <template data-order-item-template>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 items-end" data-order-item>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Product
                            <select name="item_product_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="">Select product</option>
                                {product_options}
                            </select>
                        </label>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Quantity
                            <input name="item_quantity" type="number" min="1" value="1" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                        </label>
                        <button type="button" data-remove-order-item class="px-3 py-2 text-xs font-medium text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-dark-border rounded-lg hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors">Remove</button>
                    </div>
                </template>
            </div>

            <div class="space-y-4 hidden" data-order-mode="existing">
                <p class="text-sm text-gray-500 dark:text-gray-400">Lookup an existing cart by ID or customer name. The order will use the cart totals.</p>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Cart ID
                        <input name="existing_cart_id" placeholder="UUID" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                    </label>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Customer Name
                        <input name="existing_customer_name" placeholder="Search by name" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                    </label>
                </div>
            </div>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>
    <template id="modal-order-edit-template">
        <form method="post" class="space-y-4" data-order-status-form="true">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Status
                <select name="new_status_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                    <option value="">Select status</option>
                    {status_options}
                </select>
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Notes
                <textarea name="notes" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        store_options = store_options,
        status_options = status_options,
        product_options = product_options
    )
}

fn render_orders_content(
    orders: &[Order],
    stores: &[Store],
    statuses: &[OrderStatus],
    products: &[Product],
) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    
    // Build status filter options
    let status_filter_options = statuses
        .iter()
        .map(|s| format!(r#"<option value="{}">{}</option>"#, html_escape(&s.code.to_lowercase()), html_escape(&s.name)))
        .collect::<Vec<_>>()
        .join("\n");

    let header = format!(r#"
        <div class="flex items-center justify-between mb-6">
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white">Orders</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Manage customer orders and fulfillment</p>
            </div>
            <button type="button" data-modal-target="order-create" data-modal-mode="create" data-modal-action="/admin/orders" data-modal-title="Create Order" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-2">
                <span class="material-icons text-lg">add</span>
                New Order
            </button>
        </div>

        <!-- Filter Bar -->
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm border border-gray-200 dark:border-dark-border p-4 mb-4">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="md:col-span-1">
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Search</label>
                    <input type="text" id="orderSearch" placeholder="Order #, customer..." class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Status</label>
                    <select id="statusFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Statuses</option>
                        {status_options}
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Payment</label>
                    <select id="paymentFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Payments</option>
                        <option value="pending">Pending</option>
                        <option value="paid">Paid</option>
                        <option value="failed">Failed</option>
                        <option value="refunded">Refunded</option>
                    </select>
                </div>
                <div class="flex items-end">
                    <button onclick="clearOrderFilters()" class="w-full px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-slate-800 rounded-lg hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors">
                        Clear Filters
                    </button>
                </div>
            </div>
            <div class="mt-3 text-xs text-gray-500 dark:text-gray-400">
                Showing <span id="orderCount">{total}</span> orders
            </div>
        </div>

        <!-- Bulk Actions Bar (hidden by default) -->
        <div id="bulkActionsBar" class="hidden bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 mb-4">
            <div class="flex items-center justify-between">
                <span class="text-sm font-medium text-blue-900 dark:text-blue-100">
                    <span id="selectedCount">0</span> orders selected
                </span>
                <div class="flex gap-2">
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Export Selected
                    </button>
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Bulk Update Status
                    </button>
                </div>
            </div>
        </div>
    "#, 
        total = orders.len(),
        status_options = status_filter_options
    );

    let rows = if orders.is_empty() {
        r#"<tr><td colspan="11" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No orders yet.</td></tr>"#.to_string()
    } else {
        orders
            .iter()
            .enumerate()
            .map(|(idx, order)| {
                let status_match = statuses.iter().find(|status| {
                    status.name.eq_ignore_ascii_case(&order.status)
                        || status.code.eq_ignore_ascii_case(&order.status)
                });
                let status_class = status_match
                    .map(|status| order_status_badge_class(&status.code, &status.name))
                    .unwrap_or_else(|| order_status_badge_class("", &order.status));
                let status_badge = format!(
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full {}">{}</span>"#,
                    status_class,
                    html_escape(&order.status)
                );

                // Payment status badge
                let payment_class = match order.payment_status.to_lowercase().as_str() {
                    "paid" | "completed" => "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400",
                    "pending" | "processing" => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400",
                    "failed" | "declined" => "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
                    "refunded" => "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400",
                    _ => "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400",
                };
                let payment_badge = format!(
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full {}">{}</span>"#,
                    payment_class,
                    html_escape(&order.payment_status)
                );

                let store_name = store_lookup
                    .get(&order.store_id)
                    .map(|s| s.name.as_str())
                    .unwrap_or("Unknown");
                
                // Mock item count (would come from order_items table in real scenario)
                let item_count = (idx % 5) + 1;
                
                // Format date
                let date_str = order.created_at.format("%b %d, %Y").to_string();
                
                let payload = json!({
                    "status_id": order.status_id.map(|id| id.to_string()),
                    "notes": order.notes.clone().unwrap_or_default(),
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="order-row border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50 transition-colors group"
                        data-order-number="{order_num_lower}"
                        data-customer="{customer_lower}"
                        data-status="{status_lower}"
                        data-payment="{payment_lower}">
                        <td class="px-4 py-3">
                            <input type="checkbox" class="order-checkbox rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                        </td>
                        <td class="px-4 py-3">
                            <div class="font-semibold text-gray-900 dark:text-white text-sm">#{order_num}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">{date}</div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="font-medium text-gray-900 dark:text-white text-sm">{customer_name}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">{customer_email}</div>
                        </td>
                        <td class="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">{store}</td>
                        <td class="px-4 py-3">
                            <div class="font-semibold text-gray-900 dark:text-white text-sm">${total}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">{item_count} items</div>
                        </td>
                        <td class="px-4 py-3">{status_badge}</td>
                        <td class="px-4 py-3">{payment_badge}</td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button type="button" data-modal-target="order-edit" data-modal-mode="edit" data-modal-action="/admin/orders/{order_id}/status" data-modal-title="Update Order Status" data-modal-payload="{payload}" 
                                    class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors" title="Edit Status">
                                    <span class="material-icons text-lg">edit</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors" title="View Details">
                                    <span class="material-icons text-lg">visibility</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-purple-600 dark:hover:text-purple-400 hover:bg-purple-50 dark:hover:bg-purple-900/20 rounded transition-colors" title="Download Invoice">
                                    <span class="material-icons text-lg">receipt</span>
                                </button>
                                <form method="post" action="/admin/orders/{order_id}/delete" style="display:inline;" onsubmit="return confirm('Delete this order? This cannot be undone.');">
                                    <button type="submit" class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors" title="Delete">
                                        <span class="material-icons text-lg">delete</span>
                                    </button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    order_num = html_escape(&order.order_number),
                    order_num_lower = order.order_number.to_lowercase(),
                    customer_name = html_escape(&order.customer_name),
                    customer_email = html_escape(&order.customer_email),
                    customer_lower = format!("{} {}", order.customer_name.to_lowercase(), order.customer_email.to_lowercase()),
                    store = store_name,
                    total = order.total_amount,
                    item_count = item_count,
                    date = date_str,
                    status_badge = status_badge,
                    status_lower = order.status.to_lowercase(),
                    payment_badge = payment_badge,
                    payment_lower = order.payment_status.to_lowercase(),
                    order_id = order.id,
                    payload = payload_attr
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"{header}
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50 dark:bg-slate-800/50 border-b border-gray-200 dark:border-dark-border">
                        <tr>
                            <th class="px-4 py-3 text-left w-12">
                                <input type="checkbox" id="selectAllOrders" class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                            </th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Order</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Customer</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Store</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Total</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Payment</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            
            <!-- Pagination -->
            <div class="bg-gray-50 dark:bg-slate-800/30 px-4 py-3 border-t border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        Showing <span class="font-medium">1</span> to <span class="font-medium">{count}</span> of <span class="font-medium">{count}</span> orders
                    </div>
                    <div class="flex gap-2">
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Previous</button>
                        <button class="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 border border-blue-600 rounded-lg">1</button>
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Next</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
        // Order filtering
        function filterOrders() {{
            const search = document.getElementById('orderSearch').value.toLowerCase();
            const status = document.getElementById('statusFilter').value.toLowerCase();
            const payment = document.getElementById('paymentFilter').value.toLowerCase();
            
            let visibleCount = 0;
            document.querySelectorAll('.order-row').forEach(row => {{
                const matchesSearch = !search || 
                    row.dataset.orderNumber.includes(search) || 
                    row.dataset.customer.includes(search);
                const matchesStatus = !status || row.dataset.status === status;
                const matchesPayment = !payment || row.dataset.payment === payment;
                
                const matches = matchesSearch && matchesStatus && matchesPayment;
                row.style.display = matches ? '' : 'none';
                if (matches) visibleCount++;
            }});
            
            document.getElementById('orderCount').textContent = visibleCount;
        }}

        function clearOrderFilters() {{
            document.getElementById('orderSearch').value = '';
            document.getElementById('statusFilter').value = '';
            document.getElementById('paymentFilter').value = '';
            filterOrders();
        }}

        document.getElementById('orderSearch').addEventListener('input', filterOrders);
        document.getElementById('statusFilter').addEventListener('change', filterOrders);
        document.getElementById('paymentFilter').addEventListener('change', filterOrders);

        // Bulk selection
        document.getElementById('selectAllOrders').addEventListener('change', function() {{
            const checkboxes = document.querySelectorAll('.order-row:not([style*="display: none"]) .order-checkbox');
            checkboxes.forEach(cb => cb.checked = this.checked);
            updateBulkActions();
        }});

        document.addEventListener('change', function(e) {{
            if (e.target.classList.contains('order-checkbox')) {{
                updateBulkActions();
            }}
        }});

        function updateBulkActions() {{
            const checked = document.querySelectorAll('.order-checkbox:checked').length;
            const bulkBar = document.getElementById('bulkActionsBar');
            const selectedCount = document.getElementById('selectedCount');
            
            if (checked > 0) {{
                bulkBar.classList.remove('hidden');
                selectedCount.textContent = checked;
            }} else {{
                bulkBar.classList.add('hidden');
            }}
        }}
        </script>
        
        {template}"#,
        header = header,
        rows = rows,
        count = orders.len(),
        template = render_order_modal_templates(stores, statuses, products)
    )
}

fn order_status_badge_class(code: &str, name: &str) -> &'static str {
    let key = if code.trim().is_empty() { name } else { code };
    let key = key.trim().to_lowercase();
    match key.as_str() {
        "pending" => "bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400",
        "confirmed" | "processing" | "paid" => {
            "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400"
        }
        "delivered" | "completed" | "fulfilled" | "shipped" => {
            "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
        }
        "canceled" | "cancelled" | "failed" | "refunded" => {
            "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
        }
        _ => "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400",
    }
}

fn render_invoices_content(invoices: &[Invoice], orders: &[Order], stores: &[Store]) -> String {
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Invoices</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Billing documents and payments</p>
            </div>
            <button type="button" data-modal-target="invoice" data-modal-mode="create" data-modal-action="/admin/invoices" data-modal-title="Create Invoice" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Create Invoice</button>
        </div>"#;

    let rows = if invoices.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="6" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No invoices yet.</td></tr>"#.to_string()
    } else {
        invoices
            .iter()
            .map(|inv| {
                let payload = json!({
                    "status": inv.status,
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">${}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="invoice" data-modal-mode="edit" data-modal-action="/admin/invoices/{}/update" data-modal-title="Update Invoice Status" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/invoices/{}/delete" onsubmit="return confirm('Delete this invoice?');">
                                    <button type="submit" class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(&inv.invoice_number),
                    html_escape(&inv.customer_name),
                    inv.created_at.format("%Y-%m-%d"),
                    inv.total_amount,
                    inv.status,
                    inv.id,
                    payload_attr,
                    inv.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{header}<div class=\"bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border\">
            <table class=\"w-full\">
                <thead class=\"bg-gray-50 dark:bg-slate-800/50\">
                    <tr>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Invoice #</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Customer</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Date</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Amount</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Status</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
        template = render_invoice_modal_template(orders, stores)
    )
}

fn render_invoice_modal_template(orders: &[Order], stores: &[Store]) -> String {
    let order_options = orders
        .iter()
        .map(|ord| {
            format!(
                r#"<option value="{}">{} - {} (${:.2})</option>"#,
                ord.id,
                html_escape(&ord.customer_name),
                ord.created_at.format("%Y-%m-%d"),
                ord.total_amount
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let store_options = stores
        .iter()
        .map(|store| {
            format!(
                r#"<option value="{}">{}</option>"#,
                store.id,
                html_escape(&store.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-invoice-template">
        <form method="post" class="space-y-4">
            <div data-create-only class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Order
                    <select name="order_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">Select order</option>
                        {order_options}
                    </select>
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Store
                    <select name="store_id" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">Select store</option>
                        {store_options}
                    </select>
                </label>
            </div>
            <div data-edit-only class="hidden">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Status
                    <select name="status" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="draft">Draft</option>
                        <option value="sent">Sent</option>
                        <option value="paid">Paid</option>
                        <option value="cancelled">Cancelled</option>
                    </select>
                </label>
            </div>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        order_options = order_options,
        store_options = store_options
    )
}

fn render_credit_slips_content(slips: &[CreditSlip], _orders: &[Order]) -> String {
    let rows = if slips.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="4" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No credit slips yet.</td></tr>"#.to_string()
    } else {
        slips
            .iter()
            .map(|slip| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">${}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                    </tr>"#,
                    html_escape(&slip.slip_number),
                    slip.created_at.format("%Y-%m-%d"),
                    slip.refund_amount,
                    slip.status
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <table class="w-full">
                <thead class="bg-gray-50 dark:bg-slate-800/50">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Slip #</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Date</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Amount</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_delivery_slips_content(slips: &[DeliverySlip], _orders: &[Order]) -> String {
    let rows = if slips.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No delivery slips yet.</td></tr>"#.to_string()
    } else {
        slips
            .iter()
            .map(|slip| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                    </tr>"#,
                    html_escape(&slip.slip_number),
                    slip.tracking_number.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    slip.carrier.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    slip.created_at.format("%Y-%m-%d"),
                    slip.status
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <table class="w-full">
                <thead class="bg-gray-50 dark:bg-slate-800/50">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Slip #</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Tracking</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Carrier</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Date</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_shopping_carts_content(carts: &[ShoppingCart]) -> String {
    let rows = if carts.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No shopping carts yet.</td></tr>"#.to_string()
    } else {
        carts
            .iter()
            .map(|cart| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">${}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3 text-gray-900 dark:text-white">{}</td>
                    </tr>"#,
                    cart.id,
                    cart.customer_email.as_deref().map(html_escape).unwrap_or_else(|| "Guest".to_string()),
                    cart.total_amount,
                    cart.created_at.format("%Y-%m-%d %H:%M"),
                    cart.status
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <table class="w-full">
                <thead class="bg-gray-50 dark:bg-slate-800/50">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Cart ID</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Customer</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Total</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Created</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_gift_cards_content(cards: &[GiftCard], _stores: &[Store]) -> String {
    let header = format!(r#"
        <div class="flex items-center justify-between mb-6">
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white">Gift Cards</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Issue and manage gift card balances</p>
            </div>
            <button data-modal-target="giftCardModal" data-modal-mode="create" data-modal-action="/admin/gift-cards/create" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-2">
                <span class="material-icons text-lg">card_giftcard</span>
                New Gift Card
            </button>
        </div>

        <!-- Filter Bar -->
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm border border-gray-200 dark:border-dark-border p-4 mb-4">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="md:col-span-1">
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Search</label>
                    <input type="text" id="giftCardSearch" placeholder="Code, recipient..." class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Status</label>
                    <select id="statusFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Statuses</option>
                        <option value="active">Active</option>
                        <option value="redeemed">Redeemed</option>
                        <option value="expired">Expired</option>
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Balance</label>
                    <select id="balanceFilter" class="w-full px-3 py-2 text-sm border border-gray-200 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">All Balances</option>
                        <option value="full">Full Balance</option>
                        <option value="partial">Partially Used</option>
                        <option value="empty">Fully Redeemed</option>
                    </select>
                </div>
                <div class="flex items-end">
                    <button onclick="clearGiftCardFilters()" class="w-full px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-slate-800 rounded-lg hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors">
                        Clear Filters
                    </button>
                </div>
            </div>
            <div class="mt-3 text-xs text-gray-500 dark:text-gray-400">
                Showing <span id="giftCardCount">{total}</span> gift cards
            </div>
        </div>

        <!-- Bulk Actions Bar -->
        <div id="bulkActionsBar" class="hidden bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 mb-4">
            <div class="flex items-center justify-between">
                <span class="text-sm font-medium text-blue-900 dark:text-blue-100">
                    <span id="selectedCount">0</span> gift cards selected
                </span>
                <div class="flex gap-2">
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Export Selected
                    </button>
                    <button class="px-3 py-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 bg-white dark:bg-slate-800 border border-blue-200 dark:border-blue-700 rounded-lg hover:bg-blue-50 dark:hover:bg-slate-700 transition-colors">
                        Deactivate Selected
                    </button>
                </div>
            </div>
        </div>
    "#, total = cards.len());

    let rows = if cards.is_empty() {
        r#"<tr><td colspan="10" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No gift cards yet.</td></tr>"#.to_string()
    } else {
        cards
            .iter()
            .map(|card| {
                let status_badge = match card.status.as_str() {
                    "active" => r#"<span class="px-2 py-1 rounded-full text-xs font-medium bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-400">Active</span>"#,
                    "redeemed" => r#"<span class="px-2 py-1 rounded-full text-xs font-medium bg-gray-100 dark:bg-gray-900/30 text-gray-800 dark:text-gray-400">Redeemed</span>"#,
                    "expired" => r#"<span class="px-2 py-1 rounded-full text-xs font-medium bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-400">Expired</span>"#,
                    _ => r#"<span class="px-2 py-1 rounded-full text-xs font-medium bg-gray-100 dark:bg-gray-900/30 text-gray-800 dark:text-gray-400">Unknown</span>"#,
                };
                
                // Calculate usage percentage
                let usage_pct = if card.initial_amount > rust_decimal::Decimal::ZERO {
                    let used = card.initial_amount - card.current_balance;
                    let pct = (used / card.initial_amount) * rust_decimal::Decimal::from(100);
                    pct.to_string().parse::<f64>().unwrap_or(0.0) as i32
                } else {
                    0
                };

                let balance_status = if card.current_balance == card.initial_amount {
                    "full"
                } else if card.current_balance > rust_decimal::Decimal::ZERO {
                    "partial"
                } else {
                    "empty"
                };

                let usage_bar = format!(
                    r#"<div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                        <div class="bg-blue-600 h-1.5 rounded-full" style="width: {}%"></div>
                    </div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{}% used</span>"#,
                    usage_pct, usage_pct
                );

                let date_str = card.created_at.format("%b %d, %Y").to_string();
                let expires_str = card.expires_at
                    .map(|dt| dt.format("%b %d, %Y").to_string())
                    .unwrap_or_else(|| "—".to_string());
                
                let edit_payload = serde_json::json!({
                    "id": card.id,
                    "code": card.code,
                    "initial_amount": card.initial_amount.to_string(),
                    "current_balance": card.current_balance.to_string(),
                    "recipient_email": card.recipient_email.as_deref().unwrap_or(""),
                    "recipient_name": card.recipient_name.as_deref().unwrap_or(""),
                    "status": card.status
                });
                
                format!(
                    r#"<tr class="gift-card-row border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50 transition-colors group"
                        data-code="{code_lower}"
                        data-recipient="{recipient_lower}"
                        data-status="{status_lower}"
                        data-balance="{balance_status}">
                        <td class="px-4 py-3">
                            <input type="checkbox" class="gift-card-checkbox rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                        </td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-pink-400 to-rose-500 flex items-center justify-center text-white">
                                    <span class="material-icons text-xl">card_giftcard</span>
                                </div>
                                <div>
                                    <div class="font-semibold text-gray-900 dark:text-white text-sm font-mono">{code}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">Created {date}</div>
                                </div>
                            </div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-medium text-gray-900 dark:text-white">{recipient_name}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">{recipient_email}</div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm text-gray-700 dark:text-gray-300">{sender_name}</div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-semibold text-gray-900 dark:text-white">${initial}</div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="text-sm font-semibold text-gray-900 dark:text-white">${balance}</div>
                        </td>
                        <td class="px-4 py-3">
                            <div class="space-y-1">
                                {usage_bar}
                            </div>
                        </td>
                        <td class="px-4 py-3">{status_badge}</td>
                        <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{expires}</td>
                        <td class="px-4 py-3">
                            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button data-modal-target="giftCardModal" data-modal-mode="edit" data-modal-action="/admin/gift-cards/update" data-modal-payload='{payload}'
                                    class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors" title="Edit">
                                    <span class="material-icons text-lg">edit</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors" title="View Transactions">
                                    <span class="material-icons text-lg">receipt_long</span>
                                </button>
                                <button class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-purple-600 dark:hover:text-purple-400 hover:bg-purple-50 dark:hover:bg-purple-900/20 rounded transition-colors" title="Send to Recipient">
                                    <span class="material-icons text-lg">send</span>
                                </button>
                                <form method="POST" action="/admin/gift-cards/delete" style="display:inline;" onsubmit="return confirm('Delete this gift card? This cannot be undone.');">
                                    <input type="hidden" name="id" value="{id}" />
                                    <button type="submit" class="p-1.5 text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors" title="Delete">
                                        <span class="material-icons text-lg">delete</span>
                                    </button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    code = html_escape(&card.code),
                    code_lower = card.code.to_lowercase(),
                    recipient_name = card.recipient_name.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    recipient_email = card.recipient_email.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    recipient_lower = format!("{} {}", 
                        card.recipient_name.as_deref().unwrap_or("").to_lowercase(),
                        card.recipient_email.as_deref().unwrap_or("").to_lowercase()
                    ),
                    sender_name = card.sender_name.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    initial = card.initial_amount,
                    balance = card.current_balance,
                    usage_bar = usage_bar,
                    status_badge = status_badge,
                    status_lower = card.status.to_lowercase(),
                    balance_status = balance_status,
                    expires = expires_str,
                    date = date_str,
                    payload = serde_json::to_string(&edit_payload).unwrap_or_default().replace('"', "&quot;"),
                    id = card.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"{header}
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50 dark:bg-slate-800/50 border-b border-gray-200 dark:border-dark-border">
                        <tr>
                            <th class="px-4 py-3 text-left w-12">
                                <input type="checkbox" id="selectAllGiftCards" class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:focus:ring-offset-slate-900">
                            </th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Code</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Recipient</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Sender</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Initial</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Balance</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Usage</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Expires</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
            
            <!-- Pagination -->
            <div class="bg-gray-50 dark:bg-slate-800/30 px-4 py-3 border-t border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        Showing <span class="font-medium">1</span> to <span class="font-medium">{total}</span> of <span class="font-medium">{total}</span> gift cards
                    </div>
                    <div class="flex gap-2">
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Previous</button>
                        <button class="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 border border-blue-600 rounded-lg">1</button>
                        <button disabled class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-gray-600 bg-white dark:bg-slate-800 border border-gray-200 dark:border-dark-border rounded-lg cursor-not-allowed">Next</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
        // Gift card filtering
        function filterGiftCards() {{
            const search = document.getElementById('giftCardSearch').value.toLowerCase();
            const status = document.getElementById('statusFilter').value.toLowerCase();
            const balance = document.getElementById('balanceFilter').value.toLowerCase();
            
            let visibleCount = 0;
            document.querySelectorAll('.gift-card-row').forEach(row => {{
                const matchesSearch = !search || 
                    row.dataset.code.includes(search) || 
                    row.dataset.recipient.includes(search);
                const matchesStatus = !status || row.dataset.status === status;
                const matchesBalance = !balance || row.dataset.balance === balance;
                
                const matches = matchesSearch && matchesStatus && matchesBalance;
                row.style.display = matches ? '' : 'none';
                if (matches) visibleCount++;
            }});
            
            document.getElementById('giftCardCount').textContent = visibleCount;
        }}

        function clearGiftCardFilters() {{
            document.getElementById('giftCardSearch').value = '';
            document.getElementById('statusFilter').value = '';
            document.getElementById('balanceFilter').value = '';
            filterGiftCards();
        }}

        document.getElementById('giftCardSearch').addEventListener('input', filterGiftCards);
        document.getElementById('statusFilter').addEventListener('change', filterGiftCards);
        document.getElementById('balanceFilter').addEventListener('change', filterGiftCards);

        // Bulk selection
        document.getElementById('selectAllGiftCards').addEventListener('change', function() {{
            const checkboxes = document.querySelectorAll('.gift-card-row:not([style*="display: none"]) .gift-card-checkbox');
            checkboxes.forEach(cb => cb.checked = this.checked);
            updateGiftCardBulkActions();
        }});

        document.addEventListener('change', function(e) {{
            if (e.target.classList.contains('gift-card-checkbox')) {{
                updateGiftCardBulkActions();
            }}
        }});

        function updateGiftCardBulkActions() {{
            const checked = document.querySelectorAll('.gift-card-checkbox:checked').length;
            const bulkBar = document.getElementById('bulkActionsBar');
            const selectedCount = document.getElementById('selectedCount');
            
            if (checked > 0) {{
                bulkBar.classList.remove('hidden');
                selectedCount.textContent = checked;
            }} else {{
                bulkBar.classList.add('hidden');
            }}
        }}
        </script>
        
        <!-- Gift Card Modal -->
        <div id="giftCardModal" class="hidden fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
            <div class="bg-white dark:bg-dark-panel rounded-lg max-w-lg w-full shadow-xl">
                <div class="px-6 py-4 border-b border-gray-200 dark:border-dark-border flex justify-between items-center">
                    <h3 id="modalTitle" class="text-lg font-semibold text-gray-900 dark:text-white">New Gift Card</h3>
                    <button onclick="document.getElementById('giftCardModal').classList.add('hidden')" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
                        <span class="material-icons">close</span>
                    </button>
                </div>
                <form id="giftCardForm" method="POST" class="px-6 py-4 space-y-4">
                    <input type="hidden" name="id" id="cardId" />
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Code</label>
                        <input type="text" name="code" id="cardCode" required class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="GIFT-XXXXX" />
                    </div>
                    
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Initial Amount</label>
                            <input type="number" name="initial_amount" id="cardInitialAmount" step="0.01" required class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="100.00" />
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Current Balance</label>
                            <input type="number" name="current_balance" id="cardBalance" step="0.01" required class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="100.00" />
                        </div>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Recipient Name</label>
                        <input type="text" name="recipient_name" id="cardRecipientName" class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="Optional" />
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Recipient Email</label>
                        <input type="email" name="recipient_email" id="cardRecipientEmail" class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none" placeholder="Optional" />
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Status</label>
                        <select name="status" id="cardStatus" required class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none">
                            <option value="active">Active</option>
                            <option value="redeemed">Redeemed</option>
                            <option value="expired">Expired</option>
                        </select>
                    </div>
                    
                    <div class="flex justify-end gap-3 pt-4">
                        <button type="button" onclick="document.getElementById('giftCardModal').classList.add('hidden')" class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 rounded-lg transition-colors">
                            Cancel
                        </button>
                        <button type="submit" class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors">
                            Save Gift Card
                        </button>
                    </div>
                </form>
            </div>
        </div>
        
        <script>
        document.querySelectorAll('[data-modal-target="giftCardModal"]').forEach(btn => {{
            btn.addEventListener('click', () => {{
                const modal = document.getElementById('giftCardModal');
                const form = document.getElementById('giftCardForm');
                const title = document.getElementById('modalTitle');
                const mode = btn.dataset.modalMode;
                const action = btn.dataset.modalAction;
                
                form.action = action;
                
                if (mode === 'create') {{
                    title.textContent = 'New Gift Card';
                    form.reset();
                    document.getElementById('cardId').value = '';
                    document.getElementById('cardBalance').value = '';
                }} else if (mode === 'edit') {{
                    title.textContent = 'Edit Gift Card';
                    const data = JSON.parse(btn.dataset.modalPayload);
                    document.getElementById('cardId').value = data.id;
                    document.getElementById('cardCode').value = data.code;
                    document.getElementById('cardInitialAmount').value = data.initial_amount;
                    document.getElementById('cardBalance').value = data.current_balance;
                    document.getElementById('cardRecipientName').value = data.recipient_name || '';
                    document.getElementById('cardRecipientEmail').value = data.recipient_email || '';
                    document.getElementById('cardStatus').value = data.status;
                }}
                
                modal.classList.remove('hidden');
            }});
        }});
        </script>"#,
        header = header,
        rows = rows,
        total = cards.len()
    )
}

// Helper functions
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn optional_text(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn render_error(msg: &str) -> String {
    format!(r#"<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-800 dark:text-red-400 px-4 py-3 rounded-lg">{}</div>"#, msg)
}

fn parse_key_value_lines(input: Option<String>) -> Vec<(String, String)> {
    input
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            match (parts.next(), parts.next()) {
                (Some(k), Some(v)) if !k.trim().is_empty() => {
                    Some((k.trim().to_string(), v.trim().to_string()))
                }
                _ => None,
            }
        })
        .collect()
}

// ===== Customer Service Render Functions =====

fn render_chat_content(_messages: &[ChatMessage], _customers: &[Customer]) -> String {
    r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-6 border border-gray-200 dark:border-dark-border">
        <h2 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">Customer Service Chat</h2>
        <p class="text-gray-600 dark:text-gray-400">Live chat interface - Coming soon</p>
    </div>"#.to_string()
}

fn render_order_messages_content(_messages: &[OrderMessage], _orders: &[Order]) -> String {
    r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-6 border border-gray-200 dark:border-dark-border">
        <h2 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">Order Messages</h2>
        <p class="text-gray-600 dark:text-gray-400">Customer questions about orders - Coming soon</p>
    </div>"#.to_string()
}

fn render_merchandise_returns_content(_returns: &[MerchandiseReturn], _orders: &[Order]) -> String {
    r#"<div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-6 border border-gray-200 dark:border-dark-border">
        <h2 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">Merchandise Returns</h2>
        <p class="text-gray-600 dark:text-gray-400">Product returns management - Coming soon</p>
    </div>"#.to_string()
}
