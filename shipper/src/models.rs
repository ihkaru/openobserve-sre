use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceContext {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_context: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_line: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentDetails {
    pub error_type: String,
    pub error_message: String,
    pub file_path: String,
    pub line_number: u32,
    pub stack_trace: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_context: Option<SourceContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_variables: Option<Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpRequestContext {
    pub url: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_string: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreadcrumbItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RichDiagnostics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_request: Option<HttpRequestContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumbs: Vec<BreadcrumbItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentContext {
    pub event_id: String,
    pub timestamp: String,
    pub app_metadata: EnrichedAppMetadata,
    pub incident: IncidentDetails,
    pub telemetry_context: TelemetryContext,
    pub remediation_instructions: RemediationInstructions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<RichDiagnostics>,
}
