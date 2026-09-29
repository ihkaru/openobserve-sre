# 📦 Spesifikasi Skema Konteks (Context Payload Specification)

Dokumen ini mendefinisikan skema JSON dari payload konteks yang dirakit oleh **Custom Context Shipper** dan dikirimkan ke **Coding Agent API**.

Konteks yang kaya (*high signal, low noise*) sangat penting agar Coding Agent dapat mereproduksi masalah dan memprogram perbaikan secara akurat tanpa melakukan halusinasi.

---

## 1. Skema JSON Konteks Lengkap

```json
{
  "event_id": "evt_9f81a7d6e4b2",
  "timestamp": "2026-09-30T06:30:15Z",
  "app_metadata": {
    "app_name": "toko-online-api",
    "environment": "production",
    "platform": "coolify", 
    "language": "php",
    "framework": "laravel",
    "repository": {
      "provider": "github",
      "url": "https://github.com/myorg/toko-online-api",
      "default_branch": "main",
      "target_branch": "hotfix/auto-heal-evt_9f81a7d6e4b2"
    },
    "runtime": {
      "version": "8.3.10",
      "container_id": "c1a938e7db91"
    }
  },
  "incident": {
    "severity": "CRITICAL",
    "error_type": "ErrorException",
    "error_message": "Undefined array key \"customer_tax_id\"",
    "file_path": "app/Services/CheckoutService.php",
    "line_number": 84,
    "stack_trace": [
      "app/Services/CheckoutService.php:84 in App\\Services\\CheckoutService::calculateTotal",
      "app/Http/Controllers/OrderController.php:32 in App\\Http\\Controllers\\OrderController::checkout",
      "vendor/laravel/framework/src/Illuminate/Routing/Controller.php:54 in call_user_func_array"
    ]
  },
  "telemetry_context": {
    "openobserve_stream": "coolify-apps",
    "trigger_alert_name": "production_php_fatal_errors",
    "surrounding_logs": [
      "[2026-09-30 06:30:14] INFO: Checkout initiated for user_id=402, cart_id=1089",
      "[2026-09-30 06:30:14] INFO: Applying discount coupon: FLASH2026",
      "[2026-09-30 06:30:15] ERROR: Undefined array key \"customer_tax_id\" at /var/www/html/app/Services/CheckoutService.php:84"
    ],
    "sample_http_request": {
      "method": "POST",
      "path": "/api/v1/checkout",
      "status_code": 500,
      "user_agent": "Mozilla/5.0 ... Mobile Safari",
      "payload_sanitized": {
        "cart_id": 1089,
        "coupon_code": "FLASH2026",
        "payment_method": "qris"
      }
    }
  },
  "code_context": {
    "target_file_snippet": {
      "start_line": 74,
      "end_line": 94,
      "content": "74:     public function calculateTotal(array $cart, array $customerData): float\n75:     {\n76:         $subtotal = $this->calculateSubtotal($cart);\n77:         $discount = $this->discountCalculator->apply($subtotal, $cart['coupon'] ?? null);\n78: \n79:         // Apply tax\n80:         $tax = 0.0;\n81:         if (config('tax.enabled')) {\n82:             // BUG: User registered without tax ID causes undefined array key\n83:             $tax = $this->taxEngine->compute(\n84:                 $subtotal - $discount,\n85:                 $customerData['customer_tax_id']\n86:             );\n87:         }\n88: \n89:         return $subtotal - $discount + $tax;\n90:     }"
    }
  },
  "remediation_instructions": {
    "objective": "Fix the undefined array key while preserving existing tax computation logic.",
    "acceptance_criteria": [
      "Null-safe access: handle cases where customer_tax_id is missing or null.",
      "Add or update unit test in tests/Unit/CheckoutServiceTest.php.",
      "Ensure all test suites pass."
    ],
    "verification_command": "php artisan test --filter=CheckoutServiceTest"
  }
}
```

---

## 2. Penjelasan Field Kunci untuk Agent

| Field | Sumber Data | Alasan Dibutuhkan oleh Agent |
| :--- | :--- | :--- |
| `incident.file_path` & `line_number` | OpenObserve Regex / OTel Exception | Agar agent langsung menuju baris kode yang rusak tanpa buang token untuk eksplorasi repositori. |
| `telemetry_context.surrounding_logs` | OpenObserve `{rows:N}` | Memberikan alur urutan (*breadcrumbs*) kejadian sebelum crash terjadi. |
| `code_context.target_file_snippet` | GitHub API via Shipper | Memberikan gambaran kode lokal secara instan tanpa perlu menunggu agent selesai meng-clone seluruh repo. |
| `remediation_instructions.verification_command` | Konfigurasi mapping per bahasa di Shipper | Memberi tahu agent perintah apa yang harus dijalankan untuk memverifikasi perbaikan (`npm test`, `pytest`, `go test`). |
| `app_metadata.repository.target_branch` | Otomatis dibuat oleh Shipper | Menjamin agent bekerja di branch hotfix terisolasi, bukan langsung ke `main`. |
