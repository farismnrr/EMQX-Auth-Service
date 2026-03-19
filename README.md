# EMQX Auth Service - Client Management Service

A high-performance authentication and authorization service for MQTT clients in the IoTNet ecosystem. Built with Rust and Actix-web.

## Features

- MQTT client credential management (create, list, delete)
- Client authentication with fast password verification
- JWT token generation for authenticated sessions
- Access Control List (ACL) validation
- **MQTT RPC API** - Backend-to-service communication via MQTT topics
- SQLite persistence for fast authentication and ACL checks
- RESTful API with API key validation
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
DB_PATH=./data/mqtt_auth.sqlite
SECRET_KEY=<generate-with: make key>
API_KEY=<generate-with: make key>
MQTT_PASS_ENCRYPTION_KEY=<generate-with: openssl rand -hex 32>
LOG_LEVEL=info
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

**Pull and run from GHCR:**

```bash
docker run -d \
  --name auth-plugin \
  -p 5500:5500 \
  -v ./rocksdb-data:/data \
  -e DB_PATH=/data/your_db \
  -e SECRET_KEY=<your-secret-key> \
  -e API_KEY=<your-api-key> \
  -e LOG_LEVEL=info \
  ghcr.io/farismnrr/emqx-auth-service:v0.1.0
```

**Or direct execution:**

```bash
# Start RocksDB service
docker compose up -d rocksdb

# Run the application
cargo run --release
```

### Verify

```bash
curl http://localhost:5500/
# Response: OK
```

## API Endpoints

All endpoints require the `x-api-key` header.

### Health Check

```
GET /
```

### Create MQTT Client

```
POST /mqtt/create
Content-Type: application/json

{
  "username": "<client_name>",
  "password": "<client_password>",
  "is_superuser": false
}

Response: 200 OK
{
  "success": true,
  "message": "User MQTT created successfully"
}
```

### List MQTT Clients

```
GET /mqtt

Response: 200 OK
{
  "success": true,
  "message": "User MQTT list retrieved successfully",
  "data": {
    "users": [...]
  }
}
```

### Authenticate Client

The `/mqtt/check` endpoint supports two authentication methods:

#### Method 1: Credentials Authentication

```
POST /mqtt/check
Content-Type: application/json

{
  "username": "<client_name>",
  "password": "<client_password>",
  "method": "credentials"
}

Response: 200 OK
{
  "success": true,
  "message": "User MQTT is active",
  "result": "allow"
}
```

#### Method 2: JWT Authentication

```
POST /mqtt/check
Content-Type: application/json

{
  "username": "<client_name>",
  "password": "",
  "method": "jwt"
}

Response: 200 OK
{
  "success": true,
  "message": "User MQTT is active",
  "result": "allow",
  "data": {
    "token": "<jwt_token_here>"
  }
}
```

### Generate JWT Token

Generate a JWT token for EMQX authentication. The token can be used as the password when connecting to the MQTT broker.

```
POST /mqtt/jwt
Content-Type: application/json
x-api-key: <your-api-key>

{
  "username": "<client_name>"
}

Response: 200 OK
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

### Check ACL Permission

```
POST /mqtt/acl
Content-Type: application/json

{
  "username": "<client_name>",
  "topic": "<topic_name>"
}

Response: 200 OK
{
  "success": true,
  "message": "User has access",
  "result": "allow"
}
```

### Delete MQTT Client

```
DELETE /mqtt/{<client_name>}

Response: 200 OK
{
  "success": true,
  "message": "User mqtt deleted successfully"
}
```

### Get MQTT Client by ID

```
GET /mqtt/{id}

Response: 200 OK
{
  "success": true,
  "message": "User MQTT retrieved successfully",
  "data": {
    "id": 1,
    "username": "client_id",
    "is_superuser": false
  }
}
```

---

## MQTT RPC API (Backend Integration)

The MQTT RPC API enables backend-to-auth-service communication over MQTT. It uses a request-reply pattern with per-requester reply topics.

### Enable MQTT RPC API

Set the following environment variables:

```bash
MQTT_ADMIN_ENABLED=true
MQTT_BROKER_HOST=localhost
MQTT_BROKER_PORT=1883
MQTT_ADMIN_ALLOWED_REQUESTERS=iotnet-backend  # Comma-separated list
```

### Request/Response Format

All MQTT RPC messages use JSON format with envelope structure:

**Request Envelope:**
```json
{
  "schema_version": 1,
  "request_id": "unique-uuid-here",
  "reply_to": "iotnet/auth/replies/iotnet-backend",
  "requested_by": "iotnet-backend",
  "api_key": "your-api-key-here",
  "timestamp": 1710847200000,
  // ... operation-specific fields
}
```

**Note:** `timestamp` must be epoch **milliseconds** (not seconds). Requests older than 5 minutes will be rejected with `REQUEST_EXPIRED`.

**Response Envelope:**
```json
{
  "schema_version": 1,
  "request_id": "unique-uuid-here",
  "success": true,
  "message": "Operation completed successfully",
  "data": { /* operation-specific data */ }
}
```

### MQTT RPC Topics

| Topic | Method | Description |
|-------|--------|-------------|
| `iotnet/auth/commands/users.create` | PUBLISH | Create a new MQTT user |
| `iotnet/auth/commands/users.delete` | PUBLISH | Delete an existing MQTT user |
| `iotnet/auth/commands/users.get` | PUBLISH | Get user by username |
| `iotnet/auth/commands/users.list` | PUBLISH | List MQTT users |
| `iotnet/auth/commands/tokens.issue` | PUBLISH | Issue JWT token for user |
| `iotnet/auth/commands/tokens.verify` | PUBLISH | Verify user password |

### Response Topics

Responses are published to the `reply_to` topic specified in the request:
- Format: `iotnet/auth/replies/{requested_by}`
- Example: `iotnet/auth/replies/iotnet-backend`

### Security Model

1. **Allowed Requesters**: Only requesters in `MQTT_ADMIN_ALLOWED_REQUESTERS` can issue commands
2. **Reply Topic Binding**: `reply_to` must exactly match `iotnet/auth/replies/{requested_by}`
3. **Timestamp Validation**: Requests older than 5 minutes are rejected (prevents replay attacks)
4. **Schema Enforcement**: `schema_version` must be `1`.

### Example: Create User via MQTT RPC

```json
// Publish to: iotnet/auth/commands/users.create
{
  "schema_version": 1,
  "request_id": "req-123-abc",
  "reply_to": "iotnet/auth/replies/iotnet-backend",
  "requested_by": "iotnet-backend",
  "api_key": "your-api-key-here",
  "timestamp": 1710847200000,
  "username": "new_user",
  "password": "secure_password_123",
  "is_superuser": false
}

// Response on: iotnet/auth/replies/iotnet-backend
{
  "schema_version": 1,
  "request_id": "req-123-abc",
  "success": true,
  "message": "User created successfully"
}
```

### Example: Issue Token via MQTT RPC

```json
// Publish to: iotnet/auth/commands/tokens.issue
{
  "schema_version": 1,
  "request_id": "req-456-def",
  "reply_to": "iotnet/auth/replies/iotnet-backend",
  "requested_by": "iotnet-backend",
  "api_key": "your-api-key-here",
  "timestamp": 1710847200000,
  "username": "device_001"
}

// Response on: iotnet/auth/replies/iotnet-backend
{
  "schema_version": 1,
  "request_id": "req-456-def",
  "success": true,
  "message": "Token issued successfully",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

### Error Codes

When `success` is `false`, a `code` field is included:

| Code | Description |
|------|-------------|
| `BAD_REQUEST` | Missing required envelope fields |
| `UNAUTHORIZED_COMMAND` | Invalid API key or requester not allowed |
| `INVALID_REPLY_TOPIC` | `reply_to` does not match `requested_by` binding |
| `REQUEST_EXPIRED` | Timestamp window validation failed |
| `UNSUPPORTED_SCHEMA_VERSION` | `schema_version` is not `1` |
| `UNKNOWN_COMMAND` | Command in topic is not recognized |
| `VALIDATION_ERROR` | Operation payload format is invalid |
| `USER_NOT_FOUND` | Target user does not exist |
| `USER_ALREADY_EXISTS` | Cannot create duplicate user |
| `INVALID_CREDENTIALS` | Password verification failed |
| `INTERNAL_ERROR` | Unexpected server error |

For complete MQTT RPC API documentation, see [api_documentation.md](api_documentation.md#mqtt-rpc-api-backend-integration).

## Environment Variables

| Variable             | Description                        | Required | Default     |
| -------------------- | ---------------------------------- | -------- | ----------- |
| `DB_PATH`            | SQLite database file path          | No       | `mqtt_auth.sqlite` |
| `SECRET_KEY`         | SHA256 hash for JWT signing        | Yes      | -           |
| `API_KEY`            | API key for request authentication | Yes      | -           |
| `LOG_LEVEL`          | Logging level (info, debug, warn)  | No       | `info`      |
| `MQTT_ADMIN_ENABLED` | Enable MQTT RPC API                | No       | `false`     |
| `MQTT_BROKER_HOST`   | MQTT broker host                   | No       | `localhost` |
| `MQTT_BROKER_PORT`   | MQTT broker port                   | No       | `1883`      |
| `MQTT_USE_TLS`       | Use TLS for MQTT connection        | No       | `false`     |
| `MQTT_ADMIN_USERNAME`| MQTT client username (superuser)   | No       | -           |
| `MQTT_ADMIN_PASSWORD`| MQTT client password               | No       | -           |
| `MQTT_ADMIN_ALLOWED_REQUESTERS` | Comma-separated list of allowed requesters | No | `iotnet-backend` |
| `MQTT_PASS_ENCRYPTION_KEY` | AES-256 encryption key (64 hex chars) | Yes | -   |
| `MQTT_TOPIC_AUTH_COMMAND_PREFIX` | MQTT RPC command topic prefix | No | `iotnet/auth/commands` |
| `MQTT_TOPIC_AUTH_REPLY_PREFIX` | MQTT RPC reply topic prefix | No | `iotnet/auth/replies` |

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
├── main.rs                    # Entry point
├── server.rs                  # HTTP server configuration
├── handler/                   # Request handlers
│   ├── rest/                  # REST API handlers
│   │   ├── mod.rs
│   │   ├── create_user.rs
│   │   ├── check_login.rs
│   │   ├── check_acl.rs
│   │   ├── get_credentials.rs
│   │   ├── list_users.rs
│   │   └── get_user_by_id.rs
│   └── mqtt/                  # MQTT RPC handlers
│       ├── mod.rs
│       ├── common.rs
│       ├── create_user.rs
│       ├── delete_user.rs
│       ├── get_user.rs
│       ├── issue_token.rs
│       └── verify_password.rs
├── services/                  # Business logic
├── repositories/              # Data access layer
├── middleware/                # HTTP middleware
├── infrastructure/            # Database utilities
├── entities/                  # Domain models
├── dtos/                      # Data transfer objects
└── utils/                     # Utilities
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

For issues or questions, create an issue in the repository.
