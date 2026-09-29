use regex::Regex;
use std::sync::LazyLock;
use crate::models::ParsedError;
use crate::traits::LogParser;

static NODE_PATTERN_1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"at\s+.*?\(([/\w\.-]+\.[jt]sx?):(\d+):(\d+)\)").unwrap()
});
static NODE_PATTERN_2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"at\s+([/\w\.-]+\.[jt]sx?):(\d+):(\d+)").unwrap()
});

pub struct NodeLogParser;

impl LogParser for NodeLogParser {
    fn can_parse(&self, message: &str) -> bool {
        [".js", ".ts", ".jsx", ".tsx", "node:"].iter().any(|ext| message.contains(ext))
    }

    fn parse(&self, message: &str) -> ParsedError {
        let captures = NODE_PATTERN_1.captures(message).or_else(|| NODE_PATTERN_2.captures(message));
        if let Some(caps) = captures {
            let file = caps.get(1).map_or("unknown.js", |m| m.as_str());
            let line = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);

            return ParsedError {
                file_path: file.to_string(),
                line_number: line,
                error_type: "Node/TS Unhandled Exception".to_string(),
                raw_message: message.to_string(),
            };
        }

        ParsedError {
            file_path: "unknown.js".to_string(),
            line_number: 0,
            error_type: "Node Generic Error".to_string(),
            raw_message: message.to_string(),
        }
    }
}
