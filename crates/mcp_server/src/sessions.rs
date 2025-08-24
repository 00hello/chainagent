use std::{collections::HashMap, sync::RwLock, time::{Duration, Instant}};

#[derive(Clone, Debug)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
    #[allow(dead_code)]
    pub at: Instant,
}

#[derive(Clone, Debug)]
pub struct SessionData {
    pub turns: Vec<ChatTurn>,
    pub partial_intent: Option<serde_json::Value>,
    pub updated_at: Instant,
}

pub struct SessionStore {
    inner: RwLock<HashMap<String, SessionData>>,
    pub ttl: Duration,
    pub max_turns_per_session: usize,
    pub max_sessions: usize,
}

impl SessionStore {
    pub fn new(ttl_seconds: u64, max_turns_per_session: usize, max_sessions: usize) -> Self {
        Self { inner: RwLock::new(HashMap::new()), ttl: Duration::from_secs(ttl_seconds), max_turns_per_session, max_sessions }
    }

    pub fn get(&self, session_id: &str) -> SessionData {
        self.evict_expired();
        let mut map = self.inner.write().unwrap();
        let now = Instant::now();
        let entry = map.entry(session_id.to_string()).or_insert(SessionData { turns: Vec::new(), partial_intent: None, updated_at: now });
        entry.clone()
    }

    pub fn append(&self, session_id: &str, role: String, content: String) {
        self.evict_expired();
        let mut map = self.inner.write().unwrap();
        if map.len() >= self.max_sessions && !map.contains_key(session_id) {
            // naive eviction: drop an arbitrary one
            if let Some(k) = map.keys().next().cloned() { map.remove(&k); }
        }
        let entry = map.entry(session_id.to_string()).or_insert(SessionData { turns: Vec::new(), partial_intent: None, updated_at: Instant::now() });
        entry.turns.push(ChatTurn { role, content, at: Instant::now() });
        if entry.turns.len() > self.max_turns_per_session { entry.turns.drain(0..(entry.turns.len() - self.max_turns_per_session)); }
        entry.updated_at = Instant::now();
    }

    pub fn set_partial_intent(&self, session_id: &str, intent: serde_json::Value) {
        let mut map = self.inner.write().unwrap();
        let entry = map.entry(session_id.to_string()).or_insert(SessionData { turns: Vec::new(), partial_intent: None, updated_at: Instant::now() });
        entry.partial_intent = Some(intent);
        entry.updated_at = Instant::now();
    }

    pub fn get_partial_intent(&self, session_id: &str) -> Option<serde_json::Value> {
        let map = self.inner.read().unwrap();
        map.get(session_id).and_then(|s| s.partial_intent.clone())
    }

    fn evict_expired(&self) {
        let mut map = self.inner.write().unwrap();
        let now = Instant::now();
        map.retain(|_, v| now.duration_since(v.updated_at) <= self.ttl);
    }
}


