use stargem_server::db::{create_pool, PostgresShipRepository, PostgresUserRepository};
use stargem_server::network::GameServer;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://stargem:stargem@localhost/stargem".into());

    let pool = create_pool(&database_url).await?;
    tracing::info!("Connected to database");

    let _user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let _ship_repo = Arc::new(PostgresShipRepository::new(pool.clone()));

    let server = GameServer::new("0.0.0.0:8080".into());
    let _session_manager = server.session_manager();

    tracing::info!("Starting Stargem server on 0.0.0.0:8080");
    server.start().await?;

    Ok(())
}
