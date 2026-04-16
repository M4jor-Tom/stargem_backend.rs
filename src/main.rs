use stargem_server::api::GameService;
use stargem_server::db::{create_pool, PostgresHangarRepository, PostgresShipModelRepository, PostgresShipRepository, PostgresUserRepository};
use stargem_server::network::{GameServer, SessionManager, TlsConfig};
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

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let ship_repo = Arc::new(PostgresShipRepository::new(pool.clone()));
    let ship_model_repo = Arc::new(PostgresShipModelRepository::new(pool.clone()));
    let hangar_repo = Arc::new(PostgresHangarRepository::new(pool.clone()));
    let session_manager = Arc::new(SessionManager::new());

    let game_service = Arc::new(GameService::new(
        user_repo,
        ship_repo,
        ship_model_repo,
        hangar_repo,
        session_manager.clone(),
    ));

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let use_tls = std::env::var("USE_TLS").unwrap_or_else(|_| "false".into()) == "true";

    let server = if use_tls {
        let tls_cert = std::env::var("TLS_CERT").expect("TLS_CERT required when USE_TLS=true");
        let tls_key = std::env::var("TLS_KEY").expect("TLS_KEY required when USE_TLS=true");
        let tls_config = TlsConfig::from_pem_files(&tls_cert, &tls_key)?;
        let addr = if bind_addr.contains(':') { bind_addr } else { format!("{}:443", bind_addr) };
        tracing::info!("Starting Stargem server with TLS on {}", addr);
        GameServer::with_tls(addr, game_service, tls_config)
    } else {
        tracing::info!("Starting Stargem server on {}", bind_addr);
        GameServer::new(bind_addr, game_service)
    };

    server.start().await?;

    Ok(())
}
