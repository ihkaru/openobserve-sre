# 🔭 OpenObserve SRE Hub

[![Rust](https://img.shields.io/badge/Language-Rust_1.98+-orange.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![OpenObserve](https://img.shields.io/badge/Engine-OpenObserve-blue.svg)](https://openobserve.ai/)
[![Coolify Ready](https://img.shields.io/badge/Deploy-Coolify_Auto--Deploy-purple.svg)](#-deployment-on-coolify-via-github-app-auto-deploy)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Centralized Autonomous SRE Telemetry Gateway & Incident Shipper for Multi-App Stacks.**  
> Functions as a **centralized SRE radar and telemetry sensor** that collects logs across 20+ production applications in cPanel and Coolify via zero-touch log ingestion, detects production incidents in real time (< 15 seconds), packages rich execution context (*stack traces, surrounding breadcrumb logs, git commit metadata*), and dispatches standardized problem payloads to any **Autonomous Coding Agent** to close the automated self-healing loop.

---

## ⚡ Getting Started (The Best-Practice 2-Tier Pattern)

The recommended production architecture operates as an autonomous **2-Tier SRE Sensor**:
* **Tier 2 (Deep In-App Context & SSOT - Primary):** Monitored applications use standard official Sentry SDKs with self-describing repository tags in their `.env`. When an error occurs, the exact HTTP request body, preceding SQL query breadcrumbs, and exact client GitHub repository URL are captured.
* **Tier 1 (Outer Black-Box Safety Net):** Vector streams container stdout/stderr from Docker (`docker.sock`) and cPanel `error_log` to catch fatal server crashes, startup errors, and OOM kills (mitigating the *Dead-App Paradox*).

---

### Step 1: Start the SRE Hub (2 Minutes)

Launch OpenObserve and the Rust Context Shipper locally or on your server:

```bash
git clone https://github.com/ihkaru/openobserve-sre.git
cd openobserve-sre
docker compose up -d --build
```
* **OpenObserve Web UI:** `http://localhost:5080` (Default credentials: `admin@yourdomain.com` / `ChangeThisPasswordSecure!`)
* **Rust Shipper API:** `http://localhost:8089` (Verify with `curl http://localhost:8089/healthz`)

---

### Step 2: Connect Your Monitored Applications (The 1-Line SSOT Pattern)

In your production applications (PHP Laravel, Node.js, Python, Go), install the official standard Sentry SDK. To support **different GitHub accounts or client organizations**, declare the repository directly in the application's environment (**Single Source of Truth**):

#### In Your Application `.env`:
```env
# 1. Point DSN to your SRE Shipper instance
SENTRY_DSN=http://public@sre.yourdomain.com:8089/toko-online-api

# 2. SSOT: Declare the exact GitHub account and test command (Zero Guessing!)
SENTRY_TAGS_REPO_URL=https://github.com/client-xyz/toko-online-api
SENTRY_TAGS_BRANCH=main
SENTRY_TAGS_VERIFICATION_COMMAND=php artisan test
```

#### Multi-Language Quick Setup (Zero Hub Maintenance):
* **Laravel (PHP):**
  ```bash
  composer require sentry/sentry-laravel
  ```
* **Node.js (Express / NestJS):**
  ```typescript
  import * as Sentry from "@sentry/node";
  Sentry.init({ dsn: process.env.SENTRY_DSN });
  ```
* **Python (FastAPI / Django):**
  ```python
  import sentry_sdk
  sentry_sdk.init(dsn=os.getenv("SENTRY_DSN"))
  ```

---

### 🤖 Official AI Agent Skill & Runbook (`SKILL.md`)

This repository ships with an official **Antigravity / Autonomous Coding Agent Skill** located at [`skills/sre-app-onboarding/SKILL.md`](skills/sre-app-onboarding/SKILL.md). Any AI coding assistant working on your client services can load this skill to automatically instrument Sentry, set self-describing telemetry tags, and verify incident dispatch:

* **Skill Name:** `sre-app-onboarding`
* **Runbook:** [`skills/sre-app-onboarding/SKILL.md`](skills/sre-app-onboarding/SKILL.md)
* **Multi-Language Examples:**
  - Laravel / PHP: [`skills/sre-app-onboarding/examples/laravel.php`](skills/sre-app-onboarding/examples/laravel.php)
  - Node.js / Express / TypeScript: [`skills/sre-app-onboarding/examples/express.ts`](skills/sre-app-onboarding/examples/express.ts)
  - Python / FastAPI: [`skills/sre-app-onboarding/examples/fastapi.py`](skills/sre-app-onboarding/examples/fastapi.py)
  - Go: [`skills/sre-app-onboarding/examples/golang.go`](skills/sre-app-onboarding/examples/golang.go)
* **Telemetry Specification:** [`skills/sre-app-onboarding/references/telemetry-schema.md`](skills/sre-app-onboarding/references/telemetry-schema.md)

---

### Step 3: Configure Your Autonomous Coding Agent Webhook

In `docker-compose.yml` (or via Coolify environment variables), set the webhook endpoint where your autonomous coding agent or CI runner receives tasks:

```env
AGENT_TARGET_URL=https://agent.yourdomain.com/webhook/remediation
AGENT_AUTH_TOKEN=your_secure_bearer_token
```

---

### Step 4: Verify End-to-End with a Smoke Test

Simulate a production exception containing an HTTP request body, SQL breadcrumbs, and client repository metadata:

```bash
curl -X POST http://localhost:8089/api/toko-online-api/store \
  -H "Content-Type: application/json" \
  -d '{
    "event_id": "smoke_test_001",
    "platform": "php",
    "environment": "production",
    "tags": {
      "app_name": "toko-online-api",
      "repo_url": "https://github.com/client-xyz/toko-online-api",
      "branch": "main",
      "verification_command": "php artisan test --filter=OrderTest"
    },
    "exception": {
      "values": [{
        "type": "PaymentFailedException",
        "value": "Card declined: insufficient funds",
        "stacktrace": {
          "frames": [{
            "filename": "app/Services/PaymentService.php",
            "lineno": 88,
            "function": "charge",
            "in_app": true,
            "context_line": "        throw new PaymentFailedException($res->message);",
            "pre_context": ["    public function charge($order) {"],
            "post_context": ["    }"]
          }]
        }
      }]
    },
    "request": {
      "url": "https://toko-online.com/api/checkout",
      "method": "POST",
      "data": { "item_id": 42, "qty": 2, "payment_method": "credit_card" }
    },
    "breadcrumbs": {
      "values": [
        { "category": "query", "message": "SELECT * FROM orders WHERE id = 42" },
        { "category": "http", "message": "POST https://api.stripe.com/v1/charges 402" }
      ]
    }
  }'
```

* **Expected response:**
  ```json
  {"id":"smoke_test_001"}
  ```
* **What Happens Instantly:**
  1. The event is stored in OpenObserve stream `sentry_events` for log search, dashboards, and historical queries.
  2. The Shipper compiles the rich problem payload containing the exact request body, SQL breadcrumbs, source lines, and repository `https://github.com/client-xyz/toko-online-api`.
  3. The payload is dispatched to your Coding Agent in **< 10 milliseconds**.
  4. The Coding Agent generates a reproduction unit test, implements the fix, and opens a Pull Request!

> [!TIP]
> **Circuit Breaker Test:** Execute the exact same `curl` command again. The Shipper automatically suppresses the duplicate event for 30 minutes, preventing agent invocation storms and token burnout!

---

## 📑 Table of Contents
- [⚡ Getting Started (The Best-Practice 2-Tier Pattern)](#-getting-started-the-best-practice-2-tier-pattern)
- [Why This Project Exists (The Problem)](#-why-this-project-exists-the-problem)
- [System Architecture (Event-Driven Flow)](#-system-architecture-event-driven-flow)
- [Core Features](#-core-features)
- [📦 Zero-Touch Log Collection (Coolify & cPanel)](#-zero-touch-log-collection-coolify--cpanel)
- [🩺 Deep In-App Error Context (Standard Sentry SDKs)](#-deep-in-app-error-context-standard-sentry-sdks)
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
    subgraph TargetApps ["20+ Production Applications (Multi-Account)"]
        CP[cPanel / Coolify Containers<br/>stdout / stderr / Docker Sock<br/>(Tier 1: Non-Intrusive Safety Net)]
        SentryApps[Standard Sentry SDKs in Apps<br/>SENTRY_TAGS_REPO_URL=...<br/>(Tier 2: Deep Context & SSOT)]
    end

    subgraph Hub ["openobserve-sre (The Sensor)"]
        direction TB
        V[Vector Collectors] --> OO[(OpenObserve Engine)]
        OO -->|SQL Stream Alert < 15s| RS[Rust Context Shipper<br/>(Axum + Tokio)]
        SentryApps -->|POST /api/:project_id/envelope| RS
        RS -->|Forward JSON Events| OO
        RS -->|Deduplication Cache 30m| DEDUP{Duplicate?}
        DEDUP -->|Yes| DROP[Drop / Throttle]
        DEDUP -->|No| PACK[Pack Rich Context JSON]
    end

    subgraph Actor ["Agent-Agnostic Consumers (The Mechanic)"]
        PACK -->|POST Standard Rich JSON| AGENT[Autonomous Coding Agent<br/>(Webhook Receiver / Sandbox)]
    end

    subgraph Resolution ["Resolution & Deployment"]
        AGENT -->|Create Hotfix Branch & PR| GH[GitHub / GitLab Multi-Account Repos]
        AGENT -->|Interactive Notification| WA[Maintainer (WhatsApp / Telegram)]
        WA -->|Tap 'Approve'| DEP[Coolify Webhook / cPanel Git Hook]
    end

    CP --> V
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

## 🩺 Deep In-App Error Context (Standard Sentry SDKs)

While outer log scraping (via Vector) provides an essential safety net for server crashes, integrating **official standard Sentry SDKs** in your critical services equips the coding agent with the **ultimate reproduction context**:
* **HTTP Request Body:** The exact JSON/Form POST payload sent by the user when the error occurred (sensitive passwords & bearer tokens automatically scrubbed).
* **Breadcrumbs:** Chronological trail of the last 5–10 database SQL queries and outbound HTTP calls leading up to the failure.
* **Source Context:** 5 lines of source code before and after the crashing line.
* **Local Variables:** Exact runtime variable values in the crashing stack frame.

### 1-Line Setup in Target Applications
You do **not** need a heavy self-hosted Sentry server (which requires 16+ GB RAM). The Rust Shipper includes a high-performance, native Sentry Ingest Gateway.

Point the standard Sentry SDK in your application to the Shipper instance:

```env
# In your Laravel, Express, FastAPI, or Go .env file:
SENTRY_DSN=http://public@sre.yourdomain.com:8089/toko-online-api
```

#### Multi-Language Integration Examples:
* **Laravel (PHP):**
  ```bash
  composer require sentry/sentry-laravel
  # Set SENTRY_LARAVEL_DSN=http://public@sre.yourdomain.com:8089/toko-online-api in .env
  ```
* **Express / NestJS (Node.js):**
  ```typescript
  import * as Sentry from "@sentry/node";
  Sentry.init({ dsn: "http://public@sre.yourdomain.com:8089/pos-kasir" });
  ```
* **FastAPI / Django (Python):**
  ```python
  import sentry_sdk
  sentry_sdk.init(dsn="http://public@sre.yourdomain.com:8089/billing-service")
  ```

#### How It Works Under the Hood:
1. When an exception occurs, the Sentry SDK transmits the envelope to `POST /api/:project_id/envelope`.
2. The Rust Shipper extracts the request body, SQL breadcrumbs, stack frames, and source lines.
3. The Shipper forwards the event to OpenObserve (`/api/default/sentry_events/_json`) for long-term storage and dashboard analytics.
4. The Shipper instantly constructs the rich problem payload and dispatches it to your Autonomous Coding Agent in < 10ms!

### 🔑 Multi-Account GitHub Support: Self-Describing Telemetry (SSOT)

If your 20+ applications belong to **different GitHub accounts or client organizations**, you never have to worry about the agent guessing the wrong repository. Applications declare their own repository as the **Single Source of Truth (SSOT)**:

* **Via Sentry SDK (Application Level):**
  Add the `repo_url` tag in your application's `.env`:
  ```env
  SENTRY_TAGS_REPO_URL=https://github.com/client-a/toko-online-api
  SENTRY_TAGS_BRANCH=main
  SENTRY_TAGS_VERIFICATION_COMMAND=php artisan test
  ```
* **Via Coolify / Docker (Container Level):**
  Coolify automatically attaches the `coolify.git.repository` label to your containers. Vector extracts this into OpenObserve, and the Shipper uses it automatically. You can also specify an explicit label or env variable:
  ```yaml
  labels:
    - "sre.repo_url=https://github.com/partner-agency/pos-kasir"
    - "sre.branch=release/v2"
  ```

When the error payload reaches the Shipper, it uses the self-describing `repo_url` directly, completely bypassing any default assumptions.

---

## ⚙️ Application Registry & Convention over Configuration

The SRE Hub adopts a **Hierarchical Resolution** philosophy:
1. **Self-Describing Tag (Highest Priority - SSOT):** Reads `repo_url` directly from the Sentry tag or Docker container label.
2. **Modular File Registry (`apps.d/*.yaml`):** Overrides for non-standard repos or monorepos.
3. **Convention Fallback:** Defaults to `https://github.com/<DEFAULT_GITHUB_ORG>/<app_name>` with test command inferred from the stack trace:
   - PHP Fatal Errors / Exceptions → `php` (`php artisan test`)
   - Python Exceptions → `pytest`
   - Go Runtime Panics → `go test ./...`
   - Node / TypeScript Exceptions → `npm test`

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
