use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let schema = include_str!("../sql/schema.sql");
    sqlx::raw_sql(schema).execute(pool).await?;
    tracing::info!("Database schema loaded");
    Ok(())
}

pub async fn run_seed(pool: &PgPool) -> Result<(), sqlx::Error> {
    let seed = include_str!("../sql/seed.sql");
    sqlx::raw_sql(seed).execute(pool).await?;
    tracing::info!("Database seed data loaded");
    Ok(())
}
