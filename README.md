# 🔭 OpenObserve SRE Hub (Rust Edition)

[![Rust](https://img.shields.io/badge/Language-Rust_1.98+-orange.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![OpenObserve](https://img.shields.io/badge/Engine-OpenObserve-blue.svg)](https://openobserve.ai/)
[![Architecture](https://img.shields.io/badge/Design-SOLID_Principles-success.svg)](#-arsitektur-dan-solid-principles)
[![Coolify Ready](https://img.shields.io/badge/Deploy-Coolify_Auto--Deploy-purple.svg)](#-deployment-di-coolify-via-github-app-auto-deploy)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Central Observability & Telemetry Gateway untuk One-Person Company (OPC).**  
> Bertindak sebagai **Sensor & Radar SRE terpusat** yang mengumpulkan log dari 20+ aplikasi di cPanel dan Coolify secara *zero-touch*, mendeteksi insiden produksi secara *real-time* (< 15 detik), mengemas konteks kejadian (*stack trace, breadcrumbs, git commit*), dan menembakkan *standard problem payload* ke **Aina** (atau Coding Agent mana pun) untuk memicu siklus perbaikan mandiri (*autonomous remediation*).

---

## 📑 Daftar Isi
- [Kenapa Proyek Ini Ada? (The Problem)](#-kenapa-proyek-ini-ada-the-problem)
- [Arsitektur Sistem (Separation of Concerns)](#-arsitektur-sistem-separation-of-concerns)
- [Arsitektur Kode & SOLID Principles (Rust)](#-arsitektur-dan-solid-principles)
- [🛡️ Mitigasi Ledakan Baris Kode: Modular `apps.d/`](#️-mitigasi-ledakan-baris-kode-modular-appsd)
- [🚀 Deployment di Coolify via GitHub App Auto-Deploy](#-deployment-di-coolify-via-github-app-auto-deploy)
- [⚡ Quickstart Lokal (5 Menit)](#-quickstart-lokal-5-menit)
- [📦 Cara Menghubungkan Log 20+ Aplikasi (Zero-Touch)](#-cara-menghubungkan-log-20-aplikasi-zero-touch)
- [🧪 Testing & Verifikasi](#-testing--verifikasi)
- [📚 Dokumentasi Lanjutan](#-dokumentasi-lanjutan)

---

## 🎯 Kenapa Proyek Ini Ada? (The Problem)

Sebagai *solo developer / one-person company* yang memegang 20+ aplikasi di *production*:
* **Reaktif & Memalukan:** Pengguna sering kali menemukan bug lebih dulu dan langsung komplain di WhatsApp.
* **Kelelahan Konteks (Fatigue):** Menelusuri log di 20 server berbeda menghabiskan waktu berjam-jam.
* **Biaya Observabilitas Mahal:** Menjalankan Datadog atau NewRelic untuk puluhan aplikasi membakar anggaran; menjalankan ELK stack lokal memakan RAM puluhan GB.

### 💡 Solusi: OpenObserve + Rust Shipper + Aina
1. **OpenObserve:** Engine observabilitas berbasis Rust yang sangat hemat memori (< 200 MB RAM) dan 140x lebih murah daripada Elasticsearch.
2. **Rust Context Shipper (`shipper/`):** Service perantara ultra-cepat (< 10ms latensi) yang menduplikasi error dan merakit potongan kode dari GitHub.
3. **Aina (Coding Agent):** Bertindak sebagai "mekanik" independen yang menerima notifikasi, memperbaiki bug, dan membuka Pull Request otomatis.

---

## 📐 Arsitektur Sistem (Separation of Concerns)

Proyek ini memisahkan secara tegas antara **Sensor** (repositori ini) dan **Aktor** (Aina / Coding Agent):

```mermaid
flowchart TD
    subgraph Apps ["20+ Production Apps (Zero-Touch Ingestion)"]
        CP[cPanel Shared/VPS<br/>PHP / Node / error_log]
        CL[Coolify Docker Containers<br/>stdout / stderr]
    end

    subgraph Hub ["openobserve-sre (Repositori Ini - The Sensor)"]
        V[Vector Log Collectors] --> OO[(OpenObserve Engine)]
        OO -->|SQL Stream Alert < 15s| RS[Rust Context Shipper<br/>(Axum + Tokio)]
        RS -->|apps.d/ Scan| REG[Modular App Registry]
        RS -->|Deduplication Cache 30m| DEDUP{Duplikat?}
        DEDUP -->|Ya| DROP[Drop / Throttle]
        DEDUP -->|Tidak| PACK[Pack Standard Context JSON]
    end

    subgraph Actor ["Agent-Agnostic Consumers (The Mechanic)"]
        PACK -->|Standard Webhook Payload| AINA[Aina via AGY Mesh<br/>(Primary Agent)]
        PACK -.->|Plug & Play| OTHERS[Claude Code / OpenHands / Cursor]
    end

    subgraph Resolution ["Resolution & Deployment"]
        AINA -->|Create Hotfix Branch & PR| GH[GitHub / GitLab]
        AINA -->|1-Tap Approval Message| WA[Solo Founder (WhatsApp / Telegram)]
        WA -->|Tap 'Approve'| DEP[Coolify Webhook / cPanel Git Pull]
    end

    Apps --> V
```

---

## 🧱 Arsitektur dan SOLID Principles

Kode di dalam folder [`shipper/`](shipper/) dibangun dengan **Rust** menerapkan kelima pilar **SOLID**:

| Prinsip SOLID | Implementasi di `shipper/` | File Sumber |
| :--- | :--- | :--- |
| **S - Single Responsibility** | Masing-masing modul hanya memiliki satu alasan untuk berubah: Parsing log terpisah dari Caching, Caching terpisah dari Dispatching. | `parsers/`, `cache/`, `dispatchers/` |
| **O - Open/Closed** | Mendukung bahasa/framework baru tanpa mengubah parser yang sudah ada via `CompositeLogParser` dan trait `LogParser` (PHP, Node, Python, Go). | [`src/parsers/composite.rs`](shipper/src/parsers/composite.rs) |
| **L - Liskov Substitution** | Seluruh implementasi cache (`InMemoryDeduplicator`) mengimplementasikan trait `Deduplicator` dan dapat digantikan dengan Redis/SQLite tanpa mengubah logic sistem. | [`src/cache/memory.rs`](shipper/src/cache/memory.rs) |
| **I - Interface Segregation** | Trait dibuat kecil dan spesifik (`LogParser`, `Deduplicator`, `AppRegistry`, `Dispatcher`), bukan satu interface raksasa (*god interface*). | [`src/traits.rs`](shipper/src/traits.rs) |
| **D - Dependency Inversion** | High-level `IncidentOrchestrator` tidak bergantung pada struct konkret, melainkan bergantung pada abstraksi trait (`Arc<dyn LogParser>`, `Arc<dyn Deduplicator>`, dll). | [`src/orchestrator.rs`](shipper/src/orchestrator.rs) |

---

## 🛡️ Mitigasi Ledakan Baris Kode: Modular `apps.d/`

### Masalah pada Sistem Konvensional:
Jika 50–100 aplikasi didaftarkan dalam 1 file `config.yaml` tunggal:
* File membengkak menjadi ribuan baris.
* 1 kesalahan spasi/indentasi YAML akan melumpuhkan registry **seluruh 100 aplikasi**.
* Rawan terjadi *git merge conflict* saat onboarding aplikasi baru.

### Solusi Desain: "1 App = 1 File" di [`apps.d/`](apps.d/)
Setiap aplikasi memiliki file deklarasi mandiri berukuran ~10 baris di direktori `apps.d/`:

```text
apps.d/
├── toko-online-api.yaml      # Konfigurasi App 1
├── pos-kasir.yaml            # Konfigurasi App 2
├── billing-service.yaml      # Konfigurasi App 3
└── ... (bisa ratusan file tanpa pernah terjadi konflik)
```

Contoh isi [`apps.d/toko-online-api.yaml`](apps.d/toko-online-api.yaml):
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

Engine Rust Shipper secara otomatis memindai (*directory glob scan*) seluruh file `.yaml` di `apps.d/` saat *startup*. **Menambah aplikasi baru = cukup buat 1 file baru di `apps.d/`!**

---

## 🚀 Deployment di Coolify via GitHub App Auto-Deploy

Repositori ini telah dikonfigurasi agar **100% siap di-deploy di Coolify** menggunakan GitHub App dengan siklus *auto-deploy* saat Anda melakukan push ke branch `main`.

### Langkah Setup di Coolify:
1. **Push Repositori ini ke GitHub Anda:**
   ```bash
   cd /home/server/projects/openobserve-sre
   git init -b main
   git add .
   git commit -m "feat: initial openobserve-sre stack"
   git remote add origin git@github.com:USERNAME/openobserve-sre.git
   git push -u origin main
   ```
2. **Buat Resource di Coolify:**
   * Masuk ke dashboard Coolify Anda.
   * Pilih Project / Environment Anda → Klik **+ New Resource**.
   * Pilih **Docker Compose Application**.
   * Pilih integrasi **GitHub App**, lalu pilih repositori `openobserve-sre` dan branch `main`.
3. **Environment Variables:**
   * Salin variabel dari [`.env.example`](.env.example) ke tab **Environment Variables** di Coolify:
     - `ZO_ROOT_USER_EMAIL`: Email admin OpenObserve Anda.
     - `ZO_ROOT_USER_PASSWORD`: Password admin OpenObserve Anda.
     - `AGENT_TARGET_URL`: Webhook URL Aina (misal runner webhook AGY Mesh).
     - `AGENT_AUTH_TOKEN`: Token rahasia jika diperlukan.
4. **Deploy:**
   * Klik tombol **Deploy** di Coolify.
   * Coolify akan otomatis:
     - Menjalankan image OpenObserve dengan volume persisten `openobserve_data`.
     - Mengompilasi image Rust Shipper secara multi-stage (hanya ~15MB!).
     - Mengaktifkan healthcheck di kedua service.
5. **Auto-Deploy Aktif!**
   * Setiap kali Anda menambah file baru di `apps.d/` dan melakukan `git push`, Coolify akan otomatis me-redeploy stack secara *seamless* tanpa downtime!

---

## ⚡ Quickstart Lokal (5 Menit)

Jika ingin mencoba di mesin lokal sebelum ke Coolify:

```bash
cd /home/server/projects/openobserve-sre

# 1. Jalankan OpenObserve dan Shipper bersamaan
docker compose up -d --build

# 2. Periksa status kedua container
docker compose ps

# 3. Verifikasi Shipper sehat
curl http://localhost:8089/healthz
# Output: {"active_cached_incidents":0,"runtime":"rust","service":"openobserve-sre-shipper","status":"ok"}

# 4. Smoke Test Simulasi Error
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

## 📦 Cara Menghubungkan Log 20+ Aplikasi (Zero-Touch)

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

Hasil test:
* ✅ PHP Fatal Error extractor (`file_path`, `line_number`).
* ✅ Node.js/TypeScript stacktrace extractor.
* ✅ Python traceback extractor.
* ✅ Go runtime panic extractor (`main.go:line`).
* ✅ Composite parser fallback.
* ✅ In-memory circuit breaker deduplication & cooldown TTL.
* ✅ End-to-end Orchestrator pipeline.

---

## 📚 Dokumentasi Lanjutan
* **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md):** Analisis *2-Tier Observability* dan alasan menghindari endpoint diagnostik internal.
* **[docs/CONTEXT_SPEC.md](docs/CONTEXT_SPEC.md):** Format JSON standar yang dikirimkan ke Aina.
* **[docs/ROADMAP.md](docs/ROADMAP.md):** Checklist tahapan implementasi dari Day 1 s/d Auto-Deploy.
* **[docs/sample-payload.json](docs/sample-payload.json):** Contoh konkret JSON konteks insiden.
