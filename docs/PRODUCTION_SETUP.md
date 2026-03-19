# 🚀 EMQX Auth Service - Production Setup Guide

Dokumen ini menjelaskan cara mengonfigurasi **EMQX Auth Service** dan **EMQX Broker (v5+)** untuk lingkungan produksi menggunakan skema keamanan **AES-256-GCM** dan **JWT**.

---

## 🔐 Production Listener Configuration (Matching Tunneling Server)

Untuk produksi, EMQX broker dikonfigurasi dengan listener berikut:

| Port  | Protocol | Description                          |
|-------|----------|--------------------------------------|
| 1883  | MQTT     | Plain MQTT (unencrypted)             |
| 8083  | WebSocket| Plain WebSocket (unencrypted)        |
| 8883  | SSL/TLS  | Secure MQTT over TLS                 |
| 8084  | WSS      | Secure WebSocket over TLS            |
| 18083 | HTTP     | Dashboard API                        |

### SSL Certificate Setup

Pastikan file sertifikat SSL tersedia di `/etc/emqx/certs/`:
- `broker.i-ot.net.crt` - SSL certificate
- `broker.i-ot.net.key` - SSL private key

---

## 🏗️ 3 Route Utama (EMQX Native)

Service ini menyediakan 3 endpoint khusus yang dirancang untuk berkomunikasi langsung dengan hook HTTP EMQX:

### 1. `POST /emqx/auth` (Authentication)
Digunakan oleh EMQX untuk memvalidasi username dan password client saat mencoba terhubung.
*   **Logika**: Service mengambil ciphertext dari database, mendekripsinya menggunakan kunci AES, dan mencocokkannya dengan password yang dikirim client.
*   **Response**: 
    ```json
    { "result": "allow", "is_superuser": false } // atau "deny"
    ```

### 2. `POST /emqx/login` (JWT Issuance)
Endpoint bagi client/aplikasi untuk mendapatkan token JWT.
*   **Fungsi**: Client mengirim kredensial (plain text), jika valid, service akan memberikan token JWT yang ditandatangani dengan `SECRET_KEY`.
*   **Response**:
    ```json
    { "result": "allow", "token": "eyJhbGci..." }
    ```

### 3. `POST /emqx/acl` (Authorization)
Digunakan oleh EMQX untuk mengecek apakah seorang user diizinkan melakukan **Publish** atau **Subscribe** ke suatu topic.
*   **Aturan Default**: 
    *   **Superuser**: Akses penuh ke semua topic.
    *   **Regular User**: Hanya diizinkan mengakses topic dengan prefix `users/{username}/`.
*   **Response**:
    ```json
    { "result": "allow" } // atau "deny"
    ```

---

## ⚙️ Konfigurasi Environment

Pastikan file `.env` di produksi memiliki variabel berikut:

```bash
# Kunci untuk menandatangani JWT (Sangat Rahasia)
SECRET_KEY=your_super_secret_jwt_key

# Kunci 64-karakter hex untuk enkripsi AES-256-GCM (Password di DB)
MQTT_PASS_ENCRYPTION_KEY=64_hex_characters_here

# API Key untuk mengamankan komunikasi EMQX -> Auth Service
API_KEY=your_internal_api_key
```

---

## 🛠️ Konfigurasi EMQX Broker (v5.x)

Untuk performa dan keamanan terbaik di produksi, gunakan konfigurasi **HOCON**. Anda bisa memuat konfigurasi ini melalui Dashboard (Access Control) atau via CLI (`emqx_ctl conf load`).

### 1. Authentication Chain (JWT & HTTP)
Susun agar EMQX mengecek JWT terlebih dahulu, baru kemudian HTTP Auth.

```hocon
authentication = [
  {
    mechanism = "jwt"
    use_jwks = false
    algorithm = "hmac-based"
    secret = "PASTE_YOUR_SECRET_KEY_HERE"
    from = "password"
    enable = true
  },
  {
    mechanism = "password_based"
    backend = "http"
    enable = true
    method = "post"
    url = "http://emqx-auth-service:5500/emqx/auth"
    headers {
      "Content-Type" = "application/json"
      "x-api-key" = "PASTE_YOUR_API_KEY_HERE"
    }
    body {
      username = "${username}"
      password = "${password}"
    }
  }
]
```

### 2. Authorization (ACL)
Pastikan `no_match = deny` untuk keamanan maksimal.

```hocon
authorization {
  no_match = "deny"
  cache { enable = true, ttl = "1m" }
  sources = [
    {
      type = "http"
      enable = true
      method = "post"
      url = "http://emqx-auth-service:5500/emqx/acl"
      headers {
        "Content-Type" = "application/json"
        "x-api-key" = "PASTE_YOUR_API_KEY_HERE"
      }
      body {
        username = "${username}"
        clientid = "${clientid}"
        topic = "${topic}"
        action = "${action}"
      }
    }
  ]
}
```

---

## 🔒 Security Best Practices

1.  **Localhost Binding**: Selalu jalankan Auth Service di port lokal (`127.0.0.1:5505`) atau di dalam Docker Network internal. Jangan expose port 5500/5505 ke internet publik.
2.  **Key Rotation**: Jika `MQTT_PASS_ENCRYPTION_KEY` bocor, Anda harus mengenkripsi ulang seluruh password di database. Jaga kunci ini dengan sangat ketat.
3.  **Audit Logs**: Cek logs secara berkala jika terjadi `Internal Server Error`. Detail kegagalan enkripsi/dekripsi hanya muncul di server logs, tidak dikirim ke client API.
4.  **Superuser**: Gunakan field `is_superuser: true` di database hanya untuk sistem backend atau admin IoTNet agar memiliki akses kontrol penuh tanpa batasan topic.
