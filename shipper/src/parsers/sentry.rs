use std::collections::HashMap;
use std::io::Read;
use flate2::read::GzDecoder;
use serde_json::Value;
use tracing::warn;
use crate::models::{
    BreadcrumbItem, HttpRequestContext, SourceContext,
};

#[derive(Debug, Clone)]
pub struct ParsedSentryEvent {
    pub event_id: String,
    pub app_name: String,
    pub error_type: String,
    pub error_message: String,
    pub file_path: String,
    pub line_number: u32,
    pub stack_trace: Vec<String>,
    pub source_context: Option<SourceContext>,
    pub local_variables: Option<Value>,
    pub http_request: Option<HttpRequestContext>,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub platform: String,
    pub environment: String,
    pub user: Option<Value>,
    pub tags: HashMap<String, String>,
    pub raw_json: Value,
}

pub struct SentryParser;

impl SentryParser {
    /// Parse a Sentry Envelope (used by modern Sentry SDKs v7+ via /api/:project_id/envelope).
    /// An envelope format consists of:
    /// - Line 1: Envelope Header JSON
    /// - Repeated Items: Item Header JSON, followed by payload (bytes/JSON)
    pub fn parse_envelope(raw_bytes: &[u8], project_id: &str) -> Result<ParsedSentryEvent, String> {
        let decompressed: Vec<u8>;
        let payload_bytes = if raw_bytes.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = GzDecoder::new(raw_bytes);
            let mut buf = Vec::new();
            if let Err(e) = decoder.read_to_end(&mut buf) {
                return Err(format!("Failed to decompress gzip Sentry envelope: {}", e));
            }
            decompressed = buf;
            &decompressed[..]
        } else {
            raw_bytes
        };

        let text = String::from_utf8_lossy(payload_bytes);
        let mut lines = text.lines();

        let header_line = lines.next().ok_or_else(|| "Empty envelope body".to_string())?;
        let envelope_header: Value = serde_json::from_str(header_line)
            .map_err(|e| format!("Failed to parse envelope header: {}", e))?;

        let header_event_id = envelope_header
            .get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        while let Some(item_header_line) = lines.next() {
            if item_header_line.trim().is_empty() {
                continue;
            }

            let item_header: Value = match serde_json::from_str(item_header_line) {
                Ok(v) => v,
                Err(e) => {
                    warn!("Failed to parse Sentry item header: {}", e);
                    continue;
                }
            };

            let item_type = item_header
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("event");

            if let Some(item_payload_line) = lines.next() {
                if item_type == "event" {
                    match serde_json::from_str::<Value>(item_payload_line) {
                        Ok(mut event_json) => {
                            if event_json.get("event_id").is_none() && !header_event_id.is_empty() {
                                event_json["event_id"] = Value::String(header_event_id.clone());
                            }
                            return Self::parse_event_json(&event_json, project_id);
                        }
                        Err(e) => {
                            return Err(format!("Failed to parse Sentry event payload JSON: {}", e));
                        }
                    }
                }
            }
        }

        // If no explicit event item was found (e.g. only session or transaction metrics)
        Err("No error event item found in Sentry envelope".to_string())
    }

    /// Parse a Sentry Store payload (used by /api/:project_id/store)
    pub fn parse_store(event_json: &Value, project_id: &str) -> Result<ParsedSentryEvent, String> {
        Self::parse_event_json(event_json, project_id)
    }

    /// Extract fields from Sentry Event JSON into ParsedSentryEvent
    pub fn parse_event_json(event: &Value, project_id: &str) -> Result<ParsedSentryEvent, String> {
        let event_id = event
            .get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "unknown-event")
            .to_string();

        let platform = event
            .get("platform")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let environment = event
            .get("environment")
            .and_then(|v| v.as_str())
            .unwrap_or("production")
            .to_string();

        // Extract tags
        let mut tags = HashMap::new();
        if let Some(tag_map) = event.get("tags") {
            if let Some(obj) = tag_map.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        tags.insert(k.clone(), s.to_string());
                    }
                }
            } else if let Some(arr) = tag_map.as_array() {
                // Sentry sometimes sends tags as [["key", "value"]]
                for item in arr {
                    if let Some(pair) = item.as_array() {
                        if pair.len() == 2 {
                            if let (Some(k), Some(v)) = (pair[0].as_str(), pair[1].as_str()) {
                                tags.insert(k.to_string(), v.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Determine app_name
        let app_name = if let Some(name) = tags.get("app_name").or_else(|| tags.get("app")) {
            name.clone()
        } else if let Some(server_name) = event.get("server_name").and_then(|v| v.as_str()) {
            server_name.to_string()
        } else if !project_id.is_empty() && project_id != "1" && !project_id.chars().all(|c| c.is_ascii_digit()) {
            project_id.to_string()
        } else if let Some(release) = event.get("release").and_then(|v| v.as_str()) {
            release.split('@').next().unwrap_or(project_id).to_string()
        } else if !project_id.is_empty() {
            project_id.to_string()
        } else {
            "unknown-app".to_string()
        };

        // Extract Exception and Stack Trace
        let mut error_type = "Error".to_string();
        let mut error_message = String::new();
        let mut file_path = "unknown".to_string();
        let mut line_number = 1u32;
        let mut stack_trace = Vec::new();
        let mut source_context = None;
        let mut local_variables = None;

        if let Some(exc_values) = event.get("exception").and_then(|e| e.get("values")).and_then(|v| v.as_array()) {
            if let Some(last_exc) = exc_values.last() {
                if let Some(t) = last_exc.get("type").and_then(|v| v.as_str()) {
                    error_type = t.to_string();
                }
                if let Some(m) = last_exc.get("value").and_then(|v| v.as_str()) {
                    error_message = m.to_string();
                }

                if let Some(frames) = last_exc
                    .get("stacktrace")
                    .and_then(|s| s.get("frames"))
                    .and_then(|f| f.as_array())
                {
                    // Look for the crashing frame (preferably in_app: true, or the last frame)
                    let target_frame = frames
                        .iter()
                        .rev()
                        .find(|f| f.get("in_app").and_then(|v| v.as_bool()).unwrap_or(false))
                        .or_else(|| frames.last());

                    if let Some(frame) = target_frame {
                        if let Some(f) = frame.get("filename").or_else(|| frame.get("abs_path")).and_then(|v| v.as_str()) {
                            file_path = f.to_string();
                        }
                        if let Some(l) = frame.get("lineno").and_then(|v| v.as_u64()) {
                            line_number = l as u32;
                        }

                        let pre_context: Vec<String> = frame
                            .get("pre_context")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|str_val| str_val.to_string())).collect())
                            .unwrap_or_default();

                        let context_line = frame
                            .get("context_line")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let post_context: Vec<String> = frame
                            .get("post_context")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|str_val| str_val.to_string())).collect())
                            .unwrap_or_default();

                        if context_line.is_some() || !pre_context.is_empty() || !post_context.is_empty() {
                            source_context = Some(SourceContext {
                                pre_context,
                                context_line,
                                post_context,
                            });
                        }

                        if let Some(vars) = frame.get("vars") {
                            local_variables = Some(vars.clone());
                        }
                    }

                    // Format human-readable stack trace lines
                    for frame in frames {
                        let fn_name = frame.get("function").and_then(|v| v.as_str()).unwrap_or("<anonymous>");
                        let f_name = frame.get("filename").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let l_no = frame.get("lineno").and_then(|v| v.as_u64()).unwrap_or(0);
                        stack_trace.push(format!("at {} ({}:{})", fn_name, f_name, l_no));
                    }
                }
            }
        } else if let Some(msg) = event.get("message").and_then(|v| v.as_str()) {
            error_type = "CapturedMessage".to_string();
            error_message = msg.to_string();
        } else if let Some(logentry) = event.get("logentry").and_then(|l| l.get("message")).and_then(|v| v.as_str()) {
            error_type = "LogMessage".to_string();
            error_message = logentry.to_string();
        }

        // Extract HTTP Request Context
        let mut http_request = None;
        if let Some(req) = event.get("request") {
            let url = req.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
            let query_string = req.get("query_string").and_then(|v| v.as_str()).map(|s| s.to_string());
            let body = req.get("data").cloned();

            let mut headers = HashMap::new();
            if let Some(h) = req.get("headers").and_then(|v| v.as_object()) {
                for (k, v) in h {
                    if let Some(s) = v.as_str() {
                        headers.insert(k.clone(), s.to_string());
                    }
                }
            }

            if !url.is_empty() || body.is_some() || !headers.is_empty() {
                http_request = Some(HttpRequestContext {
                    url,
                    method,
                    query_string,
                    body,
                    headers,
                });
            }
        }

        // Extract Breadcrumbs (SQL queries, outbound HTTP calls, logger)
        let mut breadcrumbs = Vec::new();
        if let Some(bc_values) = event.get("breadcrumbs").and_then(|b| b.get("values")).and_then(|v| v.as_array()) {
            for b in bc_values {
                breadcrumbs.push(BreadcrumbItem {
                    timestamp: b.get("timestamp").and_then(|v| v.as_f64()),
                    category: b.get("category").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    level: b.get("level").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    message: b.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    data: b.get("data").cloned(),
                });
            }
        }

        let user = event.get("user").cloned();

        Ok(ParsedSentryEvent {
            event_id,
            app_name,
            error_type,
            error_message,
            file_path,
            line_number,
            stack_trace,
            source_context,
            local_variables,
            http_request,
            breadcrumbs,
            platform,
            environment,
            user,
            tags,
            raw_json: event.clone(),
        })
    }
}
