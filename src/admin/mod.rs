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
use sqlx::PgPool;
use uuid::Uuid;

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
        checkout::{CreditSlip, DeliverySlip, GiftCard, Invoice, Order, ShoppingCart},
        product::{Product, ProductAttribute, ProductFeature},
        store::store::Store,
    },
    utils::slugify,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AdminNav {
    Dashboard,
    Stores,
    Categories,
    Products,
    Orders,
    Settings,
    Docs,
}

#[derive(Debug, Default, Deserialize)]
struct AdminQuery {
    edit_store: Option<Uuid>,
    edit_category: Option<Uuid>,
    edit_product: Option<Uuid>,
    edit_attribute: Option<Uuid>,
    edit_feature: Option<Uuid>,
}

#[derive(Debug, Default, Clone)]
struct DashboardSelections {
    store: Option<Store>,
    category: Option<Category>,
    product: Option<Product>,
    attribute: Option<ProductAttribute>,
    feature: Option<ProductFeature>,
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

pub fn router() -> Router {
    Router::new()
        .route("/", get(dashboard_page))
        .route("/admin", get(dashboard_page))
        .route("/admin/docs", get(api_docs))
        // Stores
        .route("/admin/stores", get(stores_page).post(create_store))
        .route("/admin/stores/:id/update", post(update_store))
        .route("/admin/stores/:id/delete", post(delete_store))
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
        .route("/admin/brands/:id/delete", post(delete_brand))
        // Orders
        .route("/admin/orders", get(orders_page))
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
    Query(params): Query<AdminQuery>,
    Extension(pool): Extension<PgPool>,
) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await;
    let categories = match category_repository::list_categories(&pool).await {
        Ok(records) => Ok(records),
        Err(err) if is_missing_relation(&err) => Ok(vec![]),
        Err(err) => Err(err),
    };
    let products = product_repository::list_products(&pool).await;
    let attributes = match product_attribute_repository::list_attributes(&pool).await {
        Ok(records) => Ok(records),
        Err(err) if is_missing_relation(&err) => Ok(vec![]),
        Err(err) => Err(err),
    };
    let features = match product_feature_repository::list_features(&pool).await {
        Ok(records) => Ok(records),
        Err(err) if is_missing_relation(&err) => Ok(vec![]),
        Err(err) => Err(err),
    };

    match (stores, categories, products, attributes, features) {
        (Ok(stores), Ok(categories), Ok(products), Ok(attributes), Ok(features)) => {
            let selections = build_selections(
                &stores,
                &categories,
                &products,
                &attributes,
                &features,
                params,
            );
            Html(render_dashboard(
                &stores,
                &categories,
                &products,
                &attributes,
                &features,
                &selections,
                None,
            ))
        }
        (stores_res, categories_res, products_res, attributes_res, features_res) => {
            let err = stores_res
                .err()
                .or_else(|| categories_res.err())
                .or_else(|| products_res.err())
                .or_else(|| attributes_res.err())
                .or_else(|| features_res.err());

            if let Some(err) = err.as_ref() {
                eprintln!("Admin dashboard data load failed: {err:?}");
            }

            Html(render_dashboard(
                &[],
                &[],
                &[],
                &[],
                &[],
                &DashboardSelections::default(),
                Some(
                    "Database connection failed. Ensure PostgreSQL is running and DATABASE_URL is set.".to_string(),
                ),
            ))
        }
    }
}

// ===== Individual Page Handlers =====

async fn stores_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Stores,
        "Stores",
        "Manage your store configurations",
        &render_stores_content(&stores),
    ))
}

async fn categories_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let stores = store_repository::list_stores(&pool).await.unwrap_or_default();
    let categories = category_repository::list_categories(&pool).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Categories,
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
        AdminNav::Products,
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
        AdminNav::Products,
        "Brands",
        "Manage product brands",
        &render_brands_content(&brands),
    ))
}

async fn orders_page(Extension(pool): Extension<PgPool>) -> Html<String> {
    let orders = checkout_repository::list_orders(&pool, None).await.unwrap_or_default();
    Html(render_page(
        AdminNav::Orders,
        "Orders",
        "View and manage customer orders",
        &render_orders_content(&orders),
    ))
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

fn build_selections(
    stores: &[Store],
    categories: &[Category],
    products: &[Product],
    attributes: &[ProductAttribute],
    features: &[ProductFeature],
    params: AdminQuery,
) -> DashboardSelections {
    DashboardSelections {
        store: params
            .edit_store
            .and_then(|id| stores.iter().find(|store| store.id == id).cloned()),
        category: params.edit_category.and_then(|id| {
            categories
                .iter()
                .find(|category| category.id == id)
                .cloned()
        }),
        product: params
            .edit_product
            .and_then(|id| products.iter().find(|product| product.id == id).cloned()),
        attribute: params.edit_attribute.and_then(|id| {
            attributes
                .iter()
                .find(|attribute| attribute.id == id)
                .cloned()
        }),
        feature: params
            .edit_feature
            .and_then(|id| features.iter().find(|feature| feature.id == id).cloned()),
    }
}

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

fn render_dashboard(
    stores: &[Store],
    categories: &[Category],
    products: &[Product],
    attributes: &[ProductAttribute],
    features: &[ProductFeature],
    selections: &DashboardSelections,
    error_message: Option<String>,
) -> String {
    let store_lookup: HashMap<Uuid, &Store> =
        stores.iter().map(|store| (store.id, store)).collect();
    let category_lookup: HashMap<Uuid, &Category> =
        categories.iter().map(|cat| (cat.id, cat)).collect();

    let attribute_lookup: HashMap<Uuid, Vec<&ProductAttribute>> =
        attributes
            .iter()
            .fold(HashMap::new(), |mut acc, attribute| {
                acc.entry(attribute.product_id)
                    .or_insert_with(Vec::new)
                    .push(attribute);
                acc
            });

    let feature_lookup: HashMap<Uuid, Vec<&ProductFeature>> =
        features.iter().fold(HashMap::new(), |mut acc, feature| {
            acc.entry(feature.product_id)
                .or_insert_with(Vec::new)
                .push(feature);
            acc
        });

    let store_rows = if stores.is_empty() {
        "<tr><td colspan=\"5\">No stores created yet.</td></tr>".to_string()
    } else {
        stores
            .iter()
            .map(|store| {
                format!(
                    "<tr>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td class=\"actions\">\
                            <a class=\"ghost\" href=\"/admin?edit_store={}#store-form\">Edit</a>\
                            <form method=\"post\" action=\"/admin/stores/{}/delete\" onsubmit=\"return confirm('Delete store and nested catalog?');\">\
                                <button class=\"danger\" type=\"submit\">Delete</button>\
                            </form>\
                        </td>\
                    </tr>",
                    html_escape(&store.name),
                    store
                        .domain
                        .as_deref()
                        .map(html_escape)
                        .unwrap_or_else(|| "—".to_string()),
                    html_escape(&store.currency),
                    store.created_at.format("%Y-%m-%d %H:%M"),
                    store.id,
                    store.id
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let category_rows = if categories.is_empty() {
        "<tr><td colspan=\"5\">Create your first category.</td></tr>".to_string()
    } else {
        categories
            .iter()
            .map(|category| {
                let store_label = store_lookup
                    .get(&category.store_id)
                    .map(|store| store.name.as_str())
                    .unwrap_or("Unknown store");

                format!(
                    "<tr>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td class=\"actions\">\
                            <a class=\"ghost\" href=\"/admin?edit_category={}#category-form\">Edit</a>\
                            <form method=\"post\" action=\"/admin/categories/{}/delete\" onsubmit=\"return confirm('Delete category? Products keep their data.');\">\
                                <button class=\"danger\" type=\"submit\">Delete</button>\
                            </form>\
                        </td>\
                    </tr>",
                    html_escape(&category.name),
                    html_escape(store_label),
                    html_escape(&category.slug),
                    category
                        .description
                        .as_deref()
                        .map(html_escape)
                        .unwrap_or_else(|| "—".to_string()),
                    category.id,
                    category.id
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let product_rows = if products.is_empty() {
        "<tr><td colspan=\"7\">No products created yet.</td></tr>".to_string()
    } else {
        products
            .iter()
            .map(|product| {
                let store_label = store_lookup
                    .get(&product.store_id)
                    .map(|store| store.name.as_str())
                    .unwrap_or("Unknown store");

                let category_label = product
                    .category_id
                    .and_then(|id| category_lookup.get(&id))
                    .map(|category| category.name.as_str())
                    .unwrap_or("—");

                let attribute_html = attribute_lookup
                    .get(&product.id)
                    .map(|items| {
                        items
                            .iter()
                            .map(|item| {
                                format!(
                                    "<div class=\"pill\">\
                                        <span><strong>{}</strong>: {}</span>\
                                        <div class=\"pill-actions\">\
                                            <a class=\"ghost\" href=\"/admin?edit_attribute={}#attribute-form\">Edit</a>\
                                            <form method=\"post\" action=\"/admin/product-attributes/{}/delete\" onsubmit=\"return confirm('Delete attribute?');\">\
                                                <button class=\"danger ghost\" type=\"submit\">✕</button>\
                                            </form>\
                                        </div>\
                                    </div>",
                                    html_escape(&item.name),
                                    html_escape(&item.value),
                                    item.id,
                                    item.id
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_else(|| "—".to_string());

                let feature_html = feature_lookup
                    .get(&product.id)
                    .map(|items| {
                        items
                            .iter()
                            .map(|item| {
                                format!(
                                    "<div class=\"pill\">\
                                        <span><strong>{}</strong>: {}</span>\
                                        <div class=\"pill-actions\">\
                                            <a class=\"ghost\" href=\"/admin?edit_feature={}#feature-form\">Edit</a>\
                                            <form method=\"post\" action=\"/admin/product-features/{}/delete\" onsubmit=\"return confirm('Delete feature?');\">\
                                                <button class=\"danger ghost\" type=\"submit\">✕</button>\
                                            </form>\
                                        </div>\
                                    </div>",
                                    html_escape(&item.label),
                                    html_escape(&item.value),
                                    item.id,
                                    item.id
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_else(|| "—".to_string());

                format!(
                    "<tr>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td>{}</td>\
                        <td class=\"actions\">\
                            <a class=\"ghost\" href=\"/admin?edit_product={}#product-form\">Edit</a>\
                            <form method=\"post\" action=\"/admin/products/{}/delete\" onsubmit=\"return confirm('Delete product?');\">\
                                <button class=\"danger\" type=\"submit\">Delete</button>\
                            </form>\
                        </td>\
                    </tr>",
                    html_escape(&product.title),
                    html_escape(store_label),
                    html_escape(category_label),
                    attribute_html,
                    feature_html,
                    if product.is_active { "Active" } else { "Archived" },
                    product.id,
                    product.id
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let store_form_state = selections.store.as_ref();
    let store_form_action = store_form_state
        .map(|store| format!("/admin/stores/{}/update", store.id))
        .unwrap_or_else(|| "/admin/stores".to_string());
    let store_form_title = store_form_state
        .map(|_| "Edit Store")
        .unwrap_or("Create Store");
    let store_form_button = store_form_state
        .map(|_| "Save Store")
        .unwrap_or("Create Store");
    let store_cancel_link = store_form_state
        .map(|_| "<a class=\"ghost\" href=\"/admin#store-form\">Cancel edit</a>".to_string());

    let category_form_state = selections.category.as_ref();
    let category_form_action = category_form_state
        .map(|category| format!("/admin/categories/{}/update", category.id))
        .unwrap_or_else(|| "/admin/categories".to_string());
    let category_form_title = category_form_state
        .map(|_| "Edit Category")
        .unwrap_or("Create Category");
    let category_form_button = category_form_state
        .map(|_| "Save Category")
        .unwrap_or("Create Category");
    let category_cancel_link = category_form_state
        .map(|_| "<a class=\"ghost\" href=\"/admin#category-form\">Cancel edit</a>".to_string());

    let product_form_state = selections.product.as_ref();
    let product_form_action = product_form_state
        .map(|product| format!("/admin/products/{}/update", product.id))
        .unwrap_or_else(|| "/admin/products".to_string());
    let product_form_title = product_form_state
        .map(|_| "Edit Product")
        .unwrap_or("Create Product");
    let product_form_button = product_form_state
        .map(|_| "Save Product")
        .unwrap_or("Create Product");
    let product_cancel_link = product_form_state
        .map(|_| "<a class=\"ghost\" href=\"/admin#product-form\">Cancel edit</a>".to_string());

    let attribute_form_state = selections.attribute.as_ref();
    let attribute_form_action = attribute_form_state
        .map(|attribute| format!("/admin/product-attributes/{}/update", attribute.id))
        .unwrap_or_else(|| "/admin/product-attributes".to_string());
    let attribute_form_title = attribute_form_state
        .map(|_| "Edit Attribute")
        .unwrap_or("Add Attribute");
    let attribute_form_button = attribute_form_state
        .map(|_| "Save Attribute")
        .unwrap_or("Add Attribute");
    let attribute_cancel_link = attribute_form_state
        .map(|_| "<a class=\"ghost\" href=\"/admin#attribute-form\">Cancel edit</a>".to_string());

    let feature_form_state = selections.feature.as_ref();
    let feature_form_action = feature_form_state
        .map(|feature| format!("/admin/product-features/{}/update", feature.id))
        .unwrap_or_else(|| "/admin/product-features".to_string());
    let feature_form_title = feature_form_state
        .map(|_| "Edit Feature")
        .unwrap_or("Add Feature");
    let feature_form_button = feature_form_state
        .map(|_| "Save Feature")
        .unwrap_or("Add Feature");
    let feature_cancel_link = feature_form_state
        .map(|_| "<a class=\"ghost\" href=\"/admin#feature-form\">Cancel edit</a>".to_string());

    let store_select_options = build_store_options(
        stores,
        selections
            .category
            .as_ref()
            .map(|category| category.store_id),
    );
    let store_option_for_product =
        build_store_options(stores, product_form_state.map(|p| p.store_id));
    let category_select_options =
        build_category_options(categories, product_form_state.and_then(|p| p.category_id));
    let attribute_product_options = build_product_options(
        products,
        attribute_form_state.map(|attribute| attribute.product_id),
    );
    let feature_product_options = build_product_options(
        products,
        feature_form_state.map(|feature| feature.product_id),
    );

    let attribute_form_values = attribute_form_state.map(|attribute| {
        (
            attribute.name.clone(),
            attribute.value.clone(),
            attribute.position,
            attribute.product_id,
        )
    });
    let feature_form_values = feature_form_state.map(|feature| {
        (
            feature.label.clone(),
            feature.value.clone(),
            feature.product_id,
        )
    });

    let product_attribute_text = product_form_state
        .and_then(|product| attribute_lookup.get(&product.id))
        .map(|items| {
            items
                .iter()
                .map(|item| format!("{}: {}", item.name, item.value))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let product_feature_text = product_form_state
        .and_then(|product| feature_lookup.get(&product.id))
        .map(|items| {
            items
                .iter()
                .map(|item| format!("{}: {}", item.label, item.value))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    let error_banner = error_message
        .map(|msg| format!("<div class=\"banner error\">{}</div>", html_escape(&msg)))
        .unwrap_or_default();

    format!(
        r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>RustCommerce Admin</title>
    <style>
        :root {{
            font-family: 'Inter', system-ui, sans-serif;
            --bg: #0f172a;
            --panel: #ffffff;
            --accent: #2563eb;
            --muted: #64748b;
        }}
        * {{ box-sizing: border-box; }}
        body {{
            margin: 0;
            min-height: 100vh;
            background: #f1f5f9;
            color: #0f172a;
        }}
        a {{ text-decoration: none; color: inherit; }}
        .app-shell {{
            display: grid;
            grid-template-columns: 260px 1fr;
            min-height: 100vh;
        }}
        aside {{
            background: var(--bg);
            color: #e2e8f0;
            padding: 2rem 1.25rem;
            display: flex;
            flex-direction: column;
            gap: 1.5rem;
            position: sticky;
            top: 0;
            height: 100vh;
        }}
        .logo {{
            font-weight: 700;
            font-size: 1.1rem;
            letter-spacing: 0.05em;
        }}
        .nav {{
            display: flex;
            flex-direction: column;
            gap: 0.4rem;
        }}
        .nav a {{
            padding: 0.55rem 0.8rem;
            border-radius: 10px;
            color: #cbd5f5;
            font-weight: 500;
        }}
        .nav a:hover {{
            background: rgba(255, 255, 255, 0.12);
            color: #fff;
        }}
        main {{
            padding: 2rem clamp(1rem, 4vw, 3rem);
            display: flex;
            flex-direction: column;
            gap: 2rem;
        }}
        header.topbar {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 1rem;
        }}
        .topbar h1 {{ margin: 0; font-size: clamp(1.5rem, 3vw, 2rem); }}
        .topbar p {{ margin: 0; color: var(--muted); }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 1rem;
        }}
        .stat-card {{
            background: var(--panel);
            border-radius: 18px;
            padding: 1.2rem;
            box-shadow: 0 20px 60px rgba(15, 23, 42, 0.08);
        }}
        .stat-card h3 {{
            margin: 0;
            color: var(--muted);
            font-size: 0.8rem;
            text-transform: uppercase;
            letter-spacing: 0.08em;
        }}
        .stat-card p {{ margin: 0.45rem 0 0; font-size: 2rem; font-weight: 700; }}
        .panel-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
            gap: 1.2rem;
        }}
        .panel {{
            background: var(--panel);
            border-radius: 18px;
            padding: 1.5rem;
            box-shadow: 0 18px 50px rgba(15, 23, 42, 0.08);
        }}
        .panel h2 {{ margin-top: 0; font-size: 1rem; }}
        form {{ display: flex; flex-direction: column; gap: 0.8rem; }}
        label {{
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--muted);
            display: flex;
            flex-direction: column;
            gap: 0.4rem;
        }}
        input, select, textarea {{
            padding: 0.7rem 0.85rem;
            border-radius: 12px;
            border: 1px solid #d7dce5;
            font-size: 1rem;
            width: 100%;
        }}
        button {{
            background: var(--accent);
            color: white;
            border: none;
            padding: 0.75rem 1rem;
            border-radius: 999px;
            cursor: pointer;
            font-weight: 600;
            transition: transform 0.1s ease;
        }}
        button:hover {{ transform: translateY(-2px); }}
        button.danger {{ background: #dc2626; }}
        button.ghost {{
            background: transparent;
            border: 1px solid rgba(148, 163, 184, 0.5);
            color: var(--muted);
            padding: 0.35rem 0.9rem;
            border-radius: 999px;
        }}
        button:disabled {{ background: #94a3b8; cursor: not-allowed; }}
        .banner {{ padding: 0.9rem 1.2rem; border-radius: 16px; font-weight: 600; }}
        .banner.error {{ background: #fee2e2; color: #991b1b; border: 1px solid #fecaca; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 0.65rem; text-align: left; border-bottom: 1px solid #e2e8f0; vertical-align: top; }}
        th {{ font-size: 0.75rem; text-transform: uppercase; color: var(--muted); letter-spacing: 0.08em; }}
        .actions {{ display: flex; gap: 0.45rem; flex-wrap: wrap; }}
        .actions form {{ margin: 0; }}
        .pill {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 0.4rem;
            padding: 0.35rem 0.6rem;
            background: #f8fafc;
            border-radius: 999px;
            margin-bottom: 0.35rem;
        }}
        .pill-actions {{ display: flex; gap: 0.25rem; }}
        .tables {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 1.25rem; }}
        .anchor {{ scroll-margin-top: 80px; }}
        @media (max-width: 960px) {{
            .app-shell {{ grid-template-columns: 1fr; }}
            aside {{
                position: relative;
                height: auto;
                flex-direction: row;
                flex-wrap: wrap;
                justify-content: space-between;
                gap: 0.8rem;
            }}
            .nav {{ flex-direction: row; flex-wrap: wrap; }}
            .nav a {{ flex: 1 1 45%; text-align: center; }}
        }}
        @media (max-width: 640px) {{
            main {{ padding: 1.25rem; }}
            .panel-grid, .tables {{ grid-template-columns: 1fr; }}
            header.topbar {{ flex-direction: column; align-items: flex-start; }}
        }}
    </style>
</head>
<body>
    <div class="app-shell">
        <aside>
            <div class="logo">RustCommerce CMS</div>
            <nav class="nav">
                <a href="#overview">Overview</a>
                <a href="#stores">Stores</a>
                <a href="#categories">Categories</a>
                <a href="#products">Products</a>
                <a href="/admin/docs">API Docs</a>
            </nav>
        </aside>
        <main>
            <header class="topbar" id="overview">
                <div class="topbar-text">
                    <h1>Commerce Control</h1>
                    <p>Track storefronts, catalog structure, and merchandising rules.</p>
                </div>
                <a class="ghost" href="/admin/docs">API Guide</a>
            </header>

            {error_banner}

            <section class="stats-grid">
                <div class="stat-card">
                    <h3>Stores</h3>
                    <p>{store_count}</p>
                </div>
                <div class="stat-card">
                    <h3>Products</h3>
                    <p>{product_count}</p>
                </div>
                <div class="stat-card">
                    <h3>Categories</h3>
                    <p>{category_count}</p>
                </div>
            </section>

            <section class="panel-grid" id="manage">
                <div class="panel anchor" id="store-form">
                    <h2>{store_form_title}</h2>
                    {store_cancel}
                    <form method="post" action="{store_form_action}">
                        <label>Name
                            <input type="text" name="name" value="{store_name}" required />
                        </label>
                        <label>Domain
                            <input type="text" name="domain" value="{store_domain}" placeholder="store.example.com" />
                        </label>
                        <label>Currency
                            <input type="text" name="currency" value="{store_currency}" maxlength="3" required />
                        </label>
                        <label>Timezone
                            <input type="text" name="timezone" value="{store_timezone}" required />
                        </label>
                        <button type="submit">{store_form_button}</button>
                    </form>
                </div>

                <div class="panel anchor" id="category-form">
                    <h2>{category_form_title}</h2>
                    {category_cancel}
                    <form method="post" action="{category_form_action}">
                        <label>Store
                            <select name="store_id" required>
                                {store_select_options}
                            </select>
                        </label>
                        <label>Name
                            <input type="text" name="name" value="{category_name}" required />
                        </label>
                        <label>Description
                            <textarea name="description" rows="2">{category_description}</textarea>
                        </label>
                        <button type="submit">{category_form_button}</button>
                    </form>
                </div>

                <div class="panel anchor" id="product-form">
                    <h2>{product_form_title}</h2>
                    {product_cancel}
                    <form method="post" action="{product_form_action}">
                        <label>Store
                            <select name="store_id" required>
                                {store_option_for_product}
                            </select>
                        </label>
                        <label>Category
                            <select name="category_id">
                                {category_select_options}
                            </select>
                        </label>
                        <label>Title
                            <input type="text" name="title" value="{product_title}" required />
                        </label>
                        <label>Description
                            <textarea name="description" rows="3">{product_description}</textarea>
                        </label>
                        <label class="checkbox">Active product?
                            <input type="checkbox" name="is_active" value="true" {product_active} />
                        </label>
                        <label>Attributes (Color: Black)
                            <textarea name="attributes" rows="3">{product_attribute_text}</textarea>
                        </label>
                        <label>Features (CPU: M3)
                            <textarea name="features" rows="3">{product_feature_text}</textarea>
                        </label>
                        <button type="submit">{product_form_button}</button>
                    </form>
                </div>

                <div class="panel anchor" id="attribute-form">
                    <h2>{attribute_form_title}</h2>
                    {attribute_cancel}
                    <form method="post" action="{attribute_form_action}">
                        <label>Product
                            <select name="product_id" required>
                                {attribute_product_options}
                            </select>
                        </label>
                        <label>Name
                            <input type="text" name="name" value="{attribute_name}" required />
                        </label>
                        <label>Value
                            <input type="text" name="value" value="{attribute_value}" required />
                        </label>
                        <label>Position
                            <input type="number" name="position" value="{attribute_position}" />
                        </label>
                        <button type="submit">{attribute_form_button}</button>
                    </form>
                </div>

                <div class="panel anchor" id="feature-form">
                    <h2>{feature_form_title}</h2>
                    {feature_cancel}
                    <form method="post" action="{feature_form_action}">
                        <label>Product
                            <select name="product_id" required>
                                {feature_product_options}
                            </select>
                        </label>
                        <label>Label
                            <input type="text" name="label" value="{feature_label}" required />
                        </label>
                        <label>Value
                            <input type="text" name="value" value="{feature_value}" required />
                        </label>
                        <button type="submit">{feature_form_button}</button>
                    </form>
                </div>
            </section>

            <section class="tables" id="collections">
                <div class="panel anchor" id="stores">
                    <h2>Stores</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Domain</th>
                                <th>Currency</th>
                                <th>Created</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {store_rows}
                        </tbody>
                    </table>
                </div>
                <div class="panel anchor" id="categories">
                    <h2>Categories</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Store</th>
                                <th>Slug</th>
                                <th>Description</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {category_rows}
                        </tbody>
                    </table>
                </div>
                <div class="panel anchor" id="products" style="grid-column: span 2;">
                    <h2>Products</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Title</th>
                                <th>Store</th>
                                <th>Category</th>
                                <th>Attributes</th>
                                <th>Features</th>
                                <th>Status</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {product_rows}
                        </tbody>
                    </table>
                </div>
            </section>
        </main>
    </div>
</body>
</html>
"###,
        error_banner = error_banner,
        store_count = stores.len(),
        product_count = products.len(),
        category_count = categories.len(),
        store_form_title = store_form_title,
        store_form_action = store_form_action,
        store_form_button = store_form_button,
        store_cancel = store_cancel_link.unwrap_or_default(),
        store_name = store_form_state
            .map(|s| html_escape(&s.name))
            .unwrap_or_default(),
        store_domain = store_form_state
            .and_then(|s| s.domain.clone())
            .map(|value| html_escape(&value))
            .unwrap_or_default(),
        store_currency = store_form_state
            .map(|s| html_escape(&s.currency))
            .unwrap_or_else(|| "USD".to_string()),
        store_timezone = store_form_state
            .map(|s| html_escape(&s.timezone))
            .unwrap_or_else(|| "UTC".to_string()),
        category_form_title = category_form_title,
        category_form_action = category_form_action,
        category_form_button = category_form_button,
        category_cancel = category_cancel_link.unwrap_or_default(),
        category_name = category_form_state
            .map(|c| html_escape(&c.name))
            .unwrap_or_default(),
        category_description = category_form_state
            .and_then(|c| c.description.clone())
            .map(|value| html_escape(&value))
            .unwrap_or_default(),
        product_form_title = product_form_title,
        product_form_action = product_form_action,
        product_form_button = product_form_button,
        product_cancel = product_cancel_link.unwrap_or_default(),
        product_title = product_form_state
            .map(|p| html_escape(&p.title))
            .unwrap_or_default(),
        product_description = product_form_state
            .and_then(|p| p.description.clone())
            .map(|value| html_escape(&value))
            .unwrap_or_default(),
        product_active = if product_form_state.map(|p| p.is_active).unwrap_or(true) {
            "checked"
        } else {
            ""
        },
        product_attribute_text = html_escape(&product_attribute_text),
        product_feature_text = html_escape(&product_feature_text),
        attribute_form_title = attribute_form_title,
        attribute_form_action = attribute_form_action,
        attribute_form_button = attribute_form_button,
        attribute_cancel = attribute_cancel_link.unwrap_or_default(),
        attribute_product_options = attribute_product_options,
        attribute_name = attribute_form_values
            .as_ref()
            .map(|(name, _, _, _)| html_escape(name))
            .unwrap_or_default(),
        attribute_value = attribute_form_values
            .as_ref()
            .map(|(_, value, _, _)| html_escape(value))
            .unwrap_or_default(),
        attribute_position = attribute_form_values
            .map(|(_, _, position, _)| position.to_string())
            .unwrap_or_else(|| "0".to_string()),
        feature_form_title = feature_form_title,
        feature_form_action = feature_form_action,
        feature_form_button = feature_form_button,
        feature_cancel = feature_cancel_link.unwrap_or_default(),
        feature_product_options = feature_product_options,
        feature_label = feature_form_values
            .as_ref()
            .map(|(label, _, _)| html_escape(label))
            .unwrap_or_default(),
        feature_value = feature_form_values
            .as_ref()
            .map(|(_, value, _)| html_escape(value))
            .unwrap_or_default(),
        store_select_options = store_select_options,
        store_option_for_product = store_option_for_product,
        category_select_options = category_select_options,
        store_rows = store_rows,
        category_rows = category_rows,
        product_rows = product_rows,
    )
}

fn build_store_options(stores: &[Store], selected: Option<Uuid>) -> String {
    if stores.is_empty() {
        "<option value=\"\">Create a store first</option>".to_string()
    } else {
        stores
            .iter()
            .map(|store| {
                let selected_attr = if Some(store.id) == selected {
                    "selected"
                } else {
                    ""
                };
                format!(
                    "<option value=\"{}\" {}>{} ({})</option>",
                    store.id,
                    selected_attr,
                    html_escape(&store.name),
                    html_escape(&store.currency)
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

fn build_category_options(categories: &[Category], selected: Option<Uuid>) -> String {
    let mut options = vec!["<option value=\"\">No category</option>".to_string()];
    options.extend(categories.iter().map(|category| {
        let selected_attr = if Some(category.id) == selected {
            "selected"
        } else {
            ""
        };
        format!(
            "<option value=\"{}\" {}>{}</option>",
            category.id,
            selected_attr,
            html_escape(&category.name)
        )
    }));
    options.join("")
}

fn build_product_options(products: &[Product], selected: Option<Uuid>) -> String {
    if products.is_empty() {
        "<option value=\"\">Create a product first</option>".to_string()
    } else {
        products
            .iter()
            .map(|product| {
                let selected_attr = if Some(product.id) == selected {
                    "selected"
                } else {
                    ""
                };
                format!(
                    "<option value=\"{}\" {}>{}</option>",
                    product.id,
                    selected_attr,
                    html_escape(&product.title)
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

fn optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn parse_key_value_lines(input: Option<String>) -> Vec<(String, String)> {
    input
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            if let Some(idx) = trimmed.find(|c| c == ':' || c == '=') {
                let (key, value) = trimmed.split_at(idx);
                Some((key.trim().to_owned(), value[1..].trim().to_owned()))
            } else {
                Some((trimmed.to_owned(), String::new()))
            }
        })
        .collect()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
}

fn render_error(message: &str) -> String {
    format!(
        r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>RustCommerce Admin Error</title>
    <style>
        body {{
            font-family: system-ui, sans-serif;
            background: #0f172a;
            color: #f8fafc;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }}
        .box {{
            background: #1e293b;
            padding: 2rem;
            border-radius: 1rem;
            width: min(400px, 90%);
            text-align: center;
        }}
        a {{
            color: #38bdf8;
        }}
    </style>
</head>
<body>
    <div class="box">
        <h1>Something went wrong</h1>
        <p>{}</p>
        <p><a href="/admin">Return to dashboard</a></p>
    </div>
</body>
</html>
"###,
        html_escape(message)
    )
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
                "<tr><td><span class=\"tag\">{}</span></td><td><code>{}</code></td><td>{}</td></tr>",
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
            <div class="panel">
                <h2>Bootstrapping</h2>
                <p>1. Create a store via <code>POST /stores</code><br />
                   2. Add categories (optional) then create products<br />
                   3. Attach attributes &amp; features through the admin UI or JSON payloads.</p>
            </div>
            <div class="panel">
                <h2>Authentication</h2>
                <p>No auth yet. The server expects trusted traffic only. Wire up API keys or JWT when deploying.</p>
            </div>
        </div>

        <div class="panel">
            <h2>Endpoints</h2>
            <table>
                <thead>
                    <tr>
                        <th>Method</th>
                        <th>Path</th>
                        <th>Description</th>
                    </tr>
                </thead>
                <tbody>
                    {endpoint_rows}
                </tbody>
            </table>
        </div>

        <div class="panel">
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
    <style>
        :root {{
            font-family: 'Inter', system-ui, sans-serif;
            --bg: #0f172a;
            --panel: #ffffff;
            --accent: #2563eb;
            --muted: #64748b;
        }}
        * {{ box-sizing: border-box; }}
        body {{
            margin: 0;
            min-height: 100vh;
            background: #f1f5f9;
            color: #0f172a;
        }}
        a {{ text-decoration: none; color: inherit; }}
        .app-shell {{
            display: grid;
            grid-template-columns: 260px 1fr;
            min-height: 100vh;
        }}
        aside {{
            background: var(--bg);
            color: #e2e8f0;
            padding: 2rem 1.25rem;
            display: flex;
            flex-direction: column;
            gap: 1.5rem;
            position: sticky;
            top: 0;
            height: 100vh;
            overflow-y: auto;
        }}
        .logo {{
            font-weight: 700;
            font-size: 1.1rem;
            letter-spacing: 0.05em;
        }}
        .nav {{
            display: flex;
            flex-direction: column;
            gap: 0.3rem;
        }}
        .nav-section {{
            margin-top: 1rem;
        }}
        .nav-section-title {{
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            color: #64748b;
            margin-bottom: 0.5rem;
            padding: 0 0.8rem;
        }}
        .nav a {{
            padding: 0.6rem 0.8rem;
            border-radius: 10px;
            color: #cbd5e1;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}
        .nav a:hover {{
            background: rgba(255, 255, 255, 0.12);
            color: #fff;
        }}
        .nav a.active {{
            background: rgba(37, 99, 235, 0.2);
            color: #60a5fa;
        }}
        .material-icons {{
            font-size: 20px;
        }}
        main {{
            padding: 2rem clamp(1rem, 4vw, 3rem);
            max-width: 1400px;
        }}
        header.topbar {{
            margin-bottom: 2rem;
        }}
        .topbar h1 {{ margin: 0; font-size: clamp(1.5rem, 3vw, 2rem); }}
        .topbar p {{ margin: 0.5rem 0 0; color: var(--muted); }}
        .panel {{
            background: var(--panel);
            border-radius: 18px;
            padding: 1.5rem;
            box-shadow: 0 20px 60px rgba(15, 23, 42, 0.08);
            margin-bottom: 1.5rem;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        th {{
            text-align: left;
            padding: 0.75rem;
            border-bottom: 2px solid #e2e8f0;
            font-weight: 600;
            font-size: 0.875rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--muted);
        }}
        td {{
            padding: 0.75rem;
            border-bottom: 1px solid #f1f5f9;
        }}
        tr:hover {{
            background: #f8fafc;
        }}
        .actions {{
            display: flex;
            gap: 0.5rem;
        }}
        button, .btn {{
            padding: 0.5rem 1rem;
            border-radius: 8px;
            border: none;
            font-weight: 500;
            cursor: pointer;
            font-size: 0.875rem;
        }}
        .btn-primary {{
            background: var(--accent);
            color: white;
        }}
        .btn-danger {{
            background: #dc2626;
            color: white;
        }}
        .ghost {{
            background: transparent;
            border: 1px solid #e2e8f0;
            color: var(--muted);
        }}
        .badge {{
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
        }}
        .badge-green {{ background: #dcfce7; color: #16a34a; }}
        .badge-orange {{ background: #fed7aa; color: #ea580c; }}
        .badge-blue {{ background: #dbeafe; color: #2563eb; }}
        .badge-gray {{ background: #f1f5f9; color: #64748b; }}
        form {{ display: inline; }}
    </style>
</head>
<body>
    <div class="app-shell">
        <aside>
            <div class="logo">RustCommerce CMS</div>
            {nav}
        </aside>
        <main>
            <header class="topbar">
                <h1>{title}</h1>
                <p>{subtitle}</p>
            </header>
            {content}
        </main>
    </div>
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
        (AdminNav::Stores, "store", "Stores", "/admin/stores"),
        (AdminNav::Categories, "category", "Categories", "/admin/categories"),
        (AdminNav::Products, "inventory_2", "Products", "/admin/products"),
        (AdminNav::Products, "sell", "Brands", "/admin/brands"),
    ];

    let order_items = [
        ("receipt_long", "Orders", "/admin/orders"),
        ("description", "Invoices", "/admin/invoices"),
        ("receipt", "Credit Slips", "/admin/credit-slips"),
        ("local_shipping", "Delivery Slips", "/admin/delivery-slips"),
        ("shopping_cart", "Shopping Carts", "/admin/shopping-carts"),
        ("card_giftcard", "Gift Cards", "/admin/gift-cards"),
    ];

    let catalog_links = nav_items
        .iter()
        .map(|(nav, icon, label, href)| {
            let active_class = if *nav == active { " active" } else { "" };
            format!(
                r#"<a href="{}" class="{}"><span class="material-icons">{}</span>{}</a>"#,
                href, active_class, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                ");

    let order_links = order_items
        .iter()
        .map(|(icon, label, href)| {
            let active_class = if active == AdminNav::Orders && href.contains(label.split_whitespace().next().unwrap_or("").to_lowercase().as_str()) {
                " active"
            } else {
                ""
            };
            format!(
                r#"<a href="{}" class="{}"><span class="material-icons">{}</span>{}</a>"#,
                href, active_class, icon, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n                ");

    format!(
        r#"<nav class="nav">
                {}
                <div class="nav-section">
                    <div class="nav-section-title">Orders</div>
                    {}
                </div>
                <div class="nav-section">
                    <a href="/admin/docs"><span class="material-icons">api</span>API Docs</a>
                </div>
            </nav>"#,
        catalog_links, order_links
    )
}

fn render_stores_content(stores: &[Store]) -> String {
    let rows = if stores.is_empty() {
        "<tr><td colspan=\"5\">No stores created yet.</td></tr>".to_string()
    } else {
        stores
            .iter()
            .map(|store| {
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td class="actions">
                            <form method="post" action="/admin/stores/{}/delete" onsubmit="return confirm('Delete store?');">
                                <button class="btn-danger" type="submit">Delete</button>
                            </form>
                        </td>
                    </tr>"#,
                    html_escape(&store.name),
                    store.domain.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    html_escape(&store.currency),
                    store.created_at.format("%Y-%m-%d"),
                    store.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Domain</th>
                        <th>Currency</th>
                        <th>Created</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_categories_content(stores: &[Store], categories: &[Category]) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    
    let rows = if categories.is_empty() {
        "<tr><td colspan=\"5\">No categories created yet.</td></tr>".to_string()
    } else {
        categories
            .iter()
            .map(|cat| {
                let store_name = store_lookup.get(&cat.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td class="actions">
                            <form method="post" action="/admin/categories/{}/delete" onsubmit="return confirm('Delete category?');">
                                <button class="btn-danger" type="submit">Delete</button>
                            </form>
                        </td>
                    </tr>"#,
                    html_escape(&cat.name),
                    html_escape(store_name),
                    cat.slug,
                    if cat.is_active { "Active" } else { "Inactive" },
                    cat.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Store</th>
                        <th>Slug</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_products_content(stores: &[Store], categories: &[Category], products: &[Product]) -> String {
    let store_lookup: HashMap<Uuid, &Store> = stores.iter().map(|s| (s.id, s)).collect();
    let category_lookup: HashMap<Uuid, &Category> = categories.iter().map(|c| (c.id, c)).collect();
    
    let rows = if products.is_empty() {
        "<tr><td colspan=\"6\">No products created yet.</td></tr>".to_string()
    } else {
        products
            .iter()
            .map(|product| {
                let store_name = store_lookup.get(&product.store_id).map(|s| s.name.as_str()).unwrap_or("Unknown");
                let category_name = product.category_id
                    .and_then(|id| category_lookup.get(&id))
                    .map(|c| c.name.as_str())
                    .unwrap_or("—");
                
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td class="actions">
                            <form method="post" action="/admin/products/{}/delete" onsubmit="return confirm('Delete product?');">
                                <button class="btn-danger" type="submit">Delete</button>
                            </form>
                        </td>
                    </tr>"#,
                    html_escape(&product.title),
                    html_escape(store_name),
                    html_escape(category_name),
                    product.slug,
                    if product.is_active { "Active" } else { "Inactive" },
                    product.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Product</th>
                        <th>Store</th>
                        <th>Category</th>
                        <th>Slug</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_brands_content(brands: &[Brand]) -> String {
    let rows = if brands.is_empty() {
        "<tr><td colspan=\"4\">No brands created yet.</td></tr>".to_string()
    } else {
        brands
            .iter()
            .map(|brand| {
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td class="actions">
                            <form method="post" action="/admin/brands/{}/delete" onsubmit="return confirm('Delete brand?');">
                                <button class="btn-danger" type="submit">Delete</button>
                            </form>
                        </td>
                    </tr>"#,
                    html_escape(&brand.name),
                    brand.slug,
                    brand.description.as_deref().map(html_escape).unwrap_or_else(|| "—".to_string()),
                    brand.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Slug</th>
                        <th>Description</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_orders_content(orders: &[Order]) -> String {
    let rows = if orders.is_empty() {
        "<tr><td colspan=\"6\">No orders yet.</td></tr>".to_string()
    } else {
        orders
            .iter()
            .map(|order| {
                let status_badge = match order.status.as_str() {
                    "pending" => r#"<span class="badge badge-orange">Pending</span>"#,
                    "confirmed" | "processing" => r#"<span class="badge badge-blue">Processing</span>"#,
                    "delivered" => r#"<span class="badge badge-green">Delivered</span>"#,
                    _ => r#"<span class="badge badge-gray">Unknown</span>"#,
                };
                
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>${}</td>
                        <td>{}</td>
                    </tr>"#,
                    html_escape(&order.order_number),
                    html_escape(&order.customer_email),
                    html_escape(&order.customer_name),
                    order.created_at.format("%Y-%m-%d %H:%M"),
                    order.total_amount,
                    status_badge
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Order #</th>
                        <th>Customer Email</th>
                        <th>Customer Name</th>
                        <th>Date</th>
                        <th>Total</th>
                        <th>Status</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}

fn render_invoices_content(invoices: &[Invoice]) -> String {
    let rows = if invoices.is_empty() {
        "<tr><td colspan=\"5\">No invoices yet.</td></tr>".to_string()
    } else {
        invoices
            .iter()
            .map(|inv| {
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>${}</td>
                        <td>{}</td>
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
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Invoice #</th>
                        <th>Customer</th>
                        <th>Date</th>
                        <th>Amount</th>
                        <th>Status</th>
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
        "<tr><td colspan=\"4\">No credit slips yet.</td></tr>".to_string()
    } else {
        slips
            .iter()
            .map(|slip| {
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>${}</td>
                        <td>{}</td>
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
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Slip #</th>
                        <th>Date</th>
                        <th>Amount</th>
                        <th>Status</th>
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
        "<tr><td colspan=\"5\">No delivery slips yet.</td></tr>".to_string()
    } else {
        slips
            .iter()
            .map(|slip| {
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
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
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Slip #</th>
                        <th>Tracking</th>
                        <th>Carrier</th>
                        <th>Date</th>
                        <th>Status</th>
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
        "<tr><td colspan=\"5\">No shopping carts yet.</td></tr>".to_string()
    } else {
        carts
            .iter()
            .map(|cart| {
                format!(
                    r#"<tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>${}</td>
                        <td>{}</td>
                        <td>{}</td>
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
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Cart ID</th>
                        <th>Customer</th>
                        <th>Total</th>
                        <th>Created</th>
                        <th>Status</th>
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
        "<tr><td colspan=\"5\">No gift cards yet.</td></tr>".to_string()
    } else {
        cards
            .iter()
            .map(|card| {
                format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td>${}</td>
                        <td>${}</td>
                        <td>{}</td>
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
        r#"<div class="panel">
            <table>
                <thead>
                    <tr>
                        <th>Code</th>
                        <th>Recipient</th>
                        <th>Initial Amount</th>
                        <th>Balance</th>
                        <th>Status</th>
                    </tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        rows
    )
}
