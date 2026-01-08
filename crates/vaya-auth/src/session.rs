//! Session management

use std::collections::HashMap;
use std::sync::Mutex;

use ring::rand::{SecureRandom, SystemRandom};
use time::{Duration, OffsetDateTime};

use crate::{AuthError, AuthResult};

/// Session data
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Creation time
    pub created_at: i64,
    /// Last activity time
    pub last_activity: i64,
    /// Expiration time
    pub expires_at: i64,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Custom data
    pub data: HashMap<String, String>,
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc().unix_timestamp() > self.expires_at
    }

    /// Update last activity
    pub fn touch(&mut self) {
        self.last_activity = OffsetDateTime::now_utc().unix_timestamp();
    }

    /// Set custom data
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get custom data
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

/// Session store configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session TTL
    pub ttl: Duration,
    /// Session ID length
    pub id_length: usize,
    /// Maximum sessions per user
    pub max_per_user: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::hours(24),
            id_length: 32,
            max_per_user: 5,
        }
    }
}

/// In-memory session store
pub struct SessionStore {
    /// Sessions by ID
    sessions: Mutex<HashMap<String, Session>>,
    /// Session IDs by user
    user_sessions: Mutex<HashMap<String, Vec<String>>>,
    /// Configuration
    config: SessionConfig,
}

impl SessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            user_sessions: Mutex::new(HashMap::new()),
            config,
        }
    }

    /// Create a new session
    pub fn create(&self, user_id: impl Into<String>) -> AuthResult<Session> {
        let user_id = user_id.into();
        let now = OffsetDateTime::now_utc();

        // Generate session ID
        let session_id = self.generate_session_id()?;

        let session = Session {
            id: session_id.clone(),
            user_id: user_id.clone(),
            created_at: now.unix_timestamp(),
            last_activity: now.unix_timestamp(),
            expires_at: (now + self.config.ttl).unix_timestamp(),
            ip_address: None,
            user_agent: None,
            data: HashMap::new(),
        };

        // Store session
        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id.clone(), session.clone());
        }

        // Track user sessions
        {
            let mut user_sessions = self.user_sessions.lock().unwrap();
            let sessions = user_sessions.entry(user_id.clone()).or_insert_with(Vec::new);
            sessions.push(session_id);

            // Enforce max sessions per user
            while sessions.len() > self.config.max_per_user {
                if let Some(old_id) = sessions.first().cloned() {
                    sessions.remove(0);
                    self.sessions.lock().unwrap().remove(&old_id);
                }
            }
        }

        Ok(session)
    }

    /// Create session with metadata
    pub fn create_with_meta(
        &self,
        user_id: impl Into<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AuthResult<Session> {
        let mut session = self.create(user_id)?;
        session.ip_address = ip_address;
        session.user_agent = user_agent;

        // Update stored session
        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session.id.clone(), session.clone());
        }

        Ok(session)
    }

    /// Get a session by ID
    pub fn get(&self, session_id: &str) -> AuthResult<Session> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(session_id)
            .ok_or(AuthError::SessionNotFound)?;

        if session.is_expired() {
            drop(sessions);
            self.remove(session_id);
            return Err(AuthError::SessionExpired);
        }

        Ok(session.clone())
    }

    /// Validate and refresh a session
    pub fn validate(&self, session_id: &str) -> AuthResult<Session> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(session_id)
            .ok_or(AuthError::SessionNotFound)?;

        if session.is_expired() {
            let id = session.id.clone();
            drop(sessions);
            self.remove(&id);
            return Err(AuthError::SessionExpired);
        }

        session.touch();
        Ok(session.clone())
    }

    /// Remove a session
    pub fn remove(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.remove(session_id) {
            let mut user_sessions = self.user_sessions.lock().unwrap();
            if let Some(user_sess) = user_sessions.get_mut(&session.user_id) {
                user_sess.retain(|id| id != session_id);
            }
        }
    }

    /// Remove all sessions for a user
    pub fn remove_user_sessions(&self, user_id: &str) {
        let mut user_sessions = self.user_sessions.lock().unwrap();
        if let Some(session_ids) = user_sessions.remove(user_id) {
            let mut sessions = self.sessions.lock().unwrap();
            for id in session_ids {
                sessions.remove(&id);
            }
        }
    }

    /// Get all sessions for a user
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<Session> {
        let user_sessions = self.user_sessions.lock().unwrap();
        let sessions = self.sessions.lock().unwrap();

        user_sessions
            .get(user_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| sessions.get(id))
                    .filter(|s| !s.is_expired())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clean up expired sessions
    pub fn cleanup(&self) {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let mut sessions = self.sessions.lock().unwrap();
        let mut user_sessions = self.user_sessions.lock().unwrap();

        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.expires_at < now)
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired {
            if let Some(session) = sessions.remove(&id) {
                if let Some(user_sess) = user_sessions.get_mut(&session.user_id) {
                    user_sess.retain(|sid| sid != &id);
                }
            }
        }
    }

    /// Generate a secure session ID
    fn generate_session_id(&self) -> AuthResult<String> {
        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; self.config.id_length];
        rng.fill(&mut bytes)
            .map_err(|_| AuthError::Internal("Failed to generate session ID".into()))?;

        // Encode as hex
        Ok(bytes.iter().map(|b| format!("{:02x}", b)).collect())
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let store = SessionStore::new();
        let session = store.create("user-123").unwrap();

        assert!(!session.id.is_empty());
        assert_eq!(session.user_id, "user-123");
        assert!(!session.is_expired());
    }

    #[test]
    fn test_get_session() {
        let store = SessionStore::new();
        let session = store.create("user-123").unwrap();

        let fetched = store.get(&session.id).unwrap();
        assert_eq!(fetched.user_id, "user-123");
    }

    #[test]
    fn test_remove_session() {
        let store = SessionStore::new();
        let session = store.create("user-123").unwrap();

        store.remove(&session.id);
        assert!(store.get(&session.id).is_err());
    }

    #[test]
    fn test_max_sessions_per_user() {
        let config = SessionConfig {
            max_per_user: 2,
            ..Default::default()
        };
        let store = SessionStore::with_config(config);

        let s1 = store.create("user-123").unwrap();
        let s2 = store.create("user-123").unwrap();
        let s3 = store.create("user-123").unwrap();

        // s1 should be evicted
        assert!(store.get(&s1.id).is_err());
        assert!(store.get(&s2.id).is_ok());
        assert!(store.get(&s3.id).is_ok());
    }

    #[test]
    fn test_user_sessions() {
        let store = SessionStore::new();
        store.create("user-123").unwrap();
        store.create("user-123").unwrap();
        store.create("user-456").unwrap();

        let sessions = store.get_user_sessions("user-123");
        assert_eq!(sessions.len(), 2);

        store.remove_user_sessions("user-123");
        let sessions = store.get_user_sessions("user-123");
        assert_eq!(sessions.len(), 0);
    }
}
