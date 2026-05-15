use crate::combat::tick::{CombatTickLoop, TickSnapshot};
use quinn::{Endpoint, ServerConfig};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn serve(addr: &str, tick_rate: u64) -> Result<(), Box<dyn std::error::Error>> {
    let (snapshot_tx, _snapshot_rx) = mpsc::channel::<TickSnapshot>(256);
    let (input_tx, input_rx) = mpsc::channel::<(String, crate::combat::physics::ShipInput)>(1024);

    let mut tick_loop = CombatTickLoop::new(tick_rate, snapshot_tx, input_rx);
    tokio::spawn(async move {
        tick_loop.run().await;
    });

    let server_config = make_server_config()?;
    let endpoint = Endpoint::server(server_config, addr.parse()?)?;
    tracing::info!("QUIC server listening on {}", addr);

    while let Some(conn) = endpoint.accept().await {
        let input_tx = input_tx.clone();
        tokio::spawn(async move {
            match conn.await {
                Ok(connection) => {
                    tracing::info!("QUIC client connected: {}", connection.remote_address());
                    if let Ok(mut bi) = connection.accept_bi().await {
                        let mut buf = vec![0u8; 1024];
                        while let Ok(Some(_size)) = bi.1.read(&mut buf).await {
                            if let Ok(input) =
                                serde_json::from_slice::<crate::combat::physics::ShipInput>(&buf)
                            {
                                let _ = input_tx.try_send(("unknown".to_string(), input));
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("QUIC connection failed: {}", e);
                }
            }
        });
    }

    Ok(())
}

fn make_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let rcgen_cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert = rustls::Certificate(rcgen_cert.serialize_der()?);
    let key = rustls::PrivateKey(rcgen_cert.serialize_private_key_der().to_vec());

    let tls_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    Ok(ServerConfig::with_crypto(Arc::new(tls_config)))
}
