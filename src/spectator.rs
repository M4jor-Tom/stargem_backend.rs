use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::proto_gen::grpc::spectator::{
    spectator_service_server::SpectatorService,
    ListMatchesRequest, ListMatchesResponse, MatchInfo,
    SubscribeRequest,
};
use crate::proto_gen::quic::combat::GameStateSnapshot;

pub struct MatchRegistry {
    matches: HashMap<Uuid, MatchEntry>,
}

struct MatchEntry {
    player_ids: Vec<String>,
    tick_number: u32,
    tx: broadcast::Sender<GameStateSnapshot>,
}

impl MatchRegistry {
    pub fn new() -> Self {
        Self { matches: HashMap::new() }
    }

    pub fn register(
        &mut self,
        match_id: Uuid,
        player_ids: Vec<String>,
        tick_number: u32,
        tx: broadcast::Sender<GameStateSnapshot>,
    ) {
        self.matches.insert(match_id, MatchEntry { player_ids, tick_number, tx });
    }

    pub fn list(&self) -> Vec<MatchInfo> {
        self.matches.iter().map(|(id, e)| MatchInfo {
            match_id: id.to_string(),
            player_ids: e.player_ids.clone(),
            tick_number: e.tick_number,
        }).collect()
    }

    pub fn subscribe(&self, id: &Uuid) -> Option<broadcast::Receiver<GameStateSnapshot>> {
        self.matches.get(id).map(|e| e.tx.subscribe())
    }
}

pub struct SpectatorHandler {
    pub registry: Arc<Mutex<MatchRegistry>>,
}

type SnapStream = Pin<Box<dyn Stream<Item = Result<GameStateSnapshot, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl SpectatorService for SpectatorHandler {
    type SubscribeStream = SnapStream;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let match_id_str = request.into_inner().match_id;
        let uuid = Uuid::parse_str(&match_id_str)
            .map_err(|_| Status::invalid_argument("match_id not a uuid"))?;

        let rx = self.registry.lock().await.subscribe(&uuid)
            .ok_or_else(|| Status::not_found("match not registered"))?;

        let stream = BroadcastStream::new(rx).map(|r| {
            r.map_err(|e| Status::data_loss(format!("spectator lagged: {e}")))
        });
        Ok(Response::new(Box::pin(stream)))
    }

    async fn list_matches(
        &self,
        _req: Request<ListMatchesRequest>,
    ) -> Result<Response<ListMatchesResponse>, Status> {
        let matches = self.registry.lock().await.list();
        Ok(Response::new(ListMatchesResponse { matches }))
    }
}