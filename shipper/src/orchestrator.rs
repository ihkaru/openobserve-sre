use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
use serde_json::Value;
use sha2::{Digest, Sha256};
use crate::models::{
    EnrichedAppMetadata, IncidentContext, IncidentDetails, RemediationInstructions,
    RepositoryInfo, TelemetryContext,
};
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
        })
    }

    pub async fn dispatch(&self, payload: &IncidentContext) -> bool {
        self.dispatcher.dispatch(payload).await
    }
}
