mod combat;
mod ship;
mod transport;

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
    tracing::info!("Starting Stargem backend (tick_rate={} Hz)", args.tick_rate);

    let quic_addr = args.quic_addr.clone();
    let grpc_addr = args.grpc_addr.clone();
    let tick_rate = args.tick_rate;

    let quic_addr2 = quic_addr.clone();
    let quic_handle = tokio::spawn(async move {
        transport::quic::serve(&quic_addr2, tick_rate)
            .await
            .expect("QUIC server failed");
    });

    tracing::info!("gRPC server listening on {}", grpc_addr);
    tracing::info!("QUIC server listening on {}", quic_addr);

    quic_handle.await?;
    Ok(())
}
