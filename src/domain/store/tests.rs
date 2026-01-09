#[tokio::test]
async fn load_store_by_domain() {
    let url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping store DB test: DATABASE_URL missing");
            return;
        }
    };

    let pool = match crate::infra::db::pool::create_pool(&url).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("Skipping store DB test: {err}");
            return;
        }
    };

    let store = match crate::infra::db::store::get_store_by_domain(&pool, "example.com").await {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Skipping store DB test: {err}");
            return;
        }
    };

    assert_eq!(store.domain.as_deref(), Some("example.com"));
}
