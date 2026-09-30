# 📐 System Architecture: OpenObserve SRE Hub (Agent-Agnostic)

Dokumen ini menjelaskan arsitektur teknis dari repositori **`openobserve-sre`** sebagai *Observability Hub & Telemetry Gateway*. Dokumen ini mengulas strategi pengumpulan log dari 20+ aplikasi di cPanel dan Coolify, prinsip *Agent-Agnostic*, serta kontrak data dengan **Coding Agent API**.

---

## 1. Filosofi: Pemisahan Sensor (SRE Hub) dan Aktor (Coding Agent)

Salah satu kelemahan sistem self-healing konvensional adalah *tight coupling* antara sistem pemantau (*monitoring*) dengan agen perbaikan kode. Jika model atau agent diganti, seluruh sistem observabilitas harus dirombak.

Di repositori ini:
* **`openobserve-sre` bertindak sebagai RADAR / SENSOR:**
  * Mengumpulkan telemetry (log, metrik, traces) dari 20+ aplikasi heterogen secara *zero-touch*.
  * Mengidentifikasi anomali dan *fatal errors* menggunakan SQL real-time stream.
  * Menjalankan **Deduplication & Circuit Breaking** (agar tidak membakar token jika terjadi lonjakan error berulang).
  * Mengemas konteks insiden (*breadcrumbs log*, baris file kode, target repositori, perintah tes) menjadi **Standard Problem Payload**.
  * Menyediakan **OpenObserve MCP (Model Context Protocol)** agar agent dapat menanyakan log runtime secara interaktif jika diperlukan.
* **Autonomous Coding Agent bertindak sebagai MEKANIK (Aktor Independen):**
  * Menerima payload standar via webhook HTTP POST.
  * Menjalankan penalaran (*root cause analysis*), membuat branch hotfix, mereproduksi masalah, memperbaiki kode, dan membuka PR di GitHub.
  * Meminta persetujuan rilis ke solo founder via WhatsApp/Telegram sebelum trigger deploy.

```mermaid
flowchart TD
    subgraph TargetApps ["20+ Production Applications (Zero-Touch)"]
        CP[cPanel Shared/VPS<br/>PHP / Node.js / Python / error_log]
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
        Shipper -->|Standard JSON Payload| AGENT[Autonomous Coding Agent<br/>(Webhook Receiver / Runner)]
        Shipper -.->|Plug & Play| OTHERS[Claude Code / OpenHands / Cursor / CI]
    end

    subgraph Remediation ["Resolution & Deployment"]
        AGENT -->|Open Pull Request| REPO[GitHub / GitLab 20+ Repos]
        AGENT -->|Interactive Alert & 1-Tap Button| WA[Solo Founder (WhatsApp / Telegram)]
        WA -->|Tap 'Approve'| DEP[Coolify API Webhook / cPanel Git Pull]
    end

    TargetApps --> V
```

---

## 2. Apakah Aplikasi Membutuhkan Endpoint Observabilitas Khusus?

Pertanyaan arsitektur: **"Apakah observabilitas bawaan aplikasi yang dimaintain harus bagus agar coding agent bisa memanggil endpoint aplikasi untuk meminta konteks tambahan?"**

### Rekomendasi: Gunakan "2-Tier Observability" (Hindari The Dead App Paradox)

1. **JANGAN mengandalkan HTTP Endpoint Aplikasi untuk Diagnostik Utama:**
   * **The Dead App Paradox:** Saat aplikasi mengalami *Fatal Error*, *PHP OOM*, atau *Crash Loop*, HTTP web server aplikasi tersebut **sedang mati**. Jika agent memanggil `GET https://app.com/debug-context`, pemanggilan itu akan gagal (*502 Bad Gateway* atau *Timeout*).
   * **Security Surface:** Menaruh endpoint diagnostik di 20 aplikasi membuka risiko kebocoran data sensitif (*runtime environment*, konfigurasi database) jika pengamanan token bocor.
   * **Friction 20 Aplikasi:** Mengubah kode pada 20 aplikasi warisan memakan waktu terlalu lama.

2. **Tier 1 (Non-Invasive Outer Context - 100% Wajib & Zero Code Change):**
   * Semua konteks diambil dari luar aplikasi:
     - **Stack Trace & Breadcrumbs:** Diambil dari OpenObserve (diteruskan dari log container Docker di Coolify dan `error_log` di cPanel).
     - **Source Code State:** Diambil langsung dari GitHub API berdasarkan branch produksi / commit hash.
     - **Container State:** Diambil dari status Coolify/Docker daemon (apakah OOMKilled, exit code, restart count).
   * *Hasil:* **85–90% bug kode (null pointer, syntax error, undefined array key, query exception) dapat diperbaiki tuntas oleh coding agent hanya bermodalkan Tier 1 ini tanpa perlu menyentuh kode aplikasi!**

3. **Tier 2 (Out-of-Band Introspection - Hanya Jika Sangat Dibutuhkan):**
   * Jika agent butuh memeriksa kondisi database (misal: status migrasi tabel), gunakan eksekusi CLI aman via container/SSH (contoh: `docker exec <container> php artisan migrate:status`), **bukan melalui HTTP request publik**.

---

## 3. Komponen Utama `openobserve-sre`

### A. OpenObserve Engine (`docker-compose.yml`)
* Single binary berbasis Rust, sangat efisien (RAM < 200 MB untuk beban puluhan aplikasi).
* Menyimpan log dalam format columnar Parquet dengan volume persisten `openobserve_data`.
* Menyediakan UI web terpusat untuk mencari log di seluruh 20+ aplikasi.

### B. Vector Collectors (`collectors/`)
* **Coolify Collector (`vector-coolify.yaml`):** Membaca `/var/run/docker.sock`, otomatis melabeli nama container sebagai `app_name`, dan mengalirkan log ke OpenObserve via HTTP ingestion.
* **cPanel Collector (`vector-cpanel.yaml`):** Memantau `~/public_html/error_log` dan Apache vhost log secara non-intrusif.

### C. Context Shipper & Deduplicator (`shipper/`)
* Ditulis dalam bahasa **Rust** menggunakan Axum dan Tokio.
* Menerima alert dari OpenObserve.
* Menghitung hash unik: `hash(app_name + error_file + error_line + error_type)`.
* Menerapkan **cooldown 30 menit** (mencegah agent memproses insiden berulang saat puluhan user mendapati error yang sama).
* Membaca registrasi modular dari `apps.d/*.yaml`.
* Mengirimkan payload JSON terstandar ke Coding Agent Webhook endpoint.

---

## 4. Keuntungan Pendekatan Agent-Agnostic

* **Bebas Keterikatan Vendor:** Sistem observabilitas ini tidak terikat pada satu LLM atau agent runtime tertentu.
* **Standarisasi Kontrak Data:** Setiap agent hanya perlu memahami satu spesifikasi payload JSON (dijelaskan di [docs/CONTEXT_SPEC.md](CONTEXT_SPEC.md)).
* **Plug & Play:** Jika Anda ingin mengganti agen perbaikan kode dari runner lokal ke cloud worker (atau GitHub Actions), cukup ubah `AGENT_TARGET_URL` di konfigurasi environment.
