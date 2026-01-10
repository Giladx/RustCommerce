use std::collections::HashMap;

use axum::{
    extract::{Extension, Path, Query},
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
            category_repository, checkout_repository, product_attribute_repository,
            product_feature_repository, product_repository, store_repository,
        },
    },
    domain::{
        brand::Brand,
        category::Category,
        checkout::{
            CreditSlip, DeliverySlip, GiftCard, Invoice, Order, OrderStatus, ShoppingCart,
        },
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
    Docs,
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
struct OrderForm {
    store_id: Uuid,
    customer_email: String,
    customer_name: String,
    total_amount: f64,
}

#[derive(Debug, Deserialize)]
struct OrderStatusForm {
    new_status_id: Uuid,
    notes: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(dashboard_page))
        .route("/admin", get(dashboard_page))
        .route("/admin/docs", get(api_docs))
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
        .route("/admin/invoices", get(invoices_page))
        .route("/admin/credit-slips", get(credit_slips_page))
        .route("/admin/delivery-slips", get(delivery_slips_page))
        .route("/admin/shopping-carts", get(shopping_carts_page))
        .route("/admin/gift-cards", get(gift_cards_page))
        // Attributes & Features
        .route(
            "/admin/product-attributes",
            get(dashboard_page).post(create_attribute),
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
            get(dashboard_page).post(create_feature),
        )
        .route("/admin/product-features/:id/update", post(update_feature))
        .route("/admin/product-features/:id/delete", post(delete_feature))
}

async fn dashboard_page(
    Extension(pool): Extension<PgPool>,
) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let categories = category_repository::list_categories(&pool).await.unwrap_or_default();
    let products = product_repository::list_products(&pool).await.unwrap_or_default();
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    
    let content = format!(
        r#"<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-5 border border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Stores</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{}</p>
                    </div>
                    <span class="material-icons text-blue-500 text-3xl">store</span>
                </div>
            </div>
            <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-5 border border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Categories</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{}</p>
                    </div>
                    <span class="material-icons text-green-500 text-3xl">category</span>
                </div>
            </div>
            <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-5 border border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Products</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{}</p>
                    </div>
                    <span class="material-icons text-purple-500 text-3xl">inventory_2</span>
                </div>
            </div>
            <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-5 border border-gray-200 dark:border-dark-border">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Orders</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{}</p>
                    </div>
                    <span class="material-icons text-orange-500 text-3xl">receipt_long</span>
                </div>
            </div>
        </div>
        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm p-6 border border-gray-200 dark:border-dark-border">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Links</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
                <a href="/admin/stores" class="flex items-center gap-2 p-3 rounded-lg border border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors">
                    <span class="material-icons text-blue-500">store</span>
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Manage Stores</span>
                </a>
                <a href="/admin/products" class="flex items-center gap-2 p-3 rounded-lg border border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors">
                    <span class="material-icons text-purple-500">inventory_2</span>
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Manage Products</span>
                </a>
                <a href="/admin/orders" class="flex items-center gap-2 p-3 rounded-lg border border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors">
                    <span class="material-icons text-orange-500">receipt_long</span>
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">View Orders</span>
                </a>
                <a href="/admin/docs" class="flex items-center gap-2 p-3 rounded-lg border border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors">
                    <span class="material-icons text-gray-500">api</span>
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">API Documentation</span>
                </a>
            </div>
        </div>"#,
        stores.len(),
        categories.len(),
        products.len(),
        orders.len()
    );
    
    Html(render_page(
        AdminNav::Dashboard,
        "Dashboard",
        "Overview of your e-commerce platform",
        &content,
    ))
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

async fn catalog_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let categories = category_repository::list_categories(&pool).await.unwrap_or_default();
    let products = product_repository::list_products(&pool).await.unwrap_or_default();
    let brands = sqlx::query_as::<_, Brand>("SELECT * FROM brands ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    let attributes = product_attribute_repository::list_attributes(&pool)
        .await
        .unwrap_or_default();
    let features = product_feature_repository::list_features(&pool)
        .await
        .unwrap_or_default();
    Html(render_page(
        AdminNav::Catalog,
        "Catalog",
        "Manage your catalog hierarchy and details",
        &render_catalog_content(
            &stores,
            &categories,
            &products,
            &brands,
            &attributes,
            &features,
        ),
    ))
}

async fn orders_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let statuses = checkout_repository::list_order_statuses(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Orders",
        "View and manage customer orders",
        &render_orders_content(&orders, &stores, &statuses),
    ))
}

async fn create_order(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<OrderForm>,
) -> impl IntoResponse {
    let total = match Decimal::from_f64(form.total_amount) {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Html(render_error("Invalid total amount")),
            )
                .into_response();
        }
    };

    match checkout_repository::create_order(
        &pool,
        form.store_id,
        form.customer_email.trim().to_owned(),
        form.customer_name.trim().to_owned(),
        total,
    )
    .await
    {
        Ok(_) => Redirect::to("/admin/orders").into_response(),
        Err(err) => {
            eprintln!("Failed to create order: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error("Unable to create order")),
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

async fn invoices_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let invoices = checkout_repository::list_invoices(&pool, None).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Invoices",
        "Billing documents and payments",
        &render_invoices_content(&invoices),
    ))
}

async fn credit_slips_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let slips = checkout_repository::list_credit_slips(&pool, None).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Credit Slips",
        "Refunds and credits",
        &render_credit_slips_content(&slips),
    ))
}

async fn delivery_slips_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let slips = checkout_repository::list_delivery_slips(&pool, None).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Delivery Slips",
        "Shipping and delivery tracking",
        &render_delivery_slips_content(&slips),
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
    Html(render_page(
        AdminNav::Orders,
        "Gift Cards",
        "Manage gift card codes and balances",
        &render_gift_cards_content(&cards),
    ))
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
        Ok(_) => Redirect::to("/admin").into_response(),
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
        Ok(_) => Redirect::to("/admin").into_response(),
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
        Ok(_) => Redirect::to("/admin").into_response(),
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
        Ok(_) => Redirect::to("/admin").into_response(),
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
        Ok(_) => Redirect::to("/admin").into_response(),
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
        Ok(_) => Redirect::to("/admin").into_response(),
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


fn is_missing_relation(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("42P01")
    )
}

async fn api_docs() -> Html<String> {
    Html(render_api_docs())
}

fn render_api_docs() -> String {
    let endpoints = vec![
        ("GET", "/health", "Simple service heartbeat"),
        ("GET", "/products", "List all products"),
        ("POST", "/products", "Create a product with attributes"),
        ("GET", "/products/{id}", "Fetch a product by ID"),
        ("PUT", "/products/{id}", "Update a product"),
        ("DELETE", "/products/{id}", "Remove a product"),
        ("POST", "/stores", "Create a new store"),
    ];

    let endpoint_rows = endpoints
        .into_iter()
        .map(|(method, path, desc)| {
            format!(
                r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                    <td class="px-4 py-3"><span class="tag">{}</span></td>
                    <td class="px-4 py-3"><code>{}</code></td>
                    <td class="px-4 py-3">{}</td>
                </tr>"#,
                method,
                path,
                desc
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
    <style>
        body {{
            font-family: 'Inter', system-ui, sans-serif;
            background: #0f172a;
            color: #e2e8f0;
            margin: 0;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            padding: 2rem;
        }}
        .card {{
            background: #111c33;
            border-radius: 24px;
            padding: 2rem;
            max-width: 840px;
            width: 100%;
            box-shadow: 0 30px 80px rgba(15,23,42,0.45);
        }}
        h1 {{ margin-top: 0; font-size: 2.2rem; }}
        p {{ color: #94a3b8; }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 1.25rem;
            margin: 1.5rem 0;
        }}
        .panel {{
            background: rgba(255,255,255,0.03);
            border-radius: 16px;
            padding: 1rem 1.25rem;
        }}
        .panel h2 {{ margin-top: 0; font-size: 1rem; text-transform: uppercase; letter-spacing: 0.08em; color: #cbd5f5; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 1rem; }}
        th, td {{ padding: 0.65rem 0.4rem; border-bottom: 1px solid rgba(255,255,255,0.08); }}
        th {{ text-align: left; font-size: 0.75rem; letter-spacing: 0.08em; color: #94a3b8; }}
        code {{ background: rgba(148, 163, 184, 0.16); padding: 0.2rem 0.4rem; border-radius: 6px; }}
        .tag {{
            border-radius: 999px;
            padding: 0.15rem 0.65rem;
            font-weight: 600;
            background: rgba(59, 130, 246, 0.2);
            border: 1px solid rgba(59, 130, 246, 0.35);
        }}
        a {{ color: #60a5fa; }}
    </style>
</head>
<body>
    <div class="card">
        <a href="/admin" style="color:#94a3b8">← Back to dashboard</a>
        <h1>RustCommerce API</h1>
        <p>Base URL: <code>http://localhost:3000</code>. Every endpoint speaks JSON and requires the appropriate body payload. Use tools like curl, HTTPie, or Postman.</p>

        <div class="grid">
            <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
                <h2>Bootstrapping</h2>
                <p>1. Create a store via <code>POST /stores</code><br />
                   2. Add categories (optional) then create products<br />
                   3. Attach attributes &amp; features through the admin UI or JSON payloads.</p>
            </div>
            <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
                <h2>Authentication</h2>
                <p>No auth yet. The server expects trusted traffic only. Wire up API keys or JWT when deploying.</p>
            </div>
        </div>

        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <h2>Endpoints</h2>
            <table class="w-full">
                <thead class="bg-gray-50 dark:bg-slate-800/50">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Method</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Path</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Description</th>
                    </tr>
                </thead>
                <tbody>
                    {endpoint_rows}
                </tbody>
            </table>
        </div>

        <div class="bg-white dark:bg-dark-panel rounded-lg shadow-sm overflow-hidden border border-gray-200 dark:border-dark-border">
            <h2>Sample Payload</h2>
            <pre style="background:rgba(15,23,42,0.9); padding:1rem; border-radius:14px; overflow:auto;">{{
  "store_id": "00000000-0000-0000-0000-000000000000",
  "category_id": null,
  "title": "Laptop",
  "description": "14\" ultrabook",
  "attributes": [{{"name":"Color","value":"Silver"}}],
  "features": [{{"label":"CPU","value":"M3"}}]
}}</pre>
        </div>
    </div>
</body>
</html>
"###,
        endpoint_rows = endpoint_rows,
    )
}

// ===== Multi-Page Layout & Rendering =====

fn render_page(active_nav: AdminNav, title: &str, subtitle: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
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
    </style>
</head>
<body class="bg-gray-50 dark:bg-dark-bg transition-colors">
    <div class="flex h-screen">
        <!-- Sidebar -->
        <aside class="w-56 bg-slate-900 dark:bg-dark-bg text-gray-200 flex flex-col overflow-y-auto border-r border-slate-800 dark:border-dark-border">
            <!-- Logo & Theme Toggle -->
            <div class="p-4 border-b border-slate-800 dark:border-dark-border">
                <div class="flex items-center justify-between mb-3">
                    <h1 class="text-base font-bold tracking-wide">RustCommerce</h1>
                    <button onclick="toggleTheme()" class="p-1.5 rounded-lg hover:bg-slate-800 transition-colors" title="Toggle theme">
                        <span class="material-icons text-sm dark:hidden">light_mode</span>
                        <span class="material-icons text-sm hidden dark:inline">dark_mode</span>
                    </button>
                </div>
            </div>
            
            <!-- Navigation -->
            {nav}
        </aside>
        
        <!-- Main Content -->
        <main class="flex-1 overflow-y-auto">
            <div class="max-w-7xl mx-auto p-6">
                <!-- Header -->
                <header class="mb-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{title}</h1>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">{subtitle}</p>
                </header>
                
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

        function toggleTheme() {{
            document.documentElement.classList.toggle('dark');
            localStorage.setItem('theme', document.documentElement.classList.contains('dark') ? 'dark' : 'light');
        }}
        
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {{
            document.documentElement.classList.add('dark');
        }}
    </script>
</body>
</html>"#,
        title = title,
        subtitle = subtitle,
        nav = render_navigation(active_nav),
        content = content,
    )
}

fn render_navigation(active: AdminNav) -> String {
    let nav_items = [
        (AdminNav::Dashboard, "dashboard", "Dashboard", "/admin"),
        (AdminNav::Catalog, "inventory_2", "Catalog", "/admin/catalog"),
        (AdminNav::Orders, "receipt_long", "Orders", "/admin/orders"),
    ];

    let catalog_entries = [
        ("category", "Categories", "/admin/catalog#categories"),
        ("inventory_2", "Products", "/admin/catalog#products"),
        ("sell", "Brands", "/admin/catalog#brands"),
        ("list_alt", "Attributes", "/admin/catalog#attributes"),
        ("label", "Features", "/admin/catalog#features"),
    ];

    let order_items = [
        ("receipt_long", "Orders", "/admin/orders"),
        ("description", "Invoices", "/admin/invoices"),
        ("receipt", "Credit Slips", "/admin/credit-slips"),
        ("local_shipping", "Delivery Slips", "/admin/delivery-slips"),
        ("shopping_cart", "Shopping Carts", "/admin/shopping-carts"),
        ("card_giftcard", "Gift Cards", "/admin/gift-cards"),
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
            format!(
                r#"<a href="{}" class="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-medium text-gray-400 hover:bg-slate-800 hover:text-gray-200">
                    <span class="material-icons">{}</span>
                    <span>{}</span>
                </a>"#,
                href, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                    ");

    let order_links = order_items
        .iter()
        .map(|(icon, label, href)| {
            let is_active = active == AdminNav::Orders && href.contains(label.split_whitespace().next().unwrap_or("").to_lowercase().as_str());
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
                    <h3 class="px-3 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Catalog</h3>
                    <div class="space-y-1">
                    {}
                    </div>
                </div>
                
                <!-- Orders Section -->
                <div>
                    <h3 class="px-3 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Orders</h3>
                    <div class="space-y-1">
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
            </nav>"#,
        primary_links, catalog_links, order_links
    )
}

fn render_catalog_content(
    stores: &[Store],
    categories: &[Category],
    products: &[Product],
    brands: &[Brand],
    attributes: &[ProductAttribute],
    features: &[ProductFeature],
) -> String {
    format!(
        r#"<div class="space-y-10">
            <section id="categories">{}</section>
            <section id="products">{}</section>
            <section id="brands">{}</section>
            <section id="attributes">{}</section>
            <section id="features">{}</section>
        </div>"#,
        render_categories_content(stores, categories),
        render_products_content(stores, categories, products),
        render_brands_content(brands),
        render_attribute_section(attributes, products),
        render_feature_section(features, products),
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
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-4 py-3">
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
    
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Categories</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Group products and manage visibility.</p>
            </div>
            <button type="button" data-modal-target="category" data-modal-mode="create" data-modal-action="/admin/categories" data-modal-title="Add Category" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Category</button>
        </div>"#;

    let rows = if categories.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No categories created yet.</td></tr>"#.to_string()
    } else {
        categories
            .iter()
            .map(|cat| {
                let store_name = store_lookup.get(&cat.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                let status_badge = if cat.is_active {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Active</span>"#
                } else {
                    r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400">Inactive</span>"#
                };
                let payload = json!({
                    "store_id": cat.store_id,
                    "name": cat.name,
                    "description": cat.description.clone().unwrap_or_default(),
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="category" data-modal-mode="edit" data-modal-action="/admin/categories/{}/update" data-modal-title="Edit Category" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/categories/{}/delete" onsubmit="return confirm('Delete category?');">
                                    <button type="submit" class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(&cat.name),
                    html_escape(store_name),
                    cat.slug,
                    status_badge,
                    cat.id,
                    payload_attr,
                    cat.id
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
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Store</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Slug</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Status</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
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
    
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Products</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Add and update catalog items.</p>
            </div>
            <button type="button" data-modal-target="product" data-modal-mode="create" data-modal-action="/admin/products" data-modal-title="Add Product" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Product</button>
        </div>"#;

    let rows = if products.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="6" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No products created yet.</td></tr>"#.to_string()
    } else {
        products
            .iter()
            .map(|product| {
                let store_name = store_lookup.get(&product.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                let category_name = product.category_id
                    .and_then(|id| category_lookup.get(&id))
                    .map(|c| c.name.as_str())
                    .unwrap_or("—");
                let payload = json!({
                    "store_id": product.store_id,
                    "category_id": product.category_id,
                    "title": product.title,
                    "description": product.description.clone().unwrap_or_default(),
                    "is_active": product.is_active,
                });
                let payload_attr = html_escape(&payload.to_string());
                
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="product" data-modal-mode="edit" data-modal-action="/admin/products/{}/update" data-modal-title="Edit Product" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/products/{}/delete" onsubmit="return confirm('Delete product?');">
                                    <button class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors" type="submit">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(&product.title),
                    html_escape(store_name),
                    html_escape(category_name),
                    product.slug,
                    if product.is_active { "Active" } else { "Inactive" },
                    product.id,
                    payload_attr,
                    product.id
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
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Store</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Category</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Slug</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Status</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
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
                    Category
                    <select name="category_id" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                        <option value="">Unassigned</option>
                        {category_options}
                    </select>
                </label>
            </div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Title
                <input name="title" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Description
                <textarea name="description" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <label class="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                <input type="checkbox" name="is_active" value="true" class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
                Active
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Attributes (one per line, format: Name:Value)
                <textarea name="attributes" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Features (one per line, format: Label:Value)
                <textarea name="features" rows="3" class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
            </label>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>"#,
        store_options = store_options,
        category_options = category_options
    )
}

fn render_brands_content(brands: &[Brand]) -> String {
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Brands</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Manage your brand portfolio.</p>
            </div>
            <button type="button" data-modal-target="brand" data-modal-mode="create" data-modal-action="/admin/brands" data-modal-title="Add Brand" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Brand</button>
        </div>"#;

    let rows = if brands.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="4" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No brands created yet.</td></tr>"#.to_string()
    } else {
        brands
            .iter()
            .map(|brand| {
                let payload = json!({
                    "name": brand.name,
                    "description": brand.description.clone().unwrap_or_default(),
                });
                let payload_attr = html_escape(&payload.to_string());
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">
                            <div class="flex gap-2">
                                <button type="button" data-modal-target="brand" data-modal-mode="edit" data-modal-action="/admin/brands/{}/update" data-modal-title="Edit Brand" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                                <form method="post" class="inline" action="/admin/brands/{}/delete" onsubmit="return confirm('Delete brand?');">
                                    <button class="px-3 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors" type="submit">Delete</button>
                                </form>
                            </div>
                        </td>
                    </tr>"#,
                    html_escape(&brand.name),
                    brand.slug,
                    brand.description.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    brand.id,
                    payload_attr,
                    brand.id
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
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Slug</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Description</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
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
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">
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
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">
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

fn render_order_modal_templates(stores: &[Store], statuses: &[OrderStatus]) -> String {
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
            format!(
                r#"<option value="{}">{}</option>"#,
                status.id,
                html_escape(&status.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<template id="modal-order-create-template">
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
                    Customer Email
                    <input name="customer_email" type="email" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Customer Name
                    <input name="customer_name" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Total Amount
                    <input name="total_amount" type="number" step="0.01" min="0" required class="mt-1 block w-full rounded-lg border border-gray-200 dark:border-dark-border bg-white dark:bg-slate-900 px-3 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500" />
                </label>
            </div>
            <div class="text-right">
                <button type="submit" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Save</button>
            </div>
        </form>
    </template>
    <template id="modal-order-edit-template">
        <form method="post" class="space-y-4">
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
        status_options = status_options
    )
}

fn render_orders_content(orders: &[Order], stores: &[Store], statuses: &[OrderStatus]) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    let header = r#"<div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Orders</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">Track and update customer purchases.</p>
            </div>
            <button type="button" data-modal-target="order-create" data-modal-mode="create" data-modal-action="/admin/orders" data-modal-title="Create Order" class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors">Add Order</button>
        </div>"#;

    let rows = if orders.is_empty() {
        r#"<tr><td colspan="7" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No orders yet.</td></tr>"#.to_string()
    } else {
        orders
            .iter()
            .map(|order| {
                let status_badge = match order.status.as_str() {
                    "pending" => r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400">Pending</span>"#,
                    "confirmed" | "processing" => r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400">Processing</span>"#,
                    "delivered" => r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Delivered</span>"#,
                    _ => r#"<span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400">Unknown</span>"#,
                };
                let store_name = store_lookup
                    .get(&order.store_id)
                    .map(|s| s.name.as_str())
                    .unwrap_or("Unknown");
                let payload = json!({
                    "status_id": order.status_id.map(|id| id.to_string()),
                    "notes": order.notes.clone().unwrap_or_default(),
                });
                let payload_attr = html_escape(&payload.to_string());

                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">${}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">
                            <button type="button" data-modal-target="order-edit" data-modal-mode="edit" data-modal-action="/admin/orders/{}/status" data-modal-title="Update Status" data-modal-payload="{}" class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 transition-colors">Edit</button>
                        </td>
                    </tr>"#,
                    html_escape(&order.order_number),
                    html_escape(&order.customer_email),
                    html_escape(&order.customer_name),
                    store_name,
                    order.total_amount,
                    status_badge,
                    order.id,
                    payload_attr
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
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Order #</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Customer Email</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Customer Name</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Store</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Total</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Status</th>
                        <th class=\"px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider\">Actions</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>{template}",
        header = header,
        rows = rows,
        template = render_order_modal_templates(stores, statuses)
    )
}

fn render_invoices_content(invoices: &[Invoice]) -> String {
    let rows = if invoices.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No invoices yet.</td></tr>"#.to_string()
    } else {
        invoices
            .iter()
            .map(|inv| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">${}</td>
                        <td class="px-4 py-3">{}</td>
                    </tr>"#,
                    html_escape(&inv.invoice_number),
                    html_escape(&inv.customer_name),
                    inv.created_at.format("%Y-%m-%d"),
                    inv.total_amount,
                    inv.status
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
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Invoice #</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Customer</th>
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

fn render_credit_slips_content(slips: &[CreditSlip]) -> String {
    let rows = if slips.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="4" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No credit slips yet.</td></tr>"#.to_string()
    } else {
        slips
            .iter()
            .map(|slip| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">${}</td>
                        <td class="px-4 py-3">{}</td>
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

fn render_delivery_slips_content(slips: &[DeliverySlip]) -> String {
    let rows = if slips.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No delivery slips yet.</td></tr>"#.to_string()
    } else {
        slips
            .iter()
            .map(|slip| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
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
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">${}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">{}</td>
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

fn render_gift_cards_content(cards: &[GiftCard]) -> String {
    let rows = if cards.is_empty() {
        r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50"><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No gift cards yet.</td></tr>"#.to_string()
    } else {
        cards
            .iter()
            .map(|card| {
                format!(
                    r#"<tr class="border-b border-gray-200 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-slate-800/50">
                        <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">{}</td>
                        <td class="px-4 py-3">{}</td>
                        <td class="px-4 py-3">${}</td>
                        <td class="px-4 py-3">${}</td>
                        <td class="px-4 py-3">{}</td>
                    </tr>"#,
                    html_escape(&card.code),
                    card.recipient_email.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    card.initial_amount,
                    card.current_balance,
                    card.status
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
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Code</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Recipient</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Initial Amount</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Balance</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">Status</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
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
