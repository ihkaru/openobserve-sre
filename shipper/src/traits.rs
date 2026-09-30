use async_trait::async_trait;
use crate::models::{AppMetadata, IncidentContext, ParsedError};

/// Parses error signatures from logs.
pub trait LogParser: Send + Sync {
    fn can_parse(&self, message: &str) -> bool;
    fn parse(&self, message: &str) -> ParsedError;
}

/// Deduplicates recurring errors and handles cooldown.
pub trait Deduplicator: Send + Sync {
    fn is_duplicate(&self, signature: &str, cooldown_seconds: u64) -> bool;
    fn record(&self, signature: &str);
    fn count_active(&self) -> usize;
}

/// Resolves application configuration and repo mappings.
pub trait AppRegistry: Send + Sync {
    fn get_app(&self, app_name: &str) -> AppMetadata;
}

/// Dispatches the incident payload to external coding agents or webhooks.
#[async_trait]
pub trait Dispatcher: Send + Sync {
    async fn dispatch(&self, payload: &IncidentContext) -> bool;
}
