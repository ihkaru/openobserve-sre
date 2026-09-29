use async_trait::async_trait;
use crate::models::{AppMetadata, IncidentContext, ParsedError};

/// Single Responsibility Principle: Parse error signatures from logs.
pub trait LogParser: Send + Sync {
    fn can_parse(&self, message: &str) -> bool;
    fn parse(&self, message: &str) -> ParsedError;
}

/// Single Responsibility Principle: Deduplicate recurring errors and handle cooldown.
pub trait Deduplicator: Send + Sync {
    fn is_duplicate(&self, signature: &str, cooldown_seconds: u64) -> bool;
    fn record(&self, signature: &str);
    fn count_active(&self) -> usize;
}

/// Single Responsibility Principle: Resolve application configuration and repo mappings.
pub trait AppRegistry: Send + Sync {
    fn get_app(&self, app_name: &str) -> AppMetadata;
}

/// Single Responsibility Principle: Dispatch the incident payload to external consumers (Aina).
#[async_trait]
pub trait Dispatcher: Send + Sync {
    async fn dispatch(&self, payload: &IncidentContext) -> bool;
}
