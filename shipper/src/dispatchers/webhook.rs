use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
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

        let title = format!(
            "{}: {} ({}:{})",
            payload.incident.error_type,
            payload.incident.error_message,
            payload.incident.file_path,
            payload.incident.line_number
        );

        let mut details = format!(
            "App: {}\nError: {}\nFile: {}:{}\nVerification Command: {}\nTarget Branch: {}",
            payload.app_metadata.app_name,
            payload.incident.error_message,
            payload.incident.file_path,
            payload.incident.line_number,
            payload.remediation_instructions.verification_command,
            payload.app_metadata.repository.target_branch
        );

        if !payload.incident.stack_trace.is_empty() {
            details.push_str("\n\nStacktrace:\n");
            for frame in payload.incident.stack_trace.iter().take(6) {
                details.push_str(&format!("  {}\n", frame));
            }
        }

        if let Some(diag) = &payload.diagnostics {
            if let Some(req) = &diag.http_request {
                details.push_str(&format!("\nHTTP Request: {} {}\n", req.method, req.url));
                if let Some(body) = &req.body {
                    details.push_str(&format!("Request Body: {}\n", body));
                }
            }
        }

        let aina_payload = json!({
            "source": "openobserve-sre",
            "severity": "critical",
            "service": payload.app_metadata.app_name,
            "title": title,
            "details": details,
            "repository": payload.app_metadata.repository.url,
            "event_id": payload.event_id,
            "incident_context": payload
        });

        let mut request = self.client.post(&self.target_url).json(&aina_payload);

        if let Some(token) = &self.auth_token {
            if !token.is_empty() {
                request = request
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-API-Key", token);
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
