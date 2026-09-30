use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use crate::models::{
    EnrichedAppMetadata, IncidentContext, IncidentDetails, RemediationInstructions,
    RepositoryInfo, RichDiagnostics, TelemetryContext,
};
use crate::parsers::ParsedSentryEvent;
use crate::traits::{AppRegistry, Deduplicator, Dispatcher, LogParser};

pub struct IncidentOrchestrator {
    parser: Arc<dyn LogParser>,
    deduplicator: Arc<dyn Deduplicator>,
    registry: Arc<dyn AppRegistry>,
    dispatcher: Arc<dyn Dispatcher>,
    cooldown_seconds: u64,
}

impl IncidentOrchestrator {
    pub fn new(
        parser: Arc<dyn LogParser>,
        deduplicator: Arc<dyn Deduplicator>,
        registry: Arc<dyn AppRegistry>,
        dispatcher: Arc<dyn Dispatcher>,
        cooldown_seconds: u64,
    ) -> Self {
        Self {
            parser,
            deduplicator,
            registry,
            dispatcher,
            cooldown_seconds,
        }
    }

    pub fn deduplicator(&self) -> &Arc<dyn Deduplicator> {
        &self.deduplicator
    }

    pub fn process_alert(&self, raw_alert_data: &Value) -> Option<IncidentContext> {
        // 1. Extract records from OpenObserve payload
        let records = raw_alert_data
            .get("records")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        if records.is_empty() {
            return None;
        }

        let first_record = &records[0];
        let raw_message = first_record
            .get("message")
            .or_else(|| first_record.get("log"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let app_name = first_record
            .get("app_name")
            .and_then(|v| v.as_str())
            .or_else(|| raw_alert_data.get("stream_name").and_then(|v| v.as_str()))
            .unwrap_or("unknown-app");

        // 2. Parse Error via LogParser
        let parsed = self.parser.parse(raw_message);

        // 3. Deduplicate (Circuit Breaker)
        let signature = format!("{}:{}:{}:{}", app_name, parsed.file_path, parsed.line_number, parsed.error_type);
        if self.deduplicator.is_duplicate(&signature, self.cooldown_seconds) {
            return None;
        }
        self.deduplicator.record(&signature);

        // 4. Resolve Metadata via AppRegistry
        let app_meta = self.registry.get_app(app_name);

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut hasher = Sha256::new();
        hasher.update(signature.as_bytes());
        let sig_hash = format!("{:x}", hasher.finalize());
        let event_id = format!("evt_{}_{}", now_sec, &sig_hash[..6]);

        let surrounding_logs: Vec<String> = records
            .iter()
            .take(5)
            .map(|r| {
                r.get("message")
                    .or_else(|| r.get("log"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();

        let (inferred_lang, inferred_cmd) = match parsed.error_type.as_str() {
            "PHP Fatal Error" | "PHP Exception" => ("php", "php artisan test"),
            "Python Exception" => ("python", "pytest"),
            "Go Runtime Panic" => ("go", "go test ./..."),
            _ => ("nodejs", "npm test"),
        };

        let language = if app_meta.language != "unknown" {
            app_meta.language
        } else {
            inferred_lang.to_string()
        };

        let verification_command = if app_meta.verification_command != "npm test" || app_meta.framework != "unknown" {
            app_meta.verification_command
        } else {
            inferred_cmd.to_string()
        };

        // 5. Assemble Standard Agent-Agnostic Payload
        Some(IncidentContext {
            event_id: event_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            app_metadata: EnrichedAppMetadata {
                app_name: app_meta.app_name,
                environment: first_record
                    .get("environment")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&app_meta.environment)
                    .to_string(),
                platform: first_record
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&app_meta.platform)
                    .to_string(),
                language,
                framework: app_meta.framework,
                repository: RepositoryInfo {
                    provider: "github".to_string(),
                    url: app_meta.repo_url,
                    default_branch: app_meta.default_branch,
                    target_branch: format!("hotfix/auto-heal-{}", event_id),
                },
            },
            incident: IncidentDetails {
                error_type: parsed.error_type.clone(),
                error_message: raw_message.chars().take(400).collect(),
                file_path: parsed.file_path.clone(),
                line_number: parsed.line_number,
                stack_trace: surrounding_logs.clone(),
                source_context: None,
                local_variables: None,
            },
            telemetry_context: TelemetryContext {
                openobserve_stream: raw_alert_data
                    .get("stream_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string(),
                trigger_alert_name: raw_alert_data
                    .get("alert_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("generic_alert")
                    .to_string(),
                surrounding_logs,
            },
            remediation_instructions: RemediationInstructions {
                objective: format!("Resolve {} in {}:{}", parsed.error_type, parsed.file_path, parsed.line_number),
                verification_command,
            },
            diagnostics: None,
        })
    }

    /// Process a rich Sentry exception event (captured directly by official Sentry SDKs)
    pub fn process_sentry_event(&self, sentry_event: ParsedSentryEvent) -> Option<IncidentContext> {
        // 1. Deduplicate (Circuit Breaker)
        let signature = format!(
            "{}:{}:{}:{}",
            sentry_event.app_name, sentry_event.file_path, sentry_event.line_number, sentry_event.error_type
        );
        if self.deduplicator.is_duplicate(&signature, self.cooldown_seconds) {
            return None;
        }
        self.deduplicator.record(&signature);

        // 2. Resolve App Metadata from registry
        let app_meta = self.registry.get_app(&sentry_event.app_name);

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let event_id = if !sentry_event.event_id.is_empty() && sentry_event.event_id != "unknown-event" {
            sentry_event.event_id.clone()
        } else {
            let mut hasher = Sha256::new();
            hasher.update(signature.as_bytes());
            let sig_hash = format!("{:x}", hasher.finalize());
            format!("sentry_{}_{}", now_sec, &sig_hash[..6])
        };

        // Determine language
        let language = if app_meta.language != "unknown" {
            app_meta.language.clone()
        } else {
            match sentry_event.platform.to_lowercase().as_str() {
                "php" => "php".to_string(),
                "python" => "python".to_string(),
                "go" => "go".to_string(),
                "node" | "javascript" | "typescript" => "nodejs".to_string(),
                _ => match sentry_event.error_type.as_str() {
                    "PHP Fatal Error" | "PHP Exception" => "php".to_string(),
                    "Python Exception" | "ZeroDivisionError" | "KeyError" => "python".to_string(),
                    "Go Runtime Panic" => "go".to_string(),
                    _ => "nodejs".to_string(),
                },
            }
        };

        // Determine verification command
        let verification_command = if app_meta.verification_command != "npm test" || app_meta.framework != "unknown" {
            app_meta.verification_command.clone()
        } else {
            match language.as_str() {
                "php" => "php artisan test".to_string(),
                "python" => "pytest".to_string(),
                "go" => "go test ./...".to_string(),
                _ => "npm test".to_string(),
            }
        };

        // Surrounding logs from breadcrumbs (chronological SQL queries, outbound HTTP calls)
        let surrounding_logs: Vec<String> = if !sentry_event.breadcrumbs.is_empty() {
            sentry_event
                .breadcrumbs
                .iter()
                .rev()
                .take(5)
                .map(|b| {
                    let cat = b.category.as_deref().unwrap_or("log");
                    let msg = b.message.as_deref().unwrap_or("");
                    if let Some(data) = &b.data {
                        format!("[breadcrumb:{}] {} data={}", cat, msg, data)
                    } else {
                        format!("[breadcrumb:{}] {}", cat, msg)
                    }
                })
                .collect()
        } else {
            sentry_event.stack_trace.iter().take(5).cloned().collect()
        };

        let diagnostics = RichDiagnostics {
            http_request: sentry_event.http_request,
            breadcrumbs: sentry_event.breadcrumbs,
            user: sentry_event.user,
            tags: sentry_event.tags,
        };

        Some(IncidentContext {
            event_id: event_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            app_metadata: EnrichedAppMetadata {
                app_name: app_meta.app_name,
                environment: sentry_event.environment,
                platform: "coolify".to_string(),
                language,
                framework: app_meta.framework,
                repository: RepositoryInfo {
                    provider: "github".to_string(),
                    url: app_meta.repo_url,
                    default_branch: app_meta.default_branch,
                    target_branch: format!("hotfix/auto-heal-{}", event_id),
                },
            },
            incident: IncidentDetails {
                error_type: sentry_event.error_type.clone(),
                error_message: sentry_event.error_message,
                file_path: sentry_event.file_path.clone(),
                line_number: sentry_event.line_number,
                stack_trace: sentry_event.stack_trace,
                source_context: sentry_event.source_context,
                local_variables: sentry_event.local_variables,
            },
            telemetry_context: TelemetryContext {
                openobserve_stream: "sentry_events".to_string(),
                trigger_alert_name: "sentry_sdk".to_string(),
                surrounding_logs,
            },
            remediation_instructions: RemediationInstructions {
                objective: format!(
                    "Resolve {} in {}:{}",
                    sentry_event.error_type, sentry_event.file_path, sentry_event.line_number
                ),
                verification_command,
            },
            diagnostics: Some(diagnostics),
        })
    }

    /// Forward Sentry raw JSON to OpenObserve for unified storage, search, and dashboard analytics
    pub async fn forward_to_openobserve(&self, payload: &Value) {
        let oo_url = match std::env::var("OPENOBSERVE_INGEST_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => return, // Not configured
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        let mut req = client.post(&oo_url).json(&json!([payload]));

        if let Ok(auth_token) = std::env::var("OPENOBSERVE_AUTH_TOKEN") {
            if !auth_token.is_empty() {
                req = req.header("Authorization", format!("Basic {}", auth_token));
            }
        }

        match req.send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    tracing::debug!("Successfully forwarded Sentry event to OpenObserve");
                } else {
                    tracing::warn!(status = %resp.status(), "OpenObserve forward returned non-success status");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to forward event to OpenObserve: {}", e);
            }
        }
    }

    pub async fn dispatch(&self, payload: &IncidentContext) -> bool {
        self.dispatcher.dispatch(payload).await
    }
}

