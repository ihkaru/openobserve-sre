use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use serde::Deserialize;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use openobserve_shipper::api::build_router;
use openobserve_shipper::cache::InMemoryDeduplicator;
use openobserve_shipper::dispatchers::WebhookDispatcher;
use openobserve_shipper::orchestrator::IncidentOrchestrator;
use openobserve_shipper::parsers::{CompositeLogParser, GoLogParser, NodeLogParser, PhpLogParser, PythonLogParser};
use openobserve_shipper::registry::YamlAppRegistry;

#[derive(Debug, Deserialize)]
struct ServerConfig {
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct DedupConfig {
    cooldown_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct DispatchConfig {
    target_url: Option<String>,
    auth_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: Option<ServerConfig>,
    deduplication: Option<DedupConfig>,
    agent_dispatch: Option<DispatchConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting OpenObserve SRE Context Shipper (Rust Edition)");

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let apps_dir = env::var("APPS_DIR").unwrap_or_else(|_| "apps.d".to_string());

    let mut config_data = AppConfig {
        server: None,
        deduplication: None,
        agent_dispatch: None,
    };

    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(parsed) = serde_yaml::from_str::<AppConfig>(&content) {
            config_data = parsed;
        }
    }

    // 1. Parsers (Open/Closed Principle: PHP, Node, Python, Go)
    let composite_parser = Arc::new(CompositeLogParser::new(vec![
        Box::new(PhpLogParser),
        Box::new(NodeLogParser),
        Box::new(PythonLogParser),
        Box::new(GoLogParser),
    ]));

    // 2. Deduplicator (Liskov Substitution Principle)
    let deduplicator = Arc::new(InMemoryDeduplicator::new());

    // 3. App Registry (Supports single config.yaml + modular apps.d/ directory)
    let mut registry_builder = YamlAppRegistry::new().load_from_path(&config_path);
    if Path::new(&apps_dir).exists() {
        registry_builder = registry_builder.load_from_path(&apps_dir);
    }
    info!(registered_apps = registry_builder.count(), "Loaded application registry");
    let registry = Arc::new(registry_builder);

    // 4. Dispatcher (Interface Segregation Principle)
    let target_url = env::var("AGENT_TARGET_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| config_data.agent_dispatch.as_ref().and_then(|d| d.target_url.clone()))
        .unwrap_or_else(|| "http://localhost:8000/webhook/agent-remediation".to_string());

    let auth_token = env::var("AGENT_AUTH_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| config_data.agent_dispatch.as_ref().and_then(|d| d.auth_token.clone()));

    let dispatcher = Arc::new(WebhookDispatcher::new(target_url, auth_token, 15));

    // 5. Orchestrator (Dependency Inversion Principle)
    let cooldown_secs = env::var("DEDUP_COOLDOWN_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| config_data.deduplication.as_ref().and_then(|d| d.cooldown_seconds))
        .unwrap_or(1800);

    let orchestrator = Arc::new(IncidentOrchestrator::new(
        composite_parser,
        deduplicator,
        registry,
        dispatcher,
        cooldown_secs,
    ));

    // 6. Router and Web Server
    let router = build_router(orchestrator);

    let port = config_data
        .server
        .as_ref()
        .and_then(|s| s.port)
        .or_else(|| env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(8089);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(addr = %addr, "Listening for OpenObserve alerts");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
