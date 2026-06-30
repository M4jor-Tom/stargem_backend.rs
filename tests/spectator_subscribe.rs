use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use uuid::Uuid;

use stargem_backend::combat::tick::snapshot_to_proto;
use stargem_backend::ship::stats::PlayerShipStats;
use stargem_backend::spectator::{MatchRegistry, SpectatorHandler};
use stargem_backend::proto_gen::grpc::spectator::spectator_service_server::SpectatorServiceServer;
use stargem_backend::proto_gen::grpc::spectator::spectator_service_client::SpectatorServiceClient;
use stargem_backend::proto_gen::grpc::spectator::{SubscribeRequest, ListMatchesRequest};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscribe_streams_snapshots() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let registry = Arc::new(Mutex::new(MatchRegistry::new()));
    let match_id = Uuid::from_u128(0xCAFE);
    let (tx, _) = broadcast::channel(64);
    registry.lock().await.register(
        match_id,
        vec!["p1".to_string()],
        0,
        tx.clone(),
    );

    let handler = SpectatorHandler { registry: registry.clone() };
    let svc = SpectatorServiceServer::new(handler);

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = SpectatorServiceClient::connect(format!("http://{}", addr))
        .await.unwrap();

    let resp = client.subscribe(SubscribeRequest {
        match_id: match_id.to_string(),
    }).await.unwrap();
    let mut stream = resp.into_inner();

    use stargem_backend::combat::tick::{TickSnapshot, PlayerState};
    use stargem_backend::combat::physics::{PhysicsState, ShipInput};
    let snap = TickSnapshot {
        tick_number: 7,
        players: vec![PlayerState {
            id: "p1".into(),
            physics: PhysicsState::new(),
            stats: PlayerShipStats {
                max_shield: 100.0, max_armor: 100.0, max_energy: 100.0,
                speed: 50.0, agility: 10.0,
                current_shield: 100.0, current_armor: 100.0, current_energy: 100.0,
            },
            shield_hp: 100.0, armor_hp: 100.0, energy: 100.0, heat_level: 0.0,
            input: ShipInput { throttle: 0.0, yaw: 0.0, pitch: 0.0, roll: 0.0 },
        }],
        damage_events: vec![],
    };
    tx.send(snapshot_to_proto(&snap)).unwrap();

    let received = tokio::time::timeout(std::time::Duration::from_secs(2), stream.message())
        .await.expect("timeout").unwrap().expect("stream ended");
    assert_eq!(received.tick_number, 7);
    assert_eq!(received.players.len(), 1);

    let lm = client.list_matches(ListMatchesRequest {}).await.unwrap().into_inner();
    assert_eq!(lm.matches.len(), 1);
    assert_eq!(lm.matches[0].player_ids.len(), 1);
}