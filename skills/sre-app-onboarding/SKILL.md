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

## 🏷️ Tag Telemetri Mandiri (Wajib Diterapkan)

Sematkan tag berikut pada inisialisasi Sentry SDK:

| Tag Key | Wajib/Rekomendasi | Deskripsi | Contoh Nilai |
| :--- | :--- | :--- | :--- |
| `app_name` | **Wajib** | Slug nama aplikasi | `my-app`, `billing-service` |
| `repository` | **Sangat Dianjurkan** | URL repositori GitHub | `https://github.com/your-org/my-app` |
| `verification_command` | **Sangat Dianjurkan** | Perintah test untuk AI Agent | `php artisan test`, `npm test`, `pytest` |
| `branch` | Opsional | Target branch default | `main` |

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
   })
   sentry.ConfigureScope(func(scope *sentry.Scope) {
       scope.SetTag("app_name", "my-go-service")
       scope.SetTag("repository", "https://github.com/your-org/my-go-service")
       scope.SetTag("verification_command", "go test ./...")
       scope.SetTag("branch", "main")
   })
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
