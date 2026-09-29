use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedError {
    pub file_path: String,
    pub line_number: u32,
    pub error_type: String,
    pub raw_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub app_name: String,
    pub environment: String,
    pub platform: String,
    pub language: String,
    pub framework: String,
    pub repo_url: String,
    pub default_branch: String,
    pub verification_command: String,
}

impl Default for AppMetadata {
    fn default() -> Self {
        Self {
            app_name: "unknown-app".to_string(),
            environment: "production".to_string(),
            platform: "coolify".to_string(),
            language: "unknown".to_string(),
            framework: "unknown".to_string(),
            repo_url: "".to_string(),
            default_branch: "main".to_string(),
            verification_command: "npm test".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub provider: String,
    pub url: String,
    pub default_branch: String,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedAppMetadata {
    pub app_name: String,
    pub environment: String,
    pub platform: String,
    pub language: String,
    pub framework: String,
    pub repository: RepositoryInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentDetails {
    pub error_type: String,
    pub error_message: String,
    pub file_path: String,
    pub line_number: u32,
    pub stack_trace: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryContext {
    pub openobserve_stream: String,
    pub trigger_alert_name: String,
    pub surrounding_logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationInstructions {
    pub objective: String,
    pub verification_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentContext {
    pub event_id: String,
    pub timestamp: String,
    pub app_metadata: EnrichedAppMetadata,
    pub incident: IncidentDetails,
    pub telemetry_context: TelemetryContext,
    pub remediation_instructions: RemediationInstructions,
}
