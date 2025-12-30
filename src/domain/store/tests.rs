#[tokio::test]
async fn load_store_by_domain() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let pool = crate::infra::db::pool::create_pool(&url).await;

    let store = crate::infra::db::store::get_store_by_domain(
        &pool,
        "example.com"
    )
    .await
    .unwrap();

    assert_eq!(store.domain.as_deref(), Some("example.com"));
}
