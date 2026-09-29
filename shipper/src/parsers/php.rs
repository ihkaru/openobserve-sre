use regex::Regex;
use std::sync::LazyLock;
use crate::models::ParsedError;
use crate::traits::LogParser;

static PHP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)in\s+([/\w\.-]+\.php)\s+on\s+line\s+(\d+)").unwrap()
});

pub struct PhpLogParser;

impl LogParser for PhpLogParser {
    fn can_parse(&self, message: &str) -> bool {
        let lower = message.to_lowercase();
        lower.contains("php") || lower.contains(".php")
    }

    fn parse(&self, message: &str) -> ParsedError {
        if let Some(caps) = PHP_PATTERN.captures(message) {
            let file = caps.get(1).map_or("unknown.php", |m| m.as_str());
            let line = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);
            let error_type = if message.contains("Fatal error") {
                "PHP Fatal Error"
            } else {
                "PHP Exception"
            };

            return ParsedError {
                file_path: file.to_string(),
                line_number: line,
                error_type: error_type.to_string(),
                raw_message: message.to_string(),
            };
        }

        ParsedError {
            file_path: "unknown.php".to_string(),
            line_number: 0,
            error_type: "PHP Generic Error".to_string(),
            raw_message: message.to_string(),
        }
    }
}
