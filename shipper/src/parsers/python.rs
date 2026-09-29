use regex::Regex;
use std::sync::LazyLock;
use crate::models::ParsedError;
use crate::traits::LogParser;

static PYTHON_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"File\s+"([/\w\.-]+\.py)",\s+line\s+(\d+)"#).unwrap()
});

pub struct PythonLogParser;

impl LogParser for PythonLogParser {
    fn can_parse(&self, message: &str) -> bool {
        message.contains(".py") || message.contains("Traceback (most recent call last)")
    }

    fn parse(&self, message: &str) -> ParsedError {
        if let Some(caps) = PYTHON_PATTERN.captures(message) {
            let file = caps.get(1).map_or("unknown.py", |m| m.as_str());
            let line = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);

            return ParsedError {
                file_path: file.to_string(),
                line_number: line,
                error_type: "Python Exception".to_string(),
                raw_message: message.to_string(),
            };
        }

        ParsedError {
            file_path: "unknown.py".to_string(),
            line_number: 0,
            error_type: "Python Generic Error".to_string(),
            raw_message: message.to_string(),
        }
    }
}
