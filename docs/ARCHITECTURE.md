# 📐 Arsitektur Sistem: OpenObserve SRE Hub (Agent-Agnostic)

Dokumen ini menjelaskan arsitektur teknis dari repositori **`openobserve-sre`** sebagai *Observability Hub & Telemetry Gateway*. Dokumen ini juga mengulas strategi pengumpulan log dari 20+ aplikasi di cPanel dan Coolify, prinsip *Agent-Agnostic*, serta integrasi dengan **Aina** (melalui ekosistem AGY Mesh).

---

## 1. Filosofi: Pemisahan Sensor (SRE Hub) dan Aktor (Aina / Coding Agent)

Salah satu kelemahan sistem self-healing konvensional adalah *tight coupling* antara sistem pemantau (*monitoring*) dengan bot perbaikan kode. Jika model atau agent diganti, seluruh sistem observabilitas harus dirombak.

Di repositori ini:
* **`openobserve-sre` bertindak sebagai RADAR / SENSOR:**
  * Mengumpulkan telemetry (log, metrik, traces) dari 20+ aplikasi heterogen secara *zero-touch*.
  * Mengidentifikasi anomali dan *fatal errors* menggunakan SQL real-time stream.
  * Menjalankan **Deduplication & Circuit Breaking** (agar tidak membakar token jika terjadi lonjakan error).
  * Mengemas konteks insiden (*breadcrumbs log*, baris file kode, target repositori) menjadi **Standard Problem Payload**.
  * Menyediakan **OpenObserve MCP (Model Context Protocol)** agar agent dapat menanyakan log runtime secara interaktif.
* **Aina (atau Agent Apa Pun) bertindak sebagai MEKANIK:**
  * Menerima payload standar via webhook.
  * Menjalankan penalaran (*root cause analysis*), membuat branch hotfix, menulis unit test reproduksi, memperbaiki kode, dan membuka PR di GitHub.
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
        SQL --> Shipper[Context Shipper & Deduplicator<br/>(Python/FastAPI Service)]
        GH_API[(GitHub API)] -->|Source Snippet Context| Shipper
    end

    subgraph Consumers ["Agent-Agnostic Consumers"]
        direction TB
        Shipper -->|Standard JSON Payload| AINA[Aina via AGY Mesh<br/>(Primary Coding Agent)]
        Shipper -.->|Alternatif: REST Webhook| EXT[Claude Code / OpenHands / GitHub Actions]
    end

    subgraph Remediation ["Resolution & Deployment"]
        AINA -->|Open Pull Request| REPO[GitHub / GitLab 20+ Repos]
        AINA -->|Interactive Alert & 1-Tap Button| WA[Solo Founder (WhatsApp / Telegram)]
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
   * *Hasil:* **85–90% bug kode (null pointer, syntax error, undefined array key, query exception) dapat diperbaiki tuntas oleh Aina hanya bermodalkan Tier 1 ini tanpa perlu menyentuh kode aplikasi!**

3. **Tier 2 (Out-of-Band Introspection - Hanya Jika Sangat Dibutuhkan):**
   * Jika Aina butuh memeriksa kondisi database (misal: status migrasi tabel), gunakan eksekusi CLI aman via container/SSH (contoh: `docker exec <container> php artisan migrate:status`), **bukan melalui HTTP request publik**.

---

## 3. Komponen Utama `openobserve-sre`

### A. OpenObserve Engine (`docker-compose.yml`)
* Single binary berbasis Rust, sangat efisien (RAM < 200 MB untuk beban puluhan aplikasi).
* Menyimpan log dalam format columnar Parquet (bisa di-mount ke disk lokal atau S3).
* Menyediakan UI web terpusat untuk mencari log di seluruh 20 aplikasi.

### B. Vector Collectors (`collectors/`)
* **Coolify Collector (`vector-coolify.yaml`):** Membaca `/var/run/docker.sock`, otomatis melabeli nama container sebagai `app_name`, dan mengalirkan log ke OpenObserve via HTTP ingestion.
* **cPanel Collector (`vector-cpanel.yaml`):** Memantau `~/public_html/error_log` dan Apache vhost log secara non-intrusif.

### C. Context Shipper & Deduplicator (`shipper/`)
* Menerima alert dari OpenObserve.
* Menghitung hash unik: `hash(app_name + error_file + error_line + error_type)`.
* Menerapkan **cooldown 30 menit** (mencegah agent memproses insiden berulang saat puluhan user mendapati error yang sama).
* Memperkaya payload dengan cuplikan kode dari GitHub API (20 baris sebelum dan sesudah baris error).
* Mengirimkan payload JSON terstandar ke Aina / Webhook Agent.

---

## 4. Keuntungan Pendekatan Agent-Agnostic bagi Ekosistem Aina

* **Aina Tetap Independen:** Aina tetap berada di ekosistem AGY mesh-nya, menjalankan perannya sebagai *senior coding companion*.
* **Standarisasi Kontrak Data:** Aina hanya perlu memahami satu spesifikasi payload JSON (dijelaskan di [docs/CONTEXT_SPEC.md](CONTEXT_SPEC.md)).
* **Plug & Play:** Jika Anda ingin menghubungkan notifikasi ini ke n8n, Slack, Discord, atau agent lain secara paralel, cukup tambahkan URL tujuan di konfigurasi `shipper/config.yaml`.
