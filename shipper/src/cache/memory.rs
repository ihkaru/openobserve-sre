use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Digest, Sha256};
use crate::traits::Deduplicator;

pub struct InMemoryDeduplicator {
    cache: RwLock<HashMap<String, u64>>,
}

impl InMemoryDeduplicator {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn hash_signature(&self, signature: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(signature.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for InMemoryDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Deduplicator for InMemoryDeduplicator {
    fn is_duplicate(&self, signature: &str, cooldown_seconds: u64) -> bool {
        let key = self.hash_signature(signature);
        let now = self.current_timestamp();

        if let Ok(read_guard) = self.cache.read() {
            if let Some(&last_seen) = read_guard.get(&key) {
                if now.saturating_sub(last_seen) < cooldown_seconds {
                    return true;
                }
            }
        }
        false
    }

    fn record(&self, signature: &str) {
        let key = self.hash_signature(signature);
        let now = self.current_timestamp();

        if let Ok(mut write_guard) = self.cache.write() {
            write_guard.insert(key, now);
        }
    }

    fn count_active(&self) -> usize {
        self.cache.read().map(|guard| guard.len()).unwrap_or(0)
    }
}
