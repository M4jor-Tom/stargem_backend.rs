#![allow(dead_code)]

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid authentication token")]
    InvalidToken,
    #[error("Session expired")]
    SessionExpired,
    #[error("Steam API error: {0}")]
    SteamApiError(String),
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, token: &str) -> Result<Uuid, AuthError>;
    async fn validate_session(&self, session_id: &str) -> Result<Uuid, AuthError>;
}

pub struct MockAuthProvider {
    sessions: Mutex<HashMap<String, Uuid>>,
    next_user_id: Mutex<u64>,
}

impl MockAuthProvider {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_user_id: Mutex::new(1),
        }
    }
}

#[async_trait]
impl AuthProvider for MockAuthProvider {
    async fn authenticate(&self, _token: &str) -> Result<Uuid, AuthError> {
        let mut next = self.next_user_id.lock().unwrap();
        let user_id = Uuid::from_u128(*next as u128);
        *next += 1;

        let session_id = Uuid::new_v4().to_string();
        self.sessions.lock().unwrap().insert(session_id, user_id);
        Ok(user_id)
    }

    async fn validate_session(&self, session_id: &str) -> Result<Uuid, AuthError> {
        self.sessions
            .lock()
            .unwrap()
            .get(session_id)
            .copied()
            .ok_or(AuthError::SessionExpired)
    }
}

pub struct SteamAuthProvider;

#[async_trait]
impl AuthProvider for SteamAuthProvider {
    async fn authenticate(&self, _token: &str) -> Result<Uuid, AuthError> {
        Err(AuthError::SteamApiError("not implemented".into()))
    }

    async fn validate_session(&self, _session_id: &str) -> Result<Uuid, AuthError> {
        Err(AuthError::SteamApiError("not implemented".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        assert_eq!(AuthError::InvalidToken.to_string(), "Invalid authentication token");
        assert_eq!(AuthError::SessionExpired.to_string(), "Session expired");
        assert_eq!(
            AuthError::SteamApiError("boom".into()).to_string(),
            "Steam API error: boom"
        );
    }

    #[test]
    fn test_mock_auth_provider_new_no_sessions() {
        let provider = MockAuthProvider::new();
        assert!(provider.sessions.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_authenticate_returns_incrementing_user_ids() {
        let provider = MockAuthProvider::new();
        let id1 = provider.authenticate("a").await.unwrap();
        let id2 = provider.authenticate("b").await.unwrap();
        assert_eq!(id1, Uuid::from_u128(1));
        assert_eq!(id2, Uuid::from_u128(2));
    }

    #[tokio::test]
    async fn test_validate_session_valid_token() {
        let provider = MockAuthProvider::new();
        let user_id = provider.authenticate("token").await.unwrap();
        let session = provider.sessions.lock().unwrap().iter().find_map(|(k, &v)| {
            if v == user_id { Some(k.clone()) } else { None }
        }).unwrap();
        let validated = provider.validate_session(&session).await.unwrap();
        assert_eq!(validated, user_id);
    }

    #[tokio::test]
    async fn test_validate_session_expired() {
        let provider = MockAuthProvider::new();
        let result = provider.validate_session("nonexistent").await;
        assert!(matches!(result, Err(AuthError::SessionExpired)));
    }

    #[tokio::test]
    async fn test_steam_auth_provider_returns_not_implemented() {
        let provider = SteamAuthProvider;
        assert!(matches!(
            provider.authenticate("x").await,
            Err(AuthError::SteamApiError(_))
        ));
        assert!(matches!(
            provider.validate_session("x").await,
            Err(AuthError::SteamApiError(_))
        ));
    }
}
