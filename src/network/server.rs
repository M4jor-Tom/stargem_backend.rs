use crate::api::GameService;
use crate::network::{ClientMessage, ClientSession, ServerMessage, SessionManager};
use crate::AppError;
use bytes::{Buf, Bytes};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct GameServer {
    session_manager: Arc<SessionManager>,
    game_service: Arc<GameService>,
    address: String,
}

impl GameServer {
    pub fn new(address: String, game_service: Arc<GameService>) -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
            game_service,
            address,
        }
    }

    pub fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }

    pub async fn start(&self) -> Result<(), AppError> {
        let listener = TcpListener::bind(&self.address).await?;
        info!("Game server listening on {}", self.address);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New connection from {}", addr);
                    let session_manager = self.session_manager.clone();
                    let game_service = self.game_service.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(socket, session_manager, game_service).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    socket: TcpStream,
    session_manager: Arc<SessionManager>,
    game_service: Arc<GameService>,
) -> Result<(), AppError> {
    let (mut read, mut write) = socket.into_split();
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

    let session = Arc::new(tokio::sync::RwLock::new(ClientSession::new(tx)));
    session_manager.add_session(session.clone());
    let session_id = session.blocking_read().id;

    let sm_reader = session_manager.clone();
    let gs_reader = game_service.clone();

    let reader_handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        loop {
            match read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let data = Bytes::copy_from_slice(&buf[..n]);
                    if let Err(e) = process_message(&sm_reader, &gs_reader, session_id, &data).await {
                        error!("Error processing message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    let writer_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let data = match serialize_message(&msg) {
                Ok(d) => d,
                Err(e) => {
                    error!("Serialization error: {}", e);
                    continue;
                }
            };
            if let Err(e) = write.write_all(&data).await {
                error!("Write error: {}", e);
                break;
            }
        }
    });

    let _ = tokio::join!(reader_handle, writer_handle);
    session_manager.remove_session(session_id);
    Ok(())
}

async fn process_message(
    session_manager: &SessionManager,
    game_service: &GameService,
    session_id: Uuid,
    data: &Bytes,
) -> Result<(), AppError> {
    let mut buf = data.clone();

    while buf.len() >= 4 {
        let msg_len = buf.get_u32() as usize;
        if buf.len() < msg_len {
            break;
        }

        let msg_data = buf.slice(..msg_len);
        buf.advance(msg_len);

        if let Ok(msg) = serde_json::from_slice::<ClientMessage>(msg_data.as_ref()) {
            debug!("Received: {:?}", msg);
            handle_client_message(session_manager, game_service, session_id, msg).await?;
        }
    }

    Ok(())
}

async fn handle_client_message(
    session_manager: &SessionManager,
    game_service: &GameService,
    session_id: Uuid,
    msg: ClientMessage,
) -> Result<(), AppError> {
    if let Some(response) = game_service.handle_message(session_id, msg).await? {
        let session = session_manager.get_session(session_id)
            .ok_or_else(|| AppError::Network("Session not found".into()))?;
        let sender = session.blocking_read().get_sender();
        let _ = sender.send(response).await;
    }
    Ok(())
}

fn serialize_message(msg: &ServerMessage) -> Result<Vec<u8>, serde_json::Error> {
    let data = serde_json::to_vec(msg)?;
    let mut result = Vec::with_capacity(4 + data.len());
    result.extend_from_slice(&(data.len() as u32).to_be_bytes());
    result.extend_from_slice(&data);
    Ok(result)
}
