pub mod models;
pub mod traits;
pub mod parsers;
pub mod cache;
pub mod registry;
pub mod dispatchers;
pub mod orchestrator;
pub mod api;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use traits::{Deduplicator, LogParser};
    use parsers::{PhpLogParser, NodeLogParser, PythonLogParser, CompositeLogParser};
    use cache::InMemoryDeduplicator;
    use registry::YamlAppRegistry;
    use dispatchers::WebhookDispatcher;
    use orchestrator::IncidentOrchestrator;
    use serde_json::json;

    #[test]
    fn test_php_parser() {
        let parser = PhpLogParser;
        let msg = "PHP Fatal error: Uncaught Error: Call to undefined method Order::save() in /var/www/html/app/Order.php on line 120";
        assert!(parser.can_parse(msg));
        let parsed = parser.parse(msg);
        assert_eq!(parsed.file_path, "/var/www/html/app/Order.php");
        assert_eq!(parsed.line_number, 120);
        assert_eq!(parsed.error_type, "PHP Fatal Error");
    }

    #[test]
    fn test_node_parser() {
        let parser = NodeLogParser;
        let msg = "TypeError: Cannot read properties of undefined (reading 'token')\n    at AuthService.login (/app/src/auth.ts:45:12)";
        assert!(parser.can_parse(msg));
        let parsed = parser.parse(msg);
        assert_eq!(parsed.file_path, "/app/src/auth.ts");
        assert_eq!(parsed.line_number, 45);
    }

    #[test]
    fn test_python_parser() {
        let parser = PythonLogParser;
        let msg = "Traceback (most recent call last):\n  File \"/srv/api/main.py\", line 34, in index\nZeroDivisionError: division by zero";
        assert!(parser.can_parse(msg));
        let parsed = parser.parse(msg);
        assert_eq!(parsed.file_path, "/srv/api/main.py");
        assert_eq!(parsed.line_number, 34);
    }

    #[test]
    fn test_composite_parser() {
        let composite = CompositeLogParser::new(vec![
            Box::new(PhpLogParser),
            Box::new(NodeLogParser),
            Box::new(PythonLogParser),
        ]);

        let msg = "PHP Fatal error: syntax error in /app/index.php on line 5";
        let parsed = composite.parse(msg);
        assert_eq!(parsed.file_path, "/app/index.php");
        assert_eq!(parsed.line_number, 5);
    }

    #[test]
    fn test_deduplicator() {
        let dedup = InMemoryDeduplicator::new();
        let sig = "app1:file.php:12:error";
        
        assert!(!dedup.is_duplicate(sig, 60));
        dedup.record(sig);
        assert!(dedup.is_duplicate(sig, 60));
    }

    #[test]
    fn test_orchestrator_pipeline() {
        let composite = Arc::new(CompositeLogParser::new(vec![Box::new(PhpLogParser)]));
        let dedup = Arc::new(InMemoryDeduplicator::new());
        let registry = Arc::new(YamlAppRegistry::new().load_from_path("non_existent.yaml"));
        let dispatcher = Arc::new(WebhookDispatcher::new("".to_string(), None, 5));

        let orchestrator = IncidentOrchestrator::new(
            composite,
            dedup,
            registry,
            dispatcher,
            300,
        );

        let alert = json!({
            "stream_name": "coolify_apps",
            "alert_name": "php_fatal",
            "records": [
                {
                    "app_name": "toko-online",
                    "message": "PHP Fatal error: Uncaught Exception in /app/Checkout.php on line 88"
                }
            ]
        });

        // 1st time: should succeed
        let res = orchestrator.process_alert(&alert);
        assert!(res.is_some());
        let incident = res.unwrap();
        assert_eq!(incident.app_metadata.app_name, "toko-online");
        assert_eq!(incident.incident.line_number, 88);

        // 2nd time immediately: should be throttled
        let res2 = orchestrator.process_alert(&alert);
        assert!(res2.is_none());
    }
}
