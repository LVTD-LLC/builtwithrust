//! Signup-only rate limiting. No work is added to cached page requests.
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct SignupLimiter(Mutex<HashMap<String, (Instant, u32)>>);

impl SignupLimiter {
    /// Single-replica, bounded-memory fixed window. Restarts reset the window.
    /// Before scaling, move this counter to shared storage or the edge.
    pub fn allow(&self, key: String) -> bool {
        let now = Instant::now();
        let mut entries = self.0.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(3600));
        if entries.len() >= 10_000 && !entries.contains_key(&key) {
            return false;
        }
        let (_, count) = entries.entry(key).or_insert((now, 0));
        if *count >= 10 {
            return false;
        }
        *count += 1;
        true
    }
}
