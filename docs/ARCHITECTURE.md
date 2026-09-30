# 📐 System Architecture: OpenObserve SRE Hub (Agent-Agnostic)

This document outlines the technical architecture of the **`openobserve-sre`** repository as a centralized *Observability Hub & Telemetry Gateway*. It details the log ingestion strategy for 20+ applications across cPanel and Coolify, the *Agent-Agnostic* design principles, and the data contract for external **Coding Agent APIs**.

---

## 1. Core Philosophy: Sensor vs. Actor Separation

A frequent failure mode in automated self-healing systems is tight coupling between the monitoring infrastructure and the specific code repair agent. When models or agent frameworks change, the entire monitoring setup must be reworked.

In this architecture:
* **`openobserve-sre` acts as the SENSOR & RADAR:**
  * Collects telemetry (logs, metrics, traces) across 20+ heterogeneous applications via zero-touch log ingestion.
  * Detects anomalies and fatal production errors using real-time SQL stream evaluation.
  * Enforces **Deduplication & Circuit Breaking** to prevent agent invocation storms and token burnout during repeating error spikes.
  * Packages rich incident context (*stack traces, breadcrumb logs, source file paths, target repository, verification commands*) into a **Standard Problem Payload**.
  * Optionally exposes an **OpenObserve MCP (Model Context Protocol)** interface for interactive log exploration.
* **The Autonomous Coding Agent acts as the ACTOR (Independent Mechanic):**
  * Consumes the standardized JSON payload via an HTTP POST webhook.
  * Performs root cause analysis, checks out an isolated hotfix branch, reproduces the defect, implements a patch, and opens a GitHub Pull Request.
  * Requests human-in-the-loop release approval via messaging channels before triggering production deployment.

```mermaid
flowchart TD
    subgraph TargetApps ["20+ Production Applications (Zero-Touch)"]
        CP[cPanel Shared / VPS<br/>PHP / Node.js / Python / error_log]
        CL[Coolify Docker Containers<br/>stdout / stderr / Docker Sock]
    end

    subgraph Hub ["openobserve-sre (Observability & Context Gateway)"]
        direction TB
        V[Vector Collectors] --> O2[(OpenObserve Engine)]
        O2 --> SQL[SQL Alert Engine<br/>Detect 500 / Fatal in < 15s]
        SQL --> Shipper[Rust Context Shipper<br/>(Axum + Tokio)]
        Shipper --> AppReg[apps.d/ Modular Registry]
    end

    subgraph Consumers ["Agent-Agnostic Consumers"]
        direction TB
        Shipper -->|Standard JSON Payload| AGENT[Autonomous Coding Agent<br/>(Webhook Receiver / Sandbox)]
        Shipper -.->|Plug & Play| OTHERS[Claude Code / OpenHands / Cursor / CI]
    end

    subgraph Remediation ["Resolution & Deployment"]
        AGENT -->|Open Pull Request| REPO[GitHub / GitLab 20+ Repos]
        AGENT -->|Interactive Notification & 1-Tap Deploy| WA[Maintainer (WhatsApp / Telegram)]
        WA -->|Tap 'Approve'| DEP[Coolify API Webhook / cPanel Git Hook]
    end

    TargetApps --> V
```

---

## 2. Does Target Application Observability Need Internal Endpoints?

Architectural question: **"Must the maintained applications feature extensive built-in observability so the coding agent can query internal application endpoints for additional context?"**

### Recommendation: "2-Tier Observability" (Avoiding the Dead App Paradox)

1. **Avoid Relying on Internal HTTP Application Endpoints for Incident Triage:**
   * **The Dead App Paradox:** When an application experiences a fatal crash, memory exhaustion, or container crash loop (500 / 502 Bad Gateway), its HTTP server is **already unresponsive**. If an agent attempts to query `GET https://app.com/debug-context`, the request will time out or return a 502 error, depriving the agent of context exactly when it is most needed.
   * **Security Surface:** Exposing endpoints that leak internal runtime state, environment variables, or database structures in production creates serious security risks if authorization headers are misconfigured.
   * **Adoption Friction:** Instrumenting custom diagnostic endpoints across 20+ legacy services in multiple programming languages creates unnecessary operational overhead.

2. **Tier 1 (Non-Invasive Outer Context - Mandatory & Zero Code Changes):**
   * Context is captured entirely from the outside:
     - **Stack Traces & Breadcrumbs:** Captured by OpenObserve from Docker stdout/stderr streams and cPanel `error_log` files.
     - **Source Code State:** Fetched directly from the GitHub API using the target branch or production commit SHA.
     - **Container State:** Extracted from Coolify / Docker daemon metadata (OOMKilled flag, exit codes, restart counts).
   * *Result:* **85–90% of production defects (null pointer exceptions, undefined array keys, syntax errors, uncaught exceptions) are completely resolvable by coding agents using Tier 1 context alone.**

3. **Tier 2 (Out-of-Band Introspection - On Demand Only):**
   * If an agent legitimately requires runtime state inspection (e.g. database migration status), execute secure CLI commands directly inside the container via SSH/Docker exec (e.g. `docker exec <container> php artisan migrate:status`), **never through a public HTTP endpoint**.

---

## 3. Core Components of `openobserve-sre`

### A. OpenObserve Engine (`docker-compose.yml`)
* High-performance, Rust-based engine operating with minimal memory footprint (< 200 MB RAM for dozens of services).
* Stores columnar log data in Parquet format backed by persistent named volumes (`openobserve_data`).
* Provides a centralized search UI across all monitored services.

### B. Vector Collectors (`collectors/`)
* **Coolify Collector (`vector-coolify.yaml`):** Monitors `/var/run/docker.sock`, extracts container names into `app_name` tags, and streams logs to OpenObserve via HTTP.
* **cPanel Collector (`vector-cpanel.yaml`):** Non-intrusively monitors `~/public_html/error_log` and web server error logs.

### C. Context Shipper & Deduplicator (`shipper/`)
* Built in **Rust** using Axum and Tokio for high throughput and sub-millisecond dispatching.
* Ingests real-time alerts from OpenObserve.
* Computes deduplication hashes: `hash(app_name + error_file + error_line + error_type)`.
* Enforces a **30-minute cooldown** to suppress redundant agent invocations.
* Dynamically scans application configurations from `apps.d/*.yaml`.
* Dispatches standardized problem payloads to the configured coding agent webhook.

---

## 4. Key Benefits of an Agent-Agnostic Design

* **Zero Vendor Lock-In:** The observability pipeline remains independent of any specific LLM provider or coding agent framework.
* **Standardized Data Contract:** Consuming agents only need to handle a single, well-defined JSON schema (specified in [docs/CONTEXT_SPEC.md](CONTEXT_SPEC.md)).
* **Plug & Play Extensibility:** Switch between local runners, cloud-hosted agents, or CI-based workflows simply by updating the `AGENT_TARGET_URL` environment variable.
