# SRE Telemetry & Ingestion Schema

Dokumentasi spesifikasi format DSN, tag telemetri mandiri (*self-describing tags*), dan kontrak payload insiden ke AI Agent (Aina).

---

## 1. DSN (Data Source Name) Format

Seluruh aplikasi yang terhubung ke OpenObserve SRE Gateway menggunakan endpoint DSN seragam:

```text
https://<public_key>@sre.yourdomain.com/<app_name>
```

- `<public_key>`: Nilai sembarang string alfanumerik (contoh: `public`, `sre`, atau nama app). Gateway mengekstrak key ini sesuai standar protokol Sentry.
- `<app_name>`: Pengenal unik aplikasi dalam huruf kecil dan kebab-case (contoh: `my-app`, `billing-service`, `pos-kasir`). Bagian ini menjadi fallback project identifier di gateway.

Contoh:
```env
SENTRY_DSN=https://public@sre.yourdomain.com/my-app
```

---

## 2. Self-Describing Tags Specification

Agar SRE Gateway dan Coding Agent (Aina) dapat bekerja secara zero-config tanpa registri file lokal, setiap aplikasi WAJIB menyertakan tag telemetri mandiri pada Sentry SDK client.

| Tag Key | Wajib / Opsional | Tipe Data | Deskripsi | Contoh |
| :--- | :--- | :--- | :--- | :--- |
| `app_name` | **Sangat Dianjurkan** | `string` | Nama unik aplikasi / microservice | `"my-app"`, `"billing-service"` |
| `repository` | **Sangat Dianjurkan** | `string` | URL lengkap GitHub repositori | `"https://github.com/your-org/my-app"` |
| `verification_command` | **Sangat Dianjurkan** | `string` | Perintah test/verifikasi untuk coding agent | `"php artisan test"`, `"npm test"`, `"pytest"` |
| `branch` | Opsional | `string` | Branch target untuk investigasi & fix | `"main"`, `"master"` |
| `environment` | Opsional | `string` | Lingkungan runtime aplikasi | `"production"`, `"staging"`, `"development"` |

---

## 3. Resolusi Metadata Tiga Tingkat (Resolution Precedence)

Ketika insiden masuk ke SRE Context Shipper, metadata diekstrak dengan hierarki:

```
[Tier 1: Client Tags] (Prioritas Tertinggi)
       │
       ▼ (jika tidak ada)
[Tier 2: Convention Inference] (DSN project path + platform heuristics)
       │
       ▼ (jika ada file lokal)
[Tier 3: Host File Override] (/app/apps.d/<app>.yaml di host volume)
```

### Aturan Inferensi Tier 2:
- **`app_name`**: Diambil dari segment terakhir DSN URL (contoh: `/my-app` ➔ `my-app`).
- **`repository`**: `https://github.com/${DEFAULT_GITHUB_ORG:-ihkaru}/<app_name>`.
- **`verification_command`**:
  - `php` / `laravel` ➔ `php artisan test`
  - `node` / `javascript` / `typescript` ➔ `npm test`
  - `python` ➔ `pytest`
  - `go` ➔ `go test ./...`
  - `rust` ➔ `cargo test`

---

## 4. Kontrak Payload Insiden ke Aina (`/api/v1/events`)

SRE Context Shipper mengubah envelope Sentry menjadi format insiden standar yang langsung dipahami Aina:

```json
{
  "source": "openobserve-sre",
  "severity": "critical",
  "service": "my-app",
  "title": "Exception: Division by zero (/app/Http/Controllers/OrderController.php:42)",
  "details": "Stack trace:\n#0 /app/Http/Controllers/OrderController.php(42): ...",
  "repository": "https://github.com/your-org/my-app",
  "target_chat": "6281234567890@s.whatsapp.net"
}
```

Header autentikasi yang dikirim shipper:
- `X-API-Key: ${AINA_API_KEY}`
- `Content-Type: application/json`

Respon sukses dari Aina: `HTTP 202 Accepted`.
