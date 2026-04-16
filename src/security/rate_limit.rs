use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct RateLimiter {
    requests: RwLock<HashMap<String, Vec<Instant>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    pub fn check(&self, key: &str) -> RateLimitResult {
        let mut requests = self.requests.write();
        let now = Instant::now();
        let window_start = now - Duration::from_secs(self.window_secs);

        let timestamps = requests.entry(key.to_string()).or_insert_with(Vec::new);
        timestamps.retain(|t| *t > window_start);

        if timestamps.len() >= self.max_requests {
            let oldest = timestamps.first().map(|t| *t).unwrap_or(now);
            let retry_after = (window_start + Duration::from_secs(self.window_secs) - oldest)
                .as_secs()
                .max(1);
            return RateLimitResult::Limited(retry_after);
        }

        timestamps.push(now);
        RateLimitResult::Allowed
    }

    pub fn cleanup(&self) {
        let mut requests = self.requests.write();
        let now = Instant::now();
        let window_start = now - Duration::from_secs(self.window_secs * 2);
        requests.retain(|_, v| v.iter().any(|t| *t > window_start));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    Allowed,
    Limited(u64),
}

pub struct BruteForceProtector {
    failed_attempts: RwLock<HashMap<String, FailedLoginAttempt>>,
    max_attempts: usize,
    lockout_duration_secs: u64,
    reset_after_secs: u64,
}

impl BruteForceProtector {
    pub fn new(max_attempts: usize, lockout_duration_secs: u64, reset_after_secs: u64) -> Self {
        Self {
            failed_attempts: RwLock::new(HashMap::new()),
            max_attempts,
            lockout_duration_secs,
            reset_after_secs,
        }
    }

    pub fn check(&self, identifier: &str) -> BruteForceResult {
        let mut attempts = self.failed_attempts.write();
        let now = Instant::now();

        if let Some(attempt) = attempts.get_mut(identifier) {
            if attempt.locked_until.is_some() && attempt.locked_until.unwrap() > now {
                return BruteForceResult::Locked {
                    remaining_secs: attempt.locked_until.unwrap().duration_since(now).as_secs(),
                };
            }

            if let Some(last_attempt) = attempt.last_attempt {
                if now.duration_since(last_attempt).as_secs() > self.reset_after_secs {
                    *attempt = FailedLoginAttempt::default();
                }
            }

            if attempt.count >= self.max_attempts {
                attempt.locked_until = Some(now + Duration::from_secs(self.lockout_duration_secs));
                return BruteForceResult::Locked {
                    remaining_secs: self.lockout_duration_secs,
                };
            }
        }

        BruteForceResult::Allowed
    }

    pub fn record_failure(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write();
        let attempt = attempts
            .entry(identifier.to_string())
            .or_insert_with(FailedLoginAttempt::default);
        attempt.count += 1;
        attempt.last_attempt = Some(Instant::now());
    }

    pub fn record_success(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write();
        attempts.remove(identifier);
    }

    pub fn cleanup(&self) {
        let mut attempts = self.failed_attempts.write();
        let now = Instant::now();
        attempts.retain(|_, v| {
            if let Some(locked) = v.locked_until {
                locked > now
            } else {
                v.last_attempt.map_or(false, |t| {
                    now.duration_since(t).as_secs() < self.reset_after_secs * 2
                })
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BruteForceResult {
    Allowed,
    Locked { remaining_secs: u64 },
}

#[derive(Debug, Default)]
struct FailedLoginAttempt {
    count: usize,
    last_attempt: Option<Instant>,
    locked_until: Option<Instant>,
}

pub struct SessionTimeoutManager {
    sessions: RwLock<HashMap<Uuid, SessionState>>,
    idle_timeout_secs: u64,
    absolute_timeout_secs: u64,
}

impl SessionTimeoutManager {
    pub fn new(idle_timeout_secs: u64, absolute_timeout_secs: u64) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            idle_timeout_secs,
            absolute_timeout_secs,
        }
    }

    pub fn register_session(&self, session_id: Uuid, created_at: chrono::DateTime<chrono::Utc>) {
        let mut sessions = self.sessions.write();
        sessions.insert(
            session_id,
            SessionState {
                last_activity: chrono::Utc::now(),
                created_at,
                validated: false,
            },
        );
    }

    pub fn touch(&self, session_id: Uuid) -> bool {
        let mut sessions = self.sessions.write();
        if let Some(state) = sessions.get_mut(&session_id) {
            state.last_activity = chrono::Utc::now();
            state.validated = true;
            true
        } else {
            false
        }
    }

    pub fn check_timeout(&self, session_id: Uuid) -> Option<SessionTimeoutReason> {
        let sessions = self.sessions.read();
        if let Some(state) = sessions.get(&session_id) {
            let now = chrono::Utc::now();

            let idle_elapsed = (now - state.last_activity).num_seconds() as u64;
            if idle_elapsed >= self.idle_timeout_secs {
                return Some(SessionTimeoutReason::IdleTimeout);
            }

            let absolute_elapsed = (now - state.created_at).num_seconds() as u64;
            if absolute_elapsed >= self.absolute_timeout_secs {
                return Some(SessionTimeoutReason::AbsoluteTimeout);
            }
        }
        None
    }

    pub fn remove_session(&self, session_id: Uuid) {
        let mut sessions = self.sessions.write();
        sessions.remove(&session_id);
    }

    pub fn cleanup(&self) -> Vec<Uuid> {
        let mut sessions = self.sessions.write();
        let now = chrono::Utc::now();
        let mut expired = Vec::new();

        sessions.retain(|id, state| {
            let idle_elapsed = (now - state.last_activity).num_seconds() as u64;
            let absolute_elapsed = (now - state.created_at).num_seconds() as u64;
            let should_remove = idle_elapsed >= self.idle_timeout_secs
                || absolute_elapsed >= self.absolute_timeout_secs;
            if should_remove {
                expired.push(*id);
            }
            !should_remove
        });

        expired
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionTimeoutReason {
    IdleTimeout,
    AbsoluteTimeout,
}

struct SessionState {
    last_activity: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    validated: bool,
}
