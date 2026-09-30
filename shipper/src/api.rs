use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{info, warn};
use crate::orchestrator::IncidentOrchestrator;
use crate::parsers::SentryParser;

pub fn build_router(orchestrator: Arc<IncidentOrchestrator>) -> Router {
    Router::new()
        .route("/healthz", get(healthz_handler))
        .route("/webhook/openobserve", post(openobserve_webhook_handler))
        // Sentry SDK Ingest Protocol Endpoints (compatible with standard Sentry SDKs)
        .route("/api/:project_id/envelope", post(sentry_envelope_handler).options(sentry_options_handler))
        .route("/api/:project_id/envelope/", post(sentry_envelope_handler).options(sentry_options_handler))
        .route("/api/:project_id/store", post(sentry_store_handler).options(sentry_options_handler))
        .route("/api/:project_id/store/", post(sentry_store_handler).options(sentry_options_handler))
        .route("/api/:project_id/minidump", post(sentry_noop_handler).options(sentry_options_handler))
        .route("/api/:project_id/minidump/", post(sentry_noop_handler).options(sentry_options_handler))
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

/// Handler for Sentry Envelope payloads (modern Sentry SDKs v7+)
async fn sentry_envelope_handler(
    State(orchestrator): State<Arc<IncidentOrchestrator>>,
    Path(project_id): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    match SentryParser::parse_envelope(&body, &project_id) {
        Ok(sentry_event) => {
            let event_id = sentry_event.event_id.clone();
            let app_name = sentry_event.app_name.clone();
            let raw_json = sentry_event.raw_json.clone();

            info!(
                event_id = %event_id,
                app_name = %app_name,
                "Sentry envelope accepted from SDK"
            );

            // 1. Process incident and dispatch to agent
            if let Some(incident) = orchestrator.process_sentry_event(sentry_event) {
                let orch = Arc::clone(&orchestrator);
                tokio::spawn(async move {
                    orch.dispatch(&incident).await;
                });
            }

            // 2. Forward raw event to OpenObserve asynchronously
            let orch_oo = Arc::clone(&orchestrator);
            tokio::spawn(async move {
                orch_oo.forward_to_openobserve(&raw_json).await;
            });

            (
                StatusCode::OK,
                [("Content-Type", "application/json"), ("Access-Control-Allow-Origin", "*")],
                Json(json!({ "id": event_id })),
            )
        }
        Err(e) => {
            warn!(error = %e, "Non-error or unparseable Sentry envelope item acknowledged");
            (
                StatusCode::OK,
                [("Content-Type", "application/json"), ("Access-Control-Allow-Origin", "*")],
                Json(json!({ "id": "acknowledged" })),
            )
        }
    }
}

/// Handler for Sentry Store payloads (JSON)
async fn sentry_store_handler(
    State(orchestrator): State<Arc<IncidentOrchestrator>>,
    Path(project_id): Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match SentryParser::parse_store(&payload, &project_id) {
        Ok(sentry_event) => {
            let event_id = sentry_event.event_id.clone();
            let app_name = sentry_event.app_name.clone();
            let raw_json = sentry_event.raw_json.clone();

            info!(
                event_id = %event_id,
                app_name = %app_name,
                "Sentry store payload accepted"
            );

            if let Some(incident) = orchestrator.process_sentry_event(sentry_event) {
                let orch = Arc::clone(&orchestrator);
                tokio::spawn(async move {
                    orch.dispatch(&incident).await;
                });
            }

            let orch_oo = Arc::clone(&orchestrator);
            tokio::spawn(async move {
                orch_oo.forward_to_openobserve(&raw_json).await;
            });

            (
                StatusCode::OK,
                [("Content-Type", "application/json"), ("Access-Control-Allow-Origin", "*")],
                Json(json!({ "id": event_id })),
            )
        }
        Err(e) => {
            warn!(error = %e, "Failed to parse Sentry store event");
            (
                StatusCode::OK,
                [("Content-Type", "application/json"), ("Access-Control-Allow-Origin", "*")],
                Json(json!({ "id": "ignored" })),
            )
        }
    }
}

async fn sentry_noop_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "application/json"), ("Access-Control-Allow-Origin", "*")],
        Json(json!({ "status": "ok" })),
    )
}

async fn sentry_options_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "POST, GET, OPTIONS"),
            ("Access-Control-Allow-Headers", "X-Sentry-Auth, Content-Type, Authorization"),
        ],
        "",
    )
}
