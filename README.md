# EMQX Auth Service - MQTT User Management

A high-performance authentication and authorization service for MQTT clients in the IoTNet ecosystem. Built with Rust and Actix-web.

## Features

- REST API for MQTT user management (create, list, delete)
- JWT token generation for authenticated sessions
- PostgreSQL persistence
- RESTful API with API key validation
- OpenTelemetry integration for observability
- Structured error handling and logging

## Prerequisites

- Rust 1.70+ (with Cargo)
- Docker 20.10+
- Docker Compose 2.0+
- Make

## Quick Start

### Configuration

Create a `.env` file:

```bash
# Database (PostgreSQL)
DATABASE_URL=postgresql://user:password@localhost:5432/emqx_auth
# Or use individual DB_* variables
DB_HOST=localhost
DB_PORT=5432
DB_NAME=emqx_auth
DB_USER=postgres
DB_PASSWORD=your_password
DB_SCHEMA=public

# Authentication
SECRET_KEY=<generate-with: make key>
API_KEY=<generate-with: make key>

# Logging
RUST_LOG=info

# Rate limiting
MQTT_AUTH_RATE_LIMIT=100

# OpenTelemetry (optional)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=emqx-auth-service
```

Generate a secure key:

```bash
make key
```

### Build

**Local build with Docker:**

```bash
make build
```

This creates a local Docker image: `emqx-auth-service:latest`

**Or build directly with Cargo:**

```bash
cargo build --release
```

### Run

**With Docker Compose:**

```bash
docker compose up -d
```

**Or direct execution:**

```bash
# Start PostgreSQL database
docker compose up -d postgres

# Run the application
cargo run --release
```

### Verify

```bash
curl http://localhost:5500/
# Response: OK
```

## API Endpoints

All endpoints (except Health Check) require the `x-api-key` header.

### Health Check

```
GET /
```

**Response:** `OK` (text/plain)

### Create MQTT Client

```
POST /mqtt/create
Content-Type: application/json
x-api-key: <your-api-key>

{
  "username": "<client_name>",
  "password": "<client_password>",
  "is_superuser": false
}
```

**Response: 200 OK**
```json
{
  "success": true,
  "message": "User created successfully"
}
```

**Error Responses:**
- `400 Bad Request` - Invalid input (empty username/password)
- `401 Unauthorized` - Missing or invalid API key
- `409 Conflict` - Username already exists

### List MQTT Clients

```
GET /mqtt
x-api-key: <your-api-key>
```

**Query Parameters (optional):**
- `limit` (default: 100) - Maximum number of users to return
- `offset` (default: 0) - Number of users to skip

**Response: 200 OK**
```json
{
  "success": true,
  "message": "User list retrieved successfully",
  "data": {
    "users": [
      {
        "id": 1,
        "username": "device_001",
        "is_superuser": false
      }
    ],
    "total": 1,
    "limit": 100,
    "offset": 0
  }
}
```

### Delete MQTT Client

```
DELETE /mqtt/{username}
x-api-key: <your-api-key>
```

**Response: 200 OK**
```json
{
  "success": true,
  "message": "User deleted successfully"
}
```

**Error Responses:**
- `401 Unauthorized` - Missing or invalid API key
- `404 Not Found` - User not found

### Generate JWT Token

Generate a JWT token for EMQX authentication. The token can be used as the password when connecting to the MQTT broker.

```
POST /mqtt/jwt
Content-Type: application/json
x-api-key: <your-api-key>

{
  "username": "<client_name>"
}
```

**Response: 200 OK**
```json
{
  "success": true,
  "message": "JWT token generated successfully",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_at": "2026-03-20T12:00:00Z"
  }
}
```

**JWT Token Configuration:**
- Algorithm: HS256 (HMAC-SHA256)
- Issuer: `broker.i-ot.net`
- Audience: `mqtt`
- Expiration: 24 hours
- Secret: Configured via `SECRET_KEY` environment variable

**Usage with EMQX:**
```python
import paho.mqtt.client as mqtt

# Get JWT token from auth service
token_response = requests.post(
    "http://localhost:5500/mqtt/jwt",
    headers={"x-api-key": "YOUR_API_KEY"},
    json={"username": "client_001"}
)
jwt_token = token_response.json()["data"]["token"]

# Connect to EMQX using JWT as password
client = mqtt.Client(client_id="client_001")
client.username_pw_set("client_001", jwt_token)
client.connect("broker.i-ot.net", 1883)
```

## Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Yes* | - |
| `DB_HOST` | Database host | No | `localhost` |
| `DB_PORT` | Database port | No | `5432` |
| `DB_NAME` | Database name | No | `emqx_auth` |
| `DB_USER` | Database user | No | `postgres` |
| `DB_PASSWORD` | Database password | No | - |
| `DB_SCHEMA` | Database schema | No | `public` |
| `DB_MAX_CONNECTIONS` | Max DB connections | No | `10` |
| `DB_CONNECT_TIMEOUT` | Connection timeout (ms) | No | `5000` |
| `DB_IDLE_TIMEOUT` | Idle connection timeout (ms) | No | `60000` |
| `SECRET_KEY` | SHA256 hash for JWT signing | Yes | - |
| `API_KEY` | API key for request authentication | Yes | - |
| `RUST_LOG` | Logging level | No | `info` |
| `MQTT_AUTH_RATE_LIMIT` | Rate limit (requests/min) | No | `100` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry OTLP endpoint | No | `http://localhost:4317` |
| `OTEL_SERVICE_NAME` | Service name for telemetry | No | `emqx-auth-service` |

*Either `DATABASE_URL` or the individual `DB_*` variables must be provided.

## Make Commands

```bash
make help              # Show available commands
make build             # Build Docker plugin image
make key               # Generate secure SHA256 hash
make docker run        # Start services
make docker stop       # Stop services
make docker ps         # Show running containers
```

## Project Structure

```
src/
├── main.rs                    # Entry point and route definitions
├── config/                    # Application configuration
│   ├── mod.rs
│   └── database_config.rs
├── presentation/              # HTTP layer (handlers, middleware)
│   ├── handlers/
│   │   └── rest/              # REST API handlers
│   │       ├── create_user_handler.rs
│   │       ├── delete_user_handler.rs
│   │       ├── list_users_handler.rs
│   │       └── jwt_handler.rs
│   └── middleware/            # API key auth, metrics
├── application/               # Business logic (use cases)
├── domain/                    # Domain models and interfaces
└── infrastructure/            # Database, repositories, telemetry
```

## API Documentation

- **Interactive UI:** `/openapi` (Scalar)
- **OpenAPI JSON:** `/api-docs/openapi.json`
- **Detailed docs:** See [docs/api_documentation.md](docs/api_documentation.md)

## Production Setup

For production deployment guidance, see:
- [Production Setup Guide](docs/PRODUCTION_SETUP.md)
- [SSL/TLS Certificate Setup](docs/CERT_SETUP.md)
- [OpenTelemetry Integration](docs/OPENTELEMETRY_GUIDE.md)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

For issues or questions, create an issue in the repository.

## Documentation Standards

This service follows strict documentation standards to ensure accuracy:

1. **Code is Source of Truth**: API contracts must follow routes and handlers defined in `src/main.rs`
2. **No Fictional Features**: Documentation must not describe endpoints, environment variables, or storage engines that do not exist in the code
3. **OpenAPI Reference**: Runtime OpenAPI specification at `/openapi` and `/api-docs/openapi.json` is the authoritative API reference
4. **Response Accuracy**: All example responses must match actual handler return messages exactly
5. **Configuration Accuracy**: All environment variables must match `AppConfig` and `.env.example`

When updating documentation:
- Cross-check endpoints against `src/main.rs` route definitions
- Cross-check response messages against handler implementations
- Cross-check environment variables against `src/config/mod.rs`
- Verify OpenAPI spec matches documented behavior
