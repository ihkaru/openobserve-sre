use crate::models::ParsedError;
use crate::traits::LogParser;

pub struct CompositeLogParser {
    parsers: Vec<Box<dyn LogParser>>,
}

impl CompositeLogParser {
    pub fn new(parsers: Vec<Box<dyn LogParser>>) -> Self {
        Self { parsers }
    }

    pub fn register(&mut self, parser: Box<dyn LogParser>) {
        self.parsers.push(parser);
    }
}

impl LogParser for CompositeLogParser {
    fn can_parse(&self, message: &str) -> bool {
        self.parsers.iter().any(|p| p.can_parse(message))
    }

    fn parse(&self, message: &str) -> ParsedError {
        for parser in &self.parsers {
            if parser.can_parse(message) {
                return parser.parse(message);
            }
        }

        ParsedError {
            file_path: "unknown".to_string(),
            line_number: 0,
            error_type: "Generic Application Error".to_string(),
            raw_message: message.to_string(),
        }
    }
}
