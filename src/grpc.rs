#![allow(dead_code)]

use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::auth::{AuthProvider, MockAuthProvider};
use crate::game_mode::MatchManager;
use crate::proto_gen::grpc::auth::auth_service_server::AuthService;
use crate::proto_gen::grpc::auth::{LoginRequest, LoginResponse, ValidateSessionRequest, ValidateSessionResponse};
use crate::proto_gen::grpc::hangar::hangar_service_server::HangarService;
use crate::proto_gen::grpc::hangar::{AssignShipToSlotRequest, AssignShipToSlotResponse, ListHangarRequest, ListHangarResponse};
use crate::proto_gen::grpc::loadout::loadout_service_server::LoadoutService;
use crate::proto_gen::grpc::loadout::{EquipActiveModuleRequest, EquipLoadoutResponse, EquipMissileRequest, EquipPassiveModuleRequest, EquipWeaponRequest, Loadout};
use crate::proto_gen::grpc::match_history::match_history_service_server::MatchHistoryService;
use crate::proto_gen::grpc::match_history::{GetHistoryRequest, GetHistoryResponse, MatchRecord};
use crate::proto_gen::grpc::matchmaking::matchmaking_service_server::MatchmakingService;
use crate::proto_gen::grpc::matchmaking::{LeaveQueueRequest, LeaveQueueResponse, QueueForMatchRequest, QueueForMatchResponse, QueueState, QueueStatusRequest, QueueStatusResponse};
use crate::proto_gen::grpc::shop::shop_service_server::ShopService;
use crate::proto_gen::grpc::shop::{BuyShipRequest, BuyShipResponse, ListShipsRequest, ListShipsResponse, ShipModel};

pub struct AppState {
    pub auth_provider: Box<dyn AuthProvider>,
    pub match_manager: Arc<Mutex<MatchManager>>,
    pub pool: Option<sqlx::PgPool>,
}

impl AppState {
    pub fn new(pool: Option<sqlx::PgPool>) -> Self {
        Self {
            auth_provider: Box::new(MockAuthProvider::new()),
            match_manager: Arc::new(Mutex::new(MatchManager::new(4, 16))),
            pool,
        }
    }
}

pub struct AuthHandler {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl AuthService for AuthHandler {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let user_id = self.state
            .auth_provider
            .authenticate(&req.steam_auth_ticket)
            .await
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        Ok(Response::new(LoginResponse {
            session_token: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
        }))
    }

    async fn validate_session(&self, request: Request<ValidateSessionRequest>) -> Result<Response<ValidateSessionResponse>, Status> {
        let req = request.into_inner();
        match self.state.auth_provider.validate_session(&req.session_token).await {
            Ok(user_id) => Ok(Response::new(ValidateSessionResponse {
                valid: true,
                user_id: user_id.to_string(),
            })),
            Err(_) => Ok(Response::new(ValidateSessionResponse {
                valid: false,
                user_id: String::new(),
            })),
        }
    }
}

pub struct ShopHandler {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl ShopService for ShopHandler {
    async fn list_ships(&self, _request: Request<ListShipsRequest>) -> Result<Response<ListShipsResponse>, Status> {
        let ships = match &self.state.pool {
            Some(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, i32)>(
                    "SELECT id::text, size::text, role::text, price FROM ship_models"
                )
                .fetch_all(pool)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

                rows.into_iter().map(|(id, size, role, price)| ShipModel {
                    id,
                    name: String::new(),
                    size,
                    role,
                    price,
                }).collect()
            }
            None => vec![],
        };

        Ok(Response::new(ListShipsResponse { ships }))
    }

    async fn buy_ship(&self, request: Request<BuyShipRequest>) -> Result<Response<BuyShipResponse>, Status> {
        let req = request.into_inner();
        let pool = self.state.pool.as_ref().ok_or_else(|| Status::failed_precondition("database not available"))?;

        let ship_model_id = Uuid::parse_str(&req.ship_model_id).map_err(|_| Status::invalid_argument("invalid ship_model_id"))?;

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM ship_models WHERE id = $1)"
        )
        .bind(ship_model_id)
        .fetch_one(pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        if !exists {
            return Ok(Response::new(BuyShipResponse {
                success: false,
                player_ship_id: String::new(),
                error: "ship model not found".into(),
            }));
        }

        let player_ship_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO player_ships (id, user_id, ship_model_id) VALUES ($1, $2, $3)"
        )
        .bind(player_ship_id)
        .bind(Uuid::nil())
        .bind(ship_model_id)
        .execute(pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(BuyShipResponse {
            success: true,
            player_ship_id: player_ship_id.to_string(),
            error: String::new(),
        }))
    }
}

pub struct HangarHandler {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl HangarService for HangarHandler {
    async fn list_hangar(&self, _request: Request<ListHangarRequest>) -> Result<Response<ListHangarResponse>, Status> {
        Ok(Response::new(ListHangarResponse { slots: vec![] }))
    }

    async fn assign_ship_to_slot(&self, request: Request<AssignShipToSlotRequest>) -> Result<Response<AssignShipToSlotResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(AssignShipToSlotResponse {
            success: true,
            error: String::new(),
        }))
    }
}

pub struct LoadoutHandler {
    pub state: Arc<AppState>,
}

impl LoadoutHandler {
    fn empty_loadout() -> Option<Loadout> {
        Some(Loadout {
            passive_module_ids: vec![],
            active_module_ids: vec![],
            weapon_id: String::new(),
            missile_id: String::new(),
        })
    }
}

#[tonic::async_trait]
impl LoadoutService for LoadoutHandler {
    async fn equip_passive_module(&self, _request: Request<EquipPassiveModuleRequest>) -> Result<Response<EquipLoadoutResponse>, Status> {
        Ok(Response::new(EquipLoadoutResponse {
            success: true,
            loadout: Self::empty_loadout(),
            error: String::new(),
        }))
    }

    async fn equip_active_module(&self, _request: Request<EquipActiveModuleRequest>) -> Result<Response<EquipLoadoutResponse>, Status> {
        Ok(Response::new(EquipLoadoutResponse {
            success: true,
            loadout: Self::empty_loadout(),
            error: String::new(),
        }))
    }

    async fn equip_weapon(&self, _request: Request<EquipWeaponRequest>) -> Result<Response<EquipLoadoutResponse>, Status> {
        Ok(Response::new(EquipLoadoutResponse {
            success: true,
            loadout: Self::empty_loadout(),
            error: String::new(),
        }))
    }

    async fn equip_missile(&self, _request: Request<EquipMissileRequest>) -> Result<Response<EquipLoadoutResponse>, Status> {
        Ok(Response::new(EquipLoadoutResponse {
            success: true,
            loadout: Self::empty_loadout(),
            error: String::new(),
        }))
    }
}

pub struct MatchmakingHandler {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl MatchmakingService for MatchmakingHandler {
    async fn queue_for_match(&self, request: Request<QueueForMatchRequest>) -> Result<Response<QueueForMatchResponse>, Status> {
        let _req = request.into_inner();
        let mut mgr = self.state.match_manager.lock().await;

        let player_id = Uuid::new_v4();
        let position = mgr.enqueue(player_id);

        if let Some(_match) = mgr.try_start_match() {
            tracing::info!("Match started!");
        }

        Ok(Response::new(QueueForMatchResponse {
            queued: true,
            state: Some(QueueState {
                position: position as i32,
                estimated_wait_seconds: 30,
                status: "queued".into(),
            }),
        }))
    }

    async fn queue_status(&self, _request: Request<QueueStatusRequest>) -> Result<Response<QueueStatusResponse>, Status> {
        Ok(Response::new(QueueStatusResponse {
            state: Some(QueueState {
                position: 0,
                estimated_wait_seconds: 0,
                status: "unknown".into(),
            }),
        }))
    }

    async fn leave_queue(&self, _request: Request<LeaveQueueRequest>) -> Result<Response<LeaveQueueResponse>, Status> {
        Ok(Response::new(LeaveQueueResponse { left: true }))
    }
}

pub struct MatchHistoryHandler {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl MatchHistoryService for MatchHistoryHandler {
    async fn get_history(&self, request: Request<GetHistoryRequest>) -> Result<Response<GetHistoryResponse>, Status> {
        let _req = request.into_inner();

        let pool = match &self.state.pool {
            Some(p) => p,
            None => return Ok(Response::new(GetHistoryResponse { matches: vec![] })),
        };

        let rows = sqlx::query_as::<_, (String, i64, i32, i32, f32, f32, String)>(
            "SELECT id::text, EXTRACT(EPOCH FROM timestamp)::bigint, kills, deaths, damage_dealt, damage_taken, result::text FROM match_records ORDER BY timestamp DESC LIMIT 50"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let matches = rows
            .into_iter()
            .map(|(id, ts, kills, deaths, dealt, taken, result)| MatchRecord {
                match_id: id,
                timestamp: ts,
                kills,
                deaths,
                damage_dealt: dealt,
                damage_taken: taken,
                result,
            })
            .collect();

        Ok(Response::new(GetHistoryResponse { matches }))
    }
}

pub async fn serve(addr: &str, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    use tonic::transport::Server;

    let auth = AuthHandler { state: state.clone() };
    let shop = ShopHandler { state: state.clone() };
    let hangar = HangarHandler { state: state.clone() };
    let loadout = LoadoutHandler { state: state.clone() };
    let matchmaking = MatchmakingHandler { state: state.clone() };
    let match_history = MatchHistoryHandler { state };

    Server::builder()
        .add_service(crate::proto_gen::grpc::auth::auth_service_server::AuthServiceServer::new(auth))
        .add_service(crate::proto_gen::grpc::shop::shop_service_server::ShopServiceServer::new(shop))
        .add_service(crate::proto_gen::grpc::hangar::hangar_service_server::HangarServiceServer::new(hangar))
        .add_service(crate::proto_gen::grpc::loadout::loadout_service_server::LoadoutServiceServer::new(loadout))
        .add_service(crate::proto_gen::grpc::matchmaking::matchmaking_service_server::MatchmakingServiceServer::new(matchmaking))
        .add_service(crate::proto_gen::grpc::match_history::match_history_service_server::MatchHistoryServiceServer::new(match_history))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new_no_pool() {
        let state = AppState::new(None);
        assert!(state.pool.is_none());
    }

    #[test]
    fn test_empty_loadout_returns_all_empty() {
        let loadout = LoadoutHandler::empty_loadout();
        assert!(loadout.is_some());
        let l = loadout.unwrap();
        assert!(l.passive_module_ids.is_empty());
        assert!(l.active_module_ids.is_empty());
        assert!(l.weapon_id.is_empty());
        assert!(l.missile_id.is_empty());
    }
}
