use regex::Regex;
use std::sync::LazyLock;
use crate::models::ParsedError;
use crate::traits::LogParser;

static GO_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([/\w\.-]+\.go):(\d+)").unwrap()
});

pub struct GoLogParser;

impl LogParser for GoLogParser {
    fn can_parse(&self, message: &str) -> bool {
        message.contains(".go:") || message.contains("panic:") || message.contains("goroutine ")
    }

    fn parse(&self, message: &str) -> ParsedError {
        if let Some(caps) = GO_PATTERN.captures(message) {
            let file = caps.get(1).map_or("main.go", |m| m.as_str());
            let line = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);

            return ParsedError {
                file_path: file.to_string(),
                line_number: line,
                error_type: "Go Runtime Panic".to_string(),
                raw_message: message.to_string(),
            };
        }

        ParsedError {
            file_path: "unknown.go".to_string(),
            line_number: 0,
            error_type: "Go Error".to_string(),
            raw_message: message.to_string(),
        }
    }
}
