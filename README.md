# 🔭 OpenObserve SRE Hub

[![Rust](https://img.shields.io/badge/Language-Rust_1.98+-orange.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![OpenObserve](https://img.shields.io/badge/Engine-OpenObserve-blue.svg)](https://openobserve.ai/)
[![Coolify Ready](https://img.shields.io/badge/Deploy-Coolify_Auto--Deploy-purple.svg)](#-deployment-on-coolify-via-github-app-auto-deploy)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Centralized Autonomous SRE Telemetry Gateway & Incident Shipper for Multi-App Stacks.**  
> Functions as a **centralized SRE radar and telemetry sensor** that collects logs across 20+ production applications in cPanel and Coolify via zero-touch log ingestion, detects production incidents in real time (< 15 seconds), packages rich execution context (*stack traces, surrounding breadcrumb logs, git commit metadata*), and dispatches standardized problem payloads to any **Autonomous Coding Agent** to close the automated self-healing loop.

---

## ⚡ Quickstart (Up & Running in 2 Minutes)

Get the complete SRE Hub running and tested locally or on your server in 3 simple steps:

### 1. Clone & Start the Stack
```bash
git clone https://github.com/ihkaru/openobserve-sre.git
cd openobserve-sre
docker compose up -d --build
```
* Both **OpenObserve** (Port `5080`) and the **Rust Context Shipper** (Port `8089`) are now live.
* Open your browser at `http://localhost:5080` (Default credentials from `docker-compose.yml`: Email `admin@yourdomain.com`, Password `ChangeThisPasswordSecure!`).

### 2. Verify Service Health
```bash
curl http://localhost:8089/healthz
```
*Expected response:*
```json
{"active_cached_incidents":0,"runtime":"rust","service":"openobserve-sre-shipper","status":"ok"}
```

### 3. Send a Mock Incident Alert (Smoke Test)
Simulate an exception from a monitored container to see the Shipper parse the stack trace, enforce deduplication, and dispatch to your agent:
```bash
curl -X POST http://localhost:8089/webhook/openobserve \
  -H "Content-Type: application/json" \
  -d '{
    "stream_name": "coolify_apps",
    "alert_name": "test_php_fatal",
    "records": [
      {
        "app_name": "toko-online-api",
        "message": "PHP Fatal error: Uncaught Error: Call to undefined method Order::calculate() in /var/www/html/app/Services/OrderService.php on line 84"
      }
    ]
  }'
```
*Expected response:*
```json
{"app_name":"toko-online-api","event_id":"evt_1727678900_a1b2c3","status":"dispatched"}
```
> [!TIP]
> **Circuit Breaker Test:** Execute the exact same `curl` command again. The Shipper will immediately return `{"status":"throttled_or_empty"}` — protecting your coding agent from repeating error storms and token burnout!

---

## 📑 Table of Contents
- [⚡ Quickstart (Up & Running in 2 Minutes)](#-quickstart-up--running-in-2-minutes)
- [Why This Project Exists (The Problem)](#-why-this-project-exists-the-problem)
- [System Architecture (Event-Driven Flow)](#-system-architecture-event-driven-flow)
- [Core Features](#-core-features)
- [📦 Zero-Touch Log Collection (Coolify & cPanel)](#-zero-touch-log-collection-coolify--cpanel)
- [⚙️ Application Registry & Convention over Configuration](#️-application-registry--convention-over-configuration)
- [📡 Coding Agent Webhook Contract Specification](#-coding-agent-webhook-contract-specification)
- [🤖 Setting Up Your Coding Agent Environment](#-setting-up-your-coding-agent-environment)
- [🚀 Deployment on Coolify via GitHub App Auto-Deploy](#-deployment-on-coolify-via-github-app-auto-deploy)
- [🧪 Testing & Verification](#-testing--verification)
- [📚 Documentation Reference](#-documentation-reference)

---

## 🎯 Why This Project Exists (The Problem)

For solo developers and small engineering teams maintaining 20+ applications in production:
* **Reactive & Stressful:** End users frequently encounter bugs before maintainers do, reporting outages via direct messages and support channels.
* **Context Fatigue:** Manually tracing logs across dozens of disparate VPS servers, cPanel accounts, and Docker hosts consumes hours of firefighting time.
* **Prohibitive Observability Costs:** Running Datadog or New Relic across 20+ services quickly becomes cost-prohibitive, while traditional self-hosted ELK stacks demand massive RAM resources.

### 💡 The Solution: OpenObserve + Rust Shipper + Coding Agent
1. **OpenObserve:** A Rust-based, petabyte-scale observability engine that requires under 200 MB of RAM and delivers up to 140x lower storage costs compared to Elasticsearch.
2. **Rust Context Shipper (`shipper/`):** A sub-10ms microservice that deduplicates recurring errors, extracts failing source lines, and constructs standardized incident payloads.
3. **Autonomous Coding Agent (Agent-Agnostic):** Any coding agent (Claude Code, OpenHands, custom runners, or CI pipelines) ingests the standardized webhook, reproduces the issue in an isolated branch, verifies fixes with test suites, and opens a GitHub Pull Request.

---

## 📐 System Architecture (Event-Driven Flow)

This architecture maintains a strict separation of concerns between the **Sensor** (this repository) and the **Actor** (the external Coding Agent):

```mermaid
flowchart TD
    subgraph Apps ["20+ Production Applications (Zero-Touch Ingestion)"]
        CP[cPanel Shared / VPS<br/>PHP / Node.js / Python / error_log]
        CL[Coolify Docker Containers<br/>stdout / stderr]
    end

    subgraph Hub ["openobserve-sre (The Sensor)"]
        V[Vector Log Collectors] --> OO[(OpenObserve Engine)]
        OO -->|SQL Stream Alert < 15s| RS[Rust Context Shipper<br/>(Axum + Tokio)]
        RS -->|Zero-Config / apps.d/| REG[App Registry]
        RS -->|Deduplication Cache 30m| DEDUP{Duplicate?}
        DEDUP -->|Yes| DROP[Drop / Throttle]
        DEDUP -->|No| PACK[Pack Standard Context JSON]
    end

    subgraph Actor ["Agent-Agnostic Consumers (The Mechanic)"]
        PACK -->|POST Standard JSON| AGENT[Autonomous Coding Agent<br/>(Webhook Receiver / Sandbox)]
    end

    subgraph Resolution ["Resolution & Deployment"]
        AGENT -->|Create Hotfix Branch & PR| GH[GitHub / GitLab]
        AGENT -->|Interactive Notification| WA[Maintainer (WhatsApp / Telegram)]
        WA -->|Tap 'Approve'| DEP[Coolify Webhook / cPanel Git Hook]
    end

    Apps --> V
```

---

## ✨ Core Features

* **⚡ Real-Time Detection (< 15 Seconds):** Leverages OpenObserve SQL Stream Alerting to catch fatal errors and unhandled exceptions instantly.
* **🛡️ Automatic Circuit Breaker & Deduplication:** Prevents agent invocation storms and token burnout when dozens of users encounter the same exception within a 30-minute window.
* **🔌 100% Agent-Agnostic:** Emits a standardized JSON contract over HTTP POST. Compatible with any autonomous agent runner without vendor lock-in.
* **⚙️ Zero-Config Default:** Automatically derives GitHub repositories, languages, and test runners from container names and stack traces without requiring configuration files.
* **🚀 Zero-Touch Log Collection:** Stream logs directly from Coolify Docker sockets and cPanel `error_log` files without modifying target application source code.

---

## 📦 Zero-Touch Log Collection (Coolify & cPanel)

You do **NOT** need to install SDKs or modify target application code.

### A. Applications on Coolify (Docker Containers)
Run a Vector container on your Coolify host using [`collectors/vector-coolify.yaml`](collectors/vector-coolify.yaml):
```bash
docker run -d \
  --name vector-coolify \
  --restart unless-stopped \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  -v $(pwd)/collectors/vector-coolify.yaml:/etc/vector/vector.yaml:ro \
  timberio/vector:latest-alpine --config /etc/vector/vector.yaml
```

### B. Applications on cPanel (Shared / VPS)
Run a standalone Vector binary under your cPanel account using [`collectors/vector-cpanel.yaml`](collectors/vector-cpanel.yaml):
```bash
vector --config collectors/vector-cpanel.yaml &
```

---

## ⚙️ Application Registry & Convention over Configuration

The SRE Hub adopts a **Zero-Config Default** philosophy:
* When an incoming exception from service `pos-kasir` is received without an explicit configuration file, the Hub dynamically derives:
  - **Repository URL:** `https://github.com/<DEFAULT_GITHUB_ORG>/<app_name>` (organization configurable via `DEFAULT_GITHUB_ORG`, defaults to `ihkaru`).
  - **Default Branch:** `main`.
  - **Language & Test Command:** Inferred automatically from the parsed stack trace:
    - PHP Fatal Errors / Exceptions → `php` (`php artisan test`)
    - Python Exceptions → `python` (`pytest`)
    - Go Runtime Panics → `go` (`go test ./...`)
    - Node / TypeScript Exceptions → `nodejs` (`npm test`)

### Custom Application Overrides (`apps.d/`)
For non-standard repositories (such as monorepos, specific release branches, or custom test runners), you can define an optional standalone YAML file in [`apps.d/`](apps.d/):

```yaml
# apps.d/billing-service.yaml
app_name: billing-service
environment: production
platform: coolify
language: python
framework: fastapi
repo_url: https://github.com/ihkaru/billing-service
default_branch: main
verification_command: pytest tests/unit/
```

Files placed in `apps.d/` are loaded dynamically on startup and take precedence over default conventions.

---

## 📡 Coding Agent Webhook Contract Specification

The Rust Shipper delivers an HTTP `POST` to `AGENT_TARGET_URL` with a context-rich JSON payload:

```json
{
  "event_id": "evt_1727678900_a1b2c3",
  "timestamp": "2026-09-30T07:00:00Z",
  "app_metadata": {
    "app_name": "toko-online-api",
    "environment": "production",
    "platform": "cpanel",
    "language": "php",
    "framework": "laravel",
    "repository": {
      "provider": "github",
      "url": "https://github.com/ihkaru/toko-online-api",
      "default_branch": "main",
      "target_branch": "hotfix/auto-heal-evt_1727678900_a1b2c3"
    }
  },
  "incident": {
    "error_type": "PHP Fatal Error",
    "error_message": "Uncaught Error: Call to undefined method Order::calculate()",
    "file_path": "app/Services/OrderService.php",
    "line_number": 84,
    "stack_trace": ["..."]
  },
  "telemetry_context": {
    "openobserve_stream": "coolify_apps",
    "trigger_alert_name": "php_fatal_errors",
    "surrounding_logs": ["..."]
  },
  "remediation_instructions": {
    "objective": "Resolve PHP Fatal Error in app/Services/OrderService.php:84",
    "verification_command": "php artisan test"
  }
}
```

---

## 🤖 Setting Up Your Coding Agent Environment

Detailed technical specifications covering ephemeral workspace sandboxing, job queues, test execution security, and 1-tap messaging approval gates are documented in:

👉 **[docs/AGENT_ENVIRONMENT_SETUP.md](docs/AGENT_ENVIRONMENT_SETUP.md)**

---

## 🚀 Deployment on Coolify via GitHub App Auto-Deploy

This repository is configured for native deployment on Coolify using the official GitHub App integration with continuous delivery on pushes to `main`.

### Coolify Setup Steps:
1. **Create New Resource in Coolify:**
   * Navigate to your Coolify project dashboard → Click **+ New Resource**.
   * Select **Docker Compose Application**.
   * Choose your **GitHub App** integration → Select `ihkaru/openobserve-sre` with branch `main`.
2. **Configure Environment Variables:**
   * Copy variables from [`.env.example`](.env.example) into Coolify's **Environment Variables** tab:
     - `ZO_ROOT_USER_EMAIL`: OpenObserve administrator email.
     - `ZO_ROOT_USER_PASSWORD`: OpenObserve administrator password.
     - `AGENT_TARGET_URL`: Webhook URL of your coding agent worker.
     - `DEFAULT_GITHUB_ORG`: Default GitHub username / organization (defaults to `ihkaru`).
3. **Deploy:**
   * Click **Deploy** in Coolify.
   * Coolify provisions OpenObserve with persistent named storage (`openobserve_data`) and builds the lightweight Rust Shipper image (~15 MB).
4. **Continuous Deployment Active:**
   * Any future `git push` triggers a seamless rolling update in Coolify with zero downtime!

---

## 🧪 Testing & Verification

The Rust Shipper includes a comprehensive test suite:

```bash
cd /home/server/projects/openobserve-sre/shipper
cargo test
```

Test coverage verifies:
* ✅ PHP Fatal Error extractor (`file_path`, `line_number`).
* ✅ Node.js / TypeScript stacktrace extractor.
* ✅ Python traceback extractor.
* ✅ Go runtime panic extractor (`main.go:line`).
* ✅ Composite parser delegation and fallback.
* ✅ In-memory circuit breaker deduplication and cooldown expiration.
* ✅ End-to-end Orchestrator processing.

---

## 📚 Documentation Reference
* **[docs/AGENT_ENVIRONMENT_SETUP.md](docs/AGENT_ENVIRONMENT_SETUP.md):** Best practices for agent runtime sandboxing, job queues, and approval gates.
* **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md):** System architecture, 2-Tier observability, and avoiding the Dead App Paradox.
* **[docs/CONTEXT_SPEC.md](docs/CONTEXT_SPEC.md):** Detailed JSON schema specification for incident payloads.
* **[docs/ROADMAP.md](docs/ROADMAP.md):** Implementation roadmap and milestone checklist.
* **[docs/sample-payload.json](docs/sample-payload.json):** Standalone sample JSON payload for local agent testing.
