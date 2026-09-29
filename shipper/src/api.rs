use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::info;
use crate::orchestrator::IncidentOrchestrator;

pub fn build_router(orchestrator: Arc<IncidentOrchestrator>) -> Router {
    Router::new()
        .route("/healthz", get(healthz_handler))
        .route("/webhook/openobserve", post(openobserve_webhook_handler))
        .with_state(orchestrator)
}

async fn healthz_handler(
    State(orchestrator): State<Arc<IncidentOrchestrator>>,
) -> impl IntoResponse {
    let active_cached = orchestrator.deduplicator().count_active();
    Json(json!({
        "status": "ok",
        "service": "openobserve-sre-shipper",
        "runtime": "rust",
        "active_cached_incidents": active_cached
    }))
}

async fn openobserve_webhook_handler(
    State(orchestrator): State<Arc<IncidentOrchestrator>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match orchestrator.process_alert(&payload) {
        Some(incident) => {
            let event_id = incident.event_id.clone();
            let app_name = incident.app_metadata.app_name.clone();

            info!(
                event_id = %event_id,
                app_name = %app_name,
                "Incident accepted and queued for agent dispatch"
            );

            // Spawn dispatch asynchronously so OpenObserve webhook receives fast ACK
            let orch = Arc::clone(&orchestrator);
            tokio::spawn(async move {
                orch.dispatch(&incident).await;
            });

            (
                StatusCode::ACCEPTED,
                Json(json!({
                    "status": "dispatched",
                    "event_id": event_id,
                    "app_name": app_name
                })),
            )
        }
        None => (
            StatusCode::OK,
            Json(json!({
                "status": "throttled_or_empty",
                "message": "Incident is either empty or within the active cooldown period."
            })),
        ),
    }
}
