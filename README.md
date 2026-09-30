# 🔭 OpenObserve SRE Hub

[![Rust](https://img.shields.io/badge/Language-Rust_1.98+-orange.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![OpenObserve](https://img.shields.io/badge/Engine-OpenObserve-blue.svg)](https://openobserve.ai/)
[![Coolify Ready](https://img.shields.io/badge/Deploy-Coolify_Auto--Deploy-purple.svg)](#-deployment-di-coolify-via-github-app-auto-deploy)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Centralized Autonomous SRE Telemetry Gateway & Incident Shipper for Multi-App Stacks.**  
> Bertindak sebagai **Sensor & Radar SRE terpusat** yang mengumpulkan log dari 20+ aplikasi di cPanel dan Coolify secara *zero-touch*, mendeteksi insiden produksi secara *real-time* (< 15 detik), mengemas konteks kejadian (*stack trace, breadcrumbs, git commit*), dan menembakkan *standard problem payload* ke **Autonomous Coding Agent** mana pun untuk memicu siklus perbaikan mandiri (*auto-healing loop*).

---

## 📑 Daftar Isi
- [Kenapa Proyek Ini Ada? (The Problem)](#-kenapa-proyek-ini-ada-the-problem)
- [Arsitektur Sistem (Event-Driven Flow)](#-arsitektur-sistem-event-driven-flow)
- [Fitur Utama](#-fitur-utama)
- [🛡️ Mitigasi Ledakan Baris Kode: Modular `apps.d/`](#️-mitigasi-ledakan-baris-kode-modular-appsd)
- [📡 Spesifikasi Webhook Kontrak Coding Agent](#-spesifikasi-webhook-kontrak-coding-agent)
- [🤖 Cara Menyiapkan Environment Coding Agent](#-cara-menyiapkan-environment-coding-agent)
- [🚀 Deployment di Coolify via GitHub App Auto-Deploy](#-deployment-di-coolify-via-github-app-auto-deploy)
- [⚡ Quickstart Lokal (5 Menit)](#-quickstart-lokal-5-menit)
- [📦 Menghubungkan Log 20+ Aplikasi (Zero-Touch)](#-menghubungkan-log-20-aplikasi-zero-touch)
- [🧪 Testing & Verifikasi](#-testing--verifikasi)
- [📚 Dokumentasi Lanjutan](#-dokumentasi-lanjutan)

---

## 🎯 Kenapa Proyek Ini Ada? (The Problem)

Sebagai pengelola atau *one-person company* yang memegang 20+ aplikasi di *production*:
* **Reaktif & Mengganggu Fokus:** Pengguna sering kali menemukan bug lebih dulu dan langsung komplain di chat/WhatsApp sebelum developer menyadarinya.
* **Kelelahan Konteks (Context Fatigue):** Menelusuri log di puluhan server/container berbeda menghabiskan waktu berjam-jam saat terjadi insiden.
* **Biaya Observabilitas Mahal:** Menjalankan Datadog atau NewRelic untuk puluhan aplikasi membakar anggaran; menjalankan ELK stack lokal memakan RAM puluhan GB.

### 💡 Solusi: OpenObserve + Rust Shipper + Coding Agent
1. **OpenObserve:** Engine observabilitas berbasis Rust yang sangat hemat memori (< 200 MB RAM) dan 140x lebih murah daripada Elasticsearch.
2. **Rust Context Shipper (`shipper/`):** Microservice perantara berlatensi rendah (< 10ms) yang menduplikasi error, mengekstrak baris kode yang rusak, dan merakit paket masalah terstandar.
3. **Autonomous Coding Agent (Agent-Agnostic):** Agen AI apa pun (Claude Code, OpenHands, runner lokal, atau custom script) menerima webhook terstandar, memperbaiki bug di branch terisolasi, menjalankan tes, dan membuka Pull Request.

---

## 📐 Arsitektur Sistem (Event-Driven Flow)

Sistem ini memisahkan secara tegas antara **Sensor** (repositori ini) dan **Aktor** (Coding Agent):

```mermaid
flowchart TD
    subgraph Apps ["20+ Production Apps (Zero-Touch Ingestion)"]
        CP[cPanel Shared/VPS<br/>PHP / Node / Python / error_log]
        CL[Coolify Docker Containers<br/>stdout / stderr]
    end

    subgraph Hub ["openobserve-sre (The Sensor)"]
        V[Vector Log Collectors] --> OO[(OpenObserve Engine)]
        OO -->|SQL Stream Alert < 15s| RS[Rust Context Shipper<br/>(Axum + Tokio)]
        RS -->|apps.d/ Scan| REG[Modular App Registry]
        RS -->|Deduplication Cache 30m| DEDUP{Duplikat?}
        DEDUP -->|Ya| DROP[Drop / Throttle]
        DEDUP -->|Tidak| PACK[Pack Standard Context JSON]
    end

    subgraph Actor ["Agent-Agnostic Consumers (The Mechanic)"]
        PACK -->|POST Standard JSON| AGENT[Autonomous Coding Agent<br/>(Webhook Receiver / Sandbox)]
    end

    subgraph Resolution ["Resolution & Deployment"]
        AGENT -->|Create Hotfix Branch & PR| GH[GitHub / GitLab]
        AGENT -->|Interactive Message| WA[Solo Founder (WhatsApp / Telegram)]
        WA -->|Tap 'Approve'| DEP[Coolify Webhook / cPanel Git Pull]
    end

    Apps --> V
```

---

## ✨ Fitur Utama

* **⚡ Deteksi Real-Time (< 15 Detik):** Memanfaatkan SQL Stream Alerting OpenObserve untuk menangkap fatal error dan exception seketika.
* **🛡️ Circuit Breaker & Deduplikasi Otomatis:** Mencegah lonjakan pemanggilan agen dan pemborosan kuota token LLM saat puluhan user mendapati error yang sama dalam kurun waktu 30 menit.
* **🔌 100% Agent-Agnostic:** Mengirimkan kontrak JSON terstandar via webhook HTTP POST. Bebas dihubungkan ke coding agent mana pun tanpa terikat vendor.
* **📦 Modular Registry (`apps.d/`):** Menambah aplikasi ke-21 hingga ke-100 cukup dengan membuat 1 file baru tanpa menyentuh file konfigurasi utama.
* **🚀 Zero-Touch Deployment:** Mengalirkan log langsung dari Docker socket Coolify dan file `error_log` cPanel tanpa mengubah satu baris pun kode aplikasi.

---

## 🛡️ Mitigasi Ledakan Baris Kode: Modular `apps.d/`

Jika 50–100 aplikasi didaftarkan dalam 1 file tunggal, file konfigurasi akan membengkak menjadi ribuan baris dan rawan *git merge conflict*. 

Repositori ini menerapkan arsitektur **"1 App = 1 File Mandiri"** di direktori [`apps.d/`](apps.d/):

```text
apps.d/
├── toko-online-api.yaml      # Konfigurasi App 1
├── pos-kasir.yaml            # Konfigurasi App 2
├── billing-service.yaml      # Konfigurasi App 3
└── ... (dapat menampung ratusan aplikasi)
```

Contoh konfigurasi [`apps.d/toko-online-api.yaml`](apps.d/toko-online-api.yaml):
```yaml
app_name: toko-online-api
environment: production
platform: cpanel
language: php
framework: laravel
repo_url: https://github.com/myorg/toko-online-api
default_branch: main
verification_command: php artisan test
```

Engine Rust Shipper secara otomatis memindai (*glob scan*) seluruh file `.yaml` di `apps.d/` saat *startup*.

---

## 📡 Spesifikasi Webhook Kontrak Coding Agent

Rust Shipper mengirimkan HTTP `POST` ke `AGENT_TARGET_URL` dengan payload JSON yang kaya konteks:

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
      "url": "https://github.com/myorg/toko-online-api",
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

## 🤖 Cara Menyiapkan Environment Coding Agent

Panduan teknis mendalam mengenai arsitektur sandbox, antrean tugas (*job queue*), keamanan eksekusi tes, dan integrasi WhatsApp/Telegram approval telah didokumentasikan secara terpisah di:

👉 **[docs/AGENT_ENVIRONMENT_SETUP.md](docs/AGENT_ENVIRONMENT_SETUP.md)**

---

## 🚀 Deployment di Coolify via GitHub App Auto-Deploy

Repositori ini telah dikonfigurasi agar **100% siap di-deploy di Coolify** dengan siklus *auto-deploy* saat Anda melakukan push ke branch `main`.

### Langkah Setup di Coolify:
1. **Push Repositori ini ke GitHub Anda:**
   ```bash
   cd /home/server/projects/openobserve-sre
   git push origin main
   ```
2. **Buat Resource di Coolify:**
   * Buka dashboard Coolify → **+ New Resource** → **Docker Compose Application**.
   * Pilih **GitHub App** → pilih repositori `ihkaru/openobserve-sre` dan branch `main`.
3. **Environment Variables:**
   * Salin variabel dari [`.env.example`](.env.example) ke tab **Environment Variables** di Coolify:
     - `ZO_ROOT_USER_EMAIL`: Email admin OpenObserve Anda.
     - `ZO_ROOT_USER_PASSWORD`: Password admin OpenObserve Anda.
     - `AGENT_TARGET_URL`: Endpoint Webhook Coding Agent Anda.
     - `AGENT_AUTH_TOKEN`: Token rahasia jika diperlukan.
4. **Deploy:**
   * Klik tombol **Deploy** di Coolify.
   * Coolify akan otomatis menjalankan OpenObserve (volume persisten `openobserve_data`) dan mengompilasi image Rust Shipper secara multi-stage (~15MB).
5. **Auto-Deploy Aktif:**
   * Setiap kali Anda menambah file baru di `apps.d/` dan melakukan `git push`, Coolify otomatis me-redeploy stack secara *seamless*!

---

## ⚡ Quickstart Lokal (5 Menit)

Jika ingin menjalankan pengujian di mesin lokal:

```bash
cd /home/server/projects/openobserve-sre

# 1. Jalankan OpenObserve dan Shipper bersamaan
docker compose up -d --build

# 2. Verifikasi status kedua service
curl http://localhost:8089/healthz
# Output: {"active_cached_incidents":0,"runtime":"rust","service":"openobserve-sre-shipper","status":"ok"}

# 3. Uji Simulasi Insiden (Smoke Test)
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
*Respons:* `{"app_name":"toko-online-api","event_id":"evt_...","status":"dispatched"}`.

---

## 📦 Menghubungkan Log 20+ Aplikasi (Zero-Touch)

Anda **TIDAK PERLU mengubah kode aplikasi** apa pun (*Zero-Touch*).

### A. Aplikasi di Coolify (Docker)
Jalankan Vector container di host Coolify menggunakan konfigurasi [`collectors/vector-coolify.yaml`](collectors/vector-coolify.yaml):
```bash
docker run -d \
  --name vector-coolify \
  --restart unless-stopped \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  -v $(pwd)/collectors/vector-coolify.yaml:/etc/vector/vector.yaml:ro \
  timberio/vector:latest-alpine --config /etc/vector/vector.yaml
```

### B. Aplikasi di cPanel (PHP / Shared)
Jalankan binary Vector standalone di akun cPanel menggunakan konfigurasi [`collectors/vector-cpanel.yaml`](collectors/vector-cpanel.yaml):
```bash
vector --config collectors/vector-cpanel.yaml &
```

---

## 🧪 Testing & Verifikasi

Seluruh logika parsing, caching, dan pipeline telah dilengkapi unit test bawaan di Rust:

```bash
cd /home/server/projects/openobserve-sre/shipper
cargo test
```

Hasil pengujian memverifikasi:
* ✅ PHP Fatal Error extractor (`file_path`, `line_number`).
* ✅ Node.js/TypeScript stacktrace extractor.
* ✅ Python traceback extractor.
* ✅ Go runtime panic extractor (`main.go:line`).
* ✅ Composite parser fallback.
* ✅ In-memory circuit breaker deduplication & cooldown TTL.
* ✅ End-to-end Orchestrator pipeline.

---

## 📚 Dokumentasi Lanjutan
* **[docs/AGENT_ENVIRONMENT_SETUP.md](docs/AGENT_ENVIRONMENT_SETUP.md):** Panduan setup *runtime sandbox*, antrean, dan alur eksekusi coding agent.
* **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md):** Analisis arsitektur sistem, *2-Tier Observability*, dan mitigasi *The Dead App Paradox*.
* **[docs/CONTEXT_SPEC.md](docs/CONTEXT_SPEC.md):** Spesifikasi lengkap format JSON payload insiden.
* **[docs/ROADMAP.md](docs/ROADMAP.md):** Checklist tahapan implementasi dari Day 1 s/d Auto-Deploy.
* **[docs/sample-payload.json](docs/sample-payload.json):** Contoh konkret JSON konteks insiden siap pakai.
