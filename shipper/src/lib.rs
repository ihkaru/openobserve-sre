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

    #[test]
    fn test_sentry_envelope_parsing() {
        use parsers::SentryParser;

        let envelope = concat!(
            "{\"event_id\":\"fc6d8c0c43fc4630ad850ee518f1b9d0\",\"sent_at\":\"2026-09-30T00:00:00Z\"}\n",
            "{\"type\":\"event\",\"content_type\":\"application/json\"}\n",
            "{\"event_id\":\"fc6d8c0c43fc4630ad850ee518f1b9d0\",\"platform\":\"php\",\"environment\":\"production\",",
            "\"tags\":{\"app_name\":\"billing-service\"},",
            "\"exception\":{\"values\":[{",
            "\"type\":\"PaymentFailedException\",",
            "\"value\":\"Card was declined by bank\",",
            "\"stacktrace\":{\"frames\":[",
            "{\"filename\":\"app/Http/Controllers/PayController.php\",\"lineno\":10,\"function\":\"init\"},",
            "{\"filename\":\"app/Services/StripeGateway.php\",\"lineno\":55,\"function\":\"charge\",",
            "\"in_app\":true,\"context_line\":\"        throw new PaymentFailedException($msg);\",",
            "\"pre_context\":[\"    public function charge() {\"],",
            "\"post_context\":[\"    }\"],",
            "\"vars\":{\"$amount\":150000,\"$currency\":\"IDR\"}}",
            "]}}]},",
            "\"request\":{\"url\":\"https://api.domain.com/v1/charge\",\"method\":\"POST\",",
            "\"data\":{\"order_id\":\"ord_999\",\"amount\":150000},\"query_string\":\"source=checkout\"},",
            "\"breadcrumbs\":{\"values\":[",
            "{\"category\":\"query\",\"message\":\"SELECT * FROM wallets WHERE user_id = 42\",\"level\":\"info\"},",
            "{\"category\":\"http\",\"message\":\"POST https://api.stripe.com/v1/charges 402\",\"level\":\"warning\"}",
            "]}}\n"
        );

        let parsed = SentryParser::parse_envelope(envelope.as_bytes(), "billing-service").unwrap();

        assert_eq!(parsed.event_id, "fc6d8c0c43fc4630ad850ee518f1b9d0");
        assert_eq!(parsed.app_name, "billing-service");
        assert_eq!(parsed.error_type, "PaymentFailedException");
        assert_eq!(parsed.error_message, "Card was declined by bank");
        assert_eq!(parsed.file_path, "app/Services/StripeGateway.php");
        assert_eq!(parsed.line_number, 55);

        // Check rich source context
        assert!(parsed.source_context.is_some());
        let src = parsed.source_context.unwrap();
        assert_eq!(src.context_line.as_deref(), Some("        throw new PaymentFailedException($msg);"));
        assert_eq!(src.pre_context.len(), 1);

        // Check local variables
        assert!(parsed.local_variables.is_some());
        let vars = parsed.local_variables.unwrap();
        assert_eq!(vars.get("$amount").and_then(|v| v.as_i64()), Some(150000));

        // Check HTTP request details
        assert!(parsed.http_request.is_some());
        let req = parsed.http_request.unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.domain.com/v1/charge");
        assert!(req.body.is_some());
        assert_eq!(req.body.unwrap().get("order_id").and_then(|v| v.as_str()), Some("ord_999"));

        // Check Breadcrumbs
        assert_eq!(parsed.breadcrumbs.len(), 2);
        assert_eq!(parsed.breadcrumbs[0].category.as_deref(), Some("query"));
        assert_eq!(parsed.breadcrumbs[0].message.as_deref(), Some("SELECT * FROM wallets WHERE user_id = 42"));
    }

    #[test]
    fn test_sentry_orchestrator_pipeline() {
        use parsers::SentryParser;

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

        let sentry_json = json!({
            "event_id": "test_sentry_123",
            "platform": "python",
            "environment": "production",
            "tags": { "app_name": "ai-worker" },
            "exception": {
                "values": [{
                    "type": "ZeroDivisionError",
                    "value": "division by zero",
                    "stacktrace": {
                        "frames": [{
                            "filename": "app/calculator.py",
                            "lineno": 42,
                            "function": "divide",
                            "in_app": true
                        }]
                    }
                }]
            },
            "request": {
                "url": "http://localhost:8000/api/calc",
                "method": "POST",
                "data": { "a": 10, "b": 0 }
            }
        });

        let parsed_event = SentryParser::parse_store(&sentry_json, "ai-worker").unwrap();

        // 1st time: should succeed and create rich incident context
        let incident_opt = orchestrator.process_sentry_event(parsed_event.clone());
        assert!(incident_opt.is_some());
        let incident = incident_opt.unwrap();

        assert_eq!(incident.event_id, "test_sentry_123");
        assert_eq!(incident.app_metadata.app_name, "ai-worker");
        assert_eq!(incident.app_metadata.language, "python");
        assert_eq!(incident.remediation_instructions.verification_command, "pytest");
        assert_eq!(incident.incident.file_path, "app/calculator.py");
        assert_eq!(incident.incident.line_number, 42);

        // Verify rich diagnostics
        assert!(incident.diagnostics.is_some());
        let diag = incident.diagnostics.unwrap();
        assert!(diag.http_request.is_some());
        let req = diag.http_request.unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.body.unwrap().get("b").and_then(|v| v.as_i64()), Some(0));

        // 2nd time: should be throttled by circuit breaker
        let incident_throttled = orchestrator.process_sentry_event(parsed_event);
        assert!(incident_throttled.is_none());
    }
}
