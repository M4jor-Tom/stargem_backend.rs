use std::sync::Arc;

use stargem_backend::database;
use stargem_backend::grpc;
use stargem_backend::transport;

use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "stargem-backend", about = "Stargem game backend server")]
struct Args {
    /// gRPC listen address
    #[arg(long, default_value = "0.0.0.0:50051")]
    grpc_addr: String,

    /// QUIC listen address
    #[arg(long, default_value = "0.0.0.0:50052")]
    quic_addr: String,

    /// Combat tick rate in Hz
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,

    /// Database connection URL
    #[arg(long)]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let grpc_addr = args.grpc_addr.clone();
    let quic_addr = args.quic_addr.clone();
    let tick_rate = args.tick_rate;

    tracing::info!("Starting Stargem backend (tick_rate={} Hz)", tick_rate);

    let pool = if let Some(ref url) = args.database_url {
        let p = database::init_pool(url).await?;
        database::run_schema(&p).await?;
        database::run_seed(&p).await?;
        tracing::info!("Database initialized");
        Some(p)
    } else {
        tracing::warn!("No --database-url provided; running without database");
        None
    };

    let grpc_state = Arc::new(grpc::AppState::new(pool));

    let grpc_handle = {
        let state = grpc_state.clone();
        let addr = grpc_addr.clone();
        tokio::spawn(async move {
            grpc::serve(&addr, state)
                .await
                .expect("gRPC server failed");
        })
    };

    let quic_addr2 = quic_addr.clone();
    let quic_handle = tokio::spawn(async move {
        transport::quic::serve(&quic_addr2, tick_rate)
            .await
            .expect("QUIC server failed");
    });

    tracing::info!("gRPC server listening on {}", grpc_addr);
    tracing::info!("QUIC server listening on {}", quic_addr);

    tokio::try_join!(grpc_handle, quic_handle)?;
    Ok(())
}
