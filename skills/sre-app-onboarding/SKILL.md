---
name: sre-app-onboarding
description: >-
  Standard operating procedure and runbook for instrumenting applications with Sentry SDK
  and connecting them to the OpenObserve SRE Gateway (sre.yourdomain.com). Use whenever
  onboarding a new service, adding error tracking/observability, configuring Sentry DSN,
  setting up self-describing telemetry tags, or verifying incident triage dispatch to Aina.
---

# SRE App Onboarding Runbook (Self-Describing Telemetry)

Panduan operasional resmi untuk mengintegrasikan aplikasi backend/frontend ke **OpenObserve SRE Gateway** (`sre.yourdomain.com`) dan menghubungkannya dengan auto-triage AI Coding Agent (**Aina**).

---

## 🌟 Prinsip Utama (Zero-Config Architecture)

1. **Stateless & Git-Clean**: Gateway SRE bekerja tanpa memerlukan file konfigurasi di server shipper (`apps.d/*.yaml`). Jangan pernah memasukkan file registri aplikasi ke Git.
2. **Self-Describing Telemetry (Tier 1 SSOT)**: Metadata kritis (`app_name`, `repository`, `verification_command`, `branch`) disematkan langsung sebagai tag di SDK Sentry aplikasi.
3. **Convention over Configuration (Tier 2 Fallback)**: Jika tag tidak lengkap, gateway secara otomatis menginferensi nama app dari rute DSN dan perintah verifikasi dari platform bahasa aplikasi.
4. **Instant AI Auto-Triage**: Setiap insiden otomatis dikirimkan ke endpoint AI Agent (`https://agent.yourdomain.com/api/v1/events`) dan dinotifikasi ke WhatsApp pengembang.

---

## 📡 Format DSN Standar

Setiap aplikasi yang ingin diobserve menggunakan format DSN:

```text
https://<public_key>@sre.yourdomain.com/<app_name>
```

- `<public_key>`: Sembarang string alfanumerik (contoh: `public`, `sre`).
- `<app_name>`: Pengenal unik aplikasi dalam huruf kecil dan kebab-case (contoh: `my-app`, `billing-service`, `pos-kasir`).

Contoh di file `.env`:
```env
SENTRY_DSN=https://public@sre.yourdomain.com/my-app
```

---

## 🏷️ Tag & Konteks Telemetri Mandiri (Self-Describing Telemetry)

Sematkan tag dan konteks berikut pada inisialisasi Sentry SDK. Informasi ini menjadi bahan bakar utama bagi **AI Coding Agent (Aina)** untuk menganalisis dan mereproduksi insiden secara presisi tanpa salah diagnosa (*hallucination / false lead*).

### 1. Tag Inti (Core Identity Tags)
| Tag Key | Wajib/Rekomendasi | Deskripsi | Contoh Nilai |
| :--- | :--- | :--- | :--- |
| `app_name` | **Wajib** | Slug unik nama aplikasi | `my-app`, `billing-service` |
| `repository` | **Sangat Dianjurkan** | URL repositori GitHub | `https://github.com/your-org/my-app` |
| `verification_command` | **Sangat Dianjurkan** | Perintah test untuk AI Agent | `php artisan test`, `npm test`, `pytest` |
| `branch` | Opsional | Target branch default | `main` |

### 2. Konteks Deployment (Deployment & Runtime Context)
Membantu AI Agent membedakan apakah error merupakan bug logika murni atau **transient deployment race condition** (misal: migrasi database belum tuntas, OPcache/Octane worker desync, atau Vite chunk hash mismatch sesaat setelah rilis).

| Tag / Field | Level | Deskripsi | Contoh Nilai |
| :--- | :--- | :--- | :--- |
| `release` | **Sangat Dianjurkan** | Git commit SHA atau tag rilis | `git.8925a9e`, `v1.2.0` |
| `environment` | **Sangat Dianjurkan** | Lingkungan eksekusi | `production`, `staging`, `local` |
| `deployment_target` | Rekomendasi | Platform deployment / orchestrator | `coolify`, `docker-compose`, `k8s`, `bare-metal` |
| `runtime` | Rekomendasi | Engine / runtime aplikasi aktif | `frankenphp-octane`, `php-fpm`, `node:20-alpine`, `bun:1.1` |

### 3. Konteks Device & Network (Khusus Frontend / Mobile / PWA)
Mencegah AI Agent terjebak memodifikasi logic bisnis ketika error sebenarnya disebabkan oleh **quirk versi browser/device lama** (butuh polyfill) atau **kehilangan sinyal network**.

| Tag / Field | Level | Deskripsi | Contoh Nilai |
| :--- | :--- | :--- | :--- |
| `device_platform` | **Sangat Dianjurkan** | Platform eksekusi client | `android`, `ios`, `browser`, `electron` |
| `device_model` | Rekomendasi | Model fisik perangkat | `Samsung SM-A055F`, `iPhone 15 Pro`, `Desktop Chrome` |
| `os_version` | Rekomendasi | Versi sistem operasi client | `Android 13`, `iOS 17.4`, `Windows 11` |
| `network_type` | Rekomendasi | Status konektivitas saat error | `wifi`, `cellular-4g`, `offline` |
| `is_native` | Rekomendasi | Apakah berjalan dalam wrapper natif | `true` (Capacitor/Cordova) / `false` (Web) |

---

## 🚀 Prosedur Onboarding per Framework

### 1. PHP / Laravel
Lihat template lengkap: [laravel.php](./examples/laravel.php)

1. Pasang dependensi:
   ```bash
   composer require sentry/sentry-laravel
   ```
2. Buat konfigurasi jika belum ada:
   ```bash
   php artisan sentry:publish
   ```
3. Set `.env`:
   ```env
   SENTRY_LARAVEL_DSN=https://public@sre.yourdomain.com/<app_name>
   SENTRY_TRACES_SAMPLE_RATE=0.2
   ```
4. Tambahkan tag mandiri di `config/sentry.php`:
   ```php
   'tags' => [
       'app_name' => env('APP_NAME_SLUG', 'my-app'),
       'repository' => env('GITHUB_REPOSITORY', 'https://github.com/your-org/my-app'),
       'verification_command' => env('SRE_VERIFY_CMD', 'php artisan test'),
       'branch' => env('GIT_BRANCH', 'main'),
   ],
   ```
5. Uji coba pengiriman event:
   ```bash
   php artisan sentry:test
   ```

### 2. Node.js / Express / TypeScript
Lihat template lengkap: [express.ts](./examples/express.ts)

1. Pasang dependensi:
   ```bash
   npm install @sentry/node
   ```
2. Inisialisasi pada entry point:
   ```typescript
   import * as Sentry from "@sentry/node";

   Sentry.init({
     dsn: process.env.SENTRY_DSN || "https://public@sre.yourdomain.com/my-app",
     initialScope: {
       tags: {
         app_name: "my-app",
         repository: "https://github.com/your-org/my-app",
         verification_command: "npm test",
         branch: "main",
       },
     },
   });
   ```

### 3. Python / FastAPI / Django
Lihat template lengkap: [fastapi.py](./examples/fastapi.py)

1. Pasang dependensi:
   ```bash
   pip install sentry-sdk
   ```
2. Inisialisasi:
   ```python
   import sentry_sdk

   sentry_sdk.init(
       dsn="https://public@sre.yourdomain.com/my-python-app",
       traces_sample_rate=0.2,
   )
   with sentry_sdk.configure_scope() as scope:
       scope.set_tag("app_name", "my-python-app")
       scope.set_tag("repository", "https://github.com/your-org/my-python-app")
       scope.set_tag("verification_command", "pytest")
       scope.set_tag("branch", "main")
   ```

### 4. Golang
Lihat template lengkap: [golang.go](./examples/golang.go)

1. Pasang package:
   ```bash
   go get github.com/getsentry/sentry-go
   ```
2. Inisialisasi dan konfigurasi scope tags:
   ```go
   sentry.Init(sentry.ClientOptions{
       Dsn: "https://public@sre.yourdomain.com/my-go-service",
       Release: "git." + os.Getenv("GIT_COMMIT"),
       Environment: os.Getenv("APP_ENV"),
   })
   sentry.ConfigureScope(func(scope *sentry.Scope) {
       scope.SetTag("app_name", "my-go-service")
       scope.SetTag("repository", "https://github.com/your-org/my-go-service")
       scope.SetTag("verification_command", "go test ./...")
       scope.SetTag("branch", "main")
       scope.SetTag("runtime", "go" + runtime.Version())
   })
   ```

### 5. Frontend SPA & Mobile Client (Vue.js / Capacitor / Offline-First)
Lihat template lengkap: [client-vue.ts](./examples/client-vue.ts)

1. Pasang dependensi:
   ```bash
   npm install @sentry/vue
   # Opsional jika native mobile (Capacitor):
   npm install @capacitor/device @capacitor/network @capacitor/app
   ```
2. Inisialisasi dengan Offline-First IndexedDB Transport & Device Context:
   ```typescript
   import * as Sentry from "@sentry/vue";
   import { makeBrowserOfflineTransport, makeFetchTransport } from "@sentry/browser";

   Sentry.init({
     app,
     dsn: import.meta.env.VITE_SENTRY_DSN || "https://public@sre.yourdomain.com/my-client-app",
     release: import.meta.env.VITE_APP_VERSION || "my-client-app@1.0.0",
     // Menyimpan envelope ke IndexedDB jika device offline & auto-flush saat online
     transport: makeBrowserOfflineTransport(makeFetchTransport),
     initialScope: {
       tags: {
         app_name: "my-client-app",
         repository: "https://github.com/your-org/my-client-app",
         verification_command: "npm test",
         branch: "main",
         device_platform: "android", // atau diisi dinamis via Capacitor Device.getInfo()
         device_model: "Samsung SM-A055F",
         is_native: "true",
       },
     },
   });
   ```

---

## 🔍 Prosedur Verifikasi End-to-End

Setelah konfigurasi diterapkan pada aplikasi:

1. **Trigger exception uji coba** dari aplikasi (misal `php artisan sentry:test` atau endpoint testing).
2. **Periksa log SRE Context Shipper**:
   ```bash
   docker logs --tail 20 $(docker ps -q --filter "name=context-shipper")
   ```
   *Tanda Berhasil:*
   ```text
   INFO openobserve_shipper::api: Sentry envelope accepted from SDK event_id=... app_name=<app>
   INFO openobserve_shipper::dispatchers::webhook: Dispatching incident payload to agent webhook event_id=... target=https://agent.yourdomain.com/api/v1/events
   INFO openobserve_shipper::dispatchers::webhook: Agent webhook response received status=202 Accepted
   ```
3. **Periksa log AI Agent (Aina)**:
   ```bash
   docker logs --tail 20 $(docker ps -q --filter "name=aina")
   ```
   *Tanda Berhasil:*
   ```text
   INFO aina::adapters::driving::web::controllers::events: Enqueued incoming event ... to chat 6281234567890@s.whatsapp.net
   INFO aina::adapters::driven::whatsmeow_http: Sending WhatsApp message ...
   ```

---

## 📚 Dokumen Referensi

- [SRE Telemetry & Ingestion Schema](./references/telemetry-schema.md): Rincian spesifikasi payload JSON dan aturan inferensi.
