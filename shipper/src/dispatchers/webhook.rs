use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use tracing::{error, info, warn};
use crate::models::IncidentContext;
use crate::traits::Dispatcher;

pub struct WebhookDispatcher {
    client: Client,
    target_url: String,
    auth_token: Option<String>,
}

impl WebhookDispatcher {
    pub fn new(target_url: String, auth_token: Option<String>, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();

        Self {
            client,
            target_url,
            auth_token,
        }
    }
}

#[async_trait]
impl Dispatcher for WebhookDispatcher {
    async fn dispatch(&self, payload: &IncidentContext) -> bool {
        if self.target_url.is_empty() {
            warn!("No agent target_url configured; skipping dispatch.");
            return false;
        }

        info!(
            event_id = %payload.event_id,
            app = %payload.app_metadata.app_name,
            target = %self.target_url,
            "Dispatching incident payload to agent webhook"
        );

        let mut request = self.client.post(&self.target_url).json(payload);

        if let Some(token) = &self.auth_token {
            if !token.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }

        match request.send().await {
            Ok(resp) => {
                let status = resp.status();
                info!(status = %status, "Agent webhook response received");
                status.is_success()
            }
            Err(err) => {
                error!(error = %err, "Failed to dispatch payload to agent webhook");
                false
            }
        }
    }
}
