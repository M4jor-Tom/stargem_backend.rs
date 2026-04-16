use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use once_cell::sync::Lazy;

static TEST_DB: Lazy<TestDatabase> = Lazy::new(|| {
    TestDatabase::new()
});

pub struct TestDatabase {
    connection_string: String,
}

impl TestDatabase {
    pub fn new() -> Self {
        let connection_string = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://stargem_test:stargem_test@localhost:5433/stargem_test".into());

        Self { connection_string }
    }

    pub async fn global() -> &'static Self {
        TEST_DB.get_or_init(|| Self::new())
    }

    pub async fn pool(&self) -> PgPool {
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&self.connection_string)
            .await
            .expect("Failed to create test database connection pool")
    }

    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

impl Default for TestDatabase {
    fn default() -> Self {
        Self::new()
    }
}
