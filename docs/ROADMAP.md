# 🗺️ Implementation Roadmap: OpenObserve SRE Hub

Roadmap ini berfokus pada pembangunan **Sensor & Observability Hub (`openobserve-sre`)** dan integrasinya dengan **Autonomous Coding Agent** secara *agent-agnostic*.

---

## 📅 Roadmap Tahapan Proyek

```mermaid
timeline
    title Tahapan Implementasi openobserve-sre
    Fase 1 : Ingestion Setup : Deploy OpenObserve di Coolify : Konfigurasi Vector (Coolify & cPanel)
    Fase 2 : Alerting Engine : Template Alert Dinamis : SQL Query Deteksi Fatal/500
    Fase 3 : Context Shipper : Dedup & Cooldown Engine : Modular apps.d/ Registry
    Fase 4 : Agent Integration : Bridge Webhook ke Coding Agent : 1-Tap WA Approval & Auto-Deploy
```

---

## 🛠️ Rincian Checklist Implementasi

### Fase 1: Setup OpenObserve & Zero-Touch Collectors (Hari 1)
* [ ] Jalankan OpenObserve menggunakan `docker-compose.yml` di Coolify / Docker VPS.
* [ ] Pasang Vector di Coolify menggunakan `collectors/vector-coolify.yaml` (membaca `/var/run/docker.sock`).
* [ ] Pasang Vector/FluentBit di cPanel menggunakan `collectors/vector-cpanel.yaml` (tailing `error_log`).
* [ ] Uji kirim log dari minimal 2 aplikasi berbeda dan pastikan data muncul di dashboard OpenObserve.

### Fase 2: Aturan Deteksi & Template Alert (Hari 2)
* [ ] Buat Stream Template di OpenObserve (`alerts/openobserve-template.json`) yang menyertakan placeholder `{alert_name}`, `{stream_name}`, dan `{rows:5}`.
* [ ] Daftarkan SQL Alert untuk deteksi insiden:
  - `alerts/php-fatal-errors.sql`: Menangkap Fatal Error, Uncaught Exception, dan Syntax Error di PHP/cPanel.
  - `alerts/nodejs-unhandled.sql`: Menangkap UnhandledPromiseRejection, uncaughtException, dan 500 error di Node/Python/Go.
* [ ] Verifikasi webhook OpenObserve berhasil menembak endpoint lokal / webhook tester.

### Fase 3: Context Shipper & Modular Registry (Hari 3)
* [ ] Kompilasi dan deploy service `shipper/` (Rust):
  - Menerima webhook alert dari OpenObserve.
  - Menghitung signature hash insiden `hash(app + file + line + error)`.
  - Menerapkan cooldown 30 menit untuk mencegah banjir token dan duplikasi.
  - Memindai konfigurasi modular dari direktori `apps.d/*.yaml`.
  - Merakit JSON payload terstandar sesuai [docs/CONTEXT_SPEC.md](CONTEXT_SPEC.md).

### Fase 4: Integrasi Agent-Agnostic & Human-in-the-Loop (Hari 4)
* [ ] Arahkan output Shipper ke endpoint Coding Agent pilihan Anda (`AGENT_TARGET_URL`).
* [ ] Uji coba skenario insiden tiruan (simulasi bug sederhana):
  1. Trigger error buatan di salah satu app test.
  2. OpenObserve menangkap error dalam < 15 detik.
  3. Shipper merakit payload dan memicu Coding Agent.
  4. Coding Agent meng-clone branch hotfix, membuat perbaikan, menjalankan test suite, dan membuka PR di GitHub.
  5. Notifikasi interaktif masuk ke WhatsApp/Telegram.
* [ ] Hubungkan aksi konfirmasi `[Approve]` ke Coolify Deploy Webhook atau Git pull hook cPanel.

---

## 🎯 Definisi Keberhasilan (Done Criteria)
1. **Zero Code Changes:** 20+ aplikasi di cPanel dan Coolify terhubung ke observabilitas tanpa mengubah satu baris pun kode pada aplikasi tersebut.
2. **Standardized Context:** Payload insiden yang dihasilkan bersifat *agent-agnostic* dan siap dieksekusi oleh Coding Agent mana pun.
3. **No Duplicate Invocations:** Tidak terjadi pemanggilan agen berulang untuk error yang identik dalam kurun waktu 30 menit.
