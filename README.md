# EMQX Auth Service - Client Management Service

A high-performance authentication and authorization service for MQTT clients in the IoTNet ecosystem. Built with Rust and Actix-web.

## Features

- MQTT client credential management (create, list, delete)
- Client authentication with fast password hashing
- JWT token generation for authenticated sessions
- Access Control List (ACL) validation
- **MQTT Admin API** - Manage users via MQTT topics (create, delete, list, get by ID)
- MySQL persistence for fast authentication and ACL checks
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
DB_PATH=./rocksdb-data/your_db
SECRET_KEY=<generate-with: make key>
API_KEY=<generate-with: make key>
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

All endpoints require the `Authorization Bearer` header.

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

Response: 201 OK
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
    "is_superuser": false,
    "is_deleted": false
  }
}
```

---

## MQTT Admin API

The MQTT Admin API allows you to manage MQTT users through MQTT topics instead of HTTP. This is useful for administrative operations performed directly over MQTT.

### Enable MQTT Admin API

Set the following environment variables:

```bash
MQTT_ADMIN_ENABLED=true
MQTT_BROKER_HOST=localhost
MQTT_BROKER_PORT=1883
```

### Request/Response Format

All MQTT messages use JSON format:

**Request:**
```json
{
  "request_id": "unique-uuid-here",
  // ... operation-specific fields
}
```

**Response:**
```json
{
  "request_id": "unique-uuid-here",
  "success": true,
  "message": "Operation completed successfully",
  "data": { /* operation-specific data */ }
}
```

### MQTT Admin Topics

| Topic | Method | Description |
|-------|--------|-------------|
| `admins/users/create` | PUBLISH | Create a new MQTT user |
| `admins/users/delete` | PUBLISH | Delete an existing MQTT user |
| `admins/users` | PUBLISH | List users with pagination |
| `admins/users/{user_id}` | SUBSCRIBE | Get user by ID |

### Response Topics

| Topic | Description |
|-------|-------------|
| `admins/users/create/response` | Response for create operations |
| `admins/users/delete/response` | Response for delete operations |
| `admins/users/list/response` | Response for list operations |
| `admins/users/detail/response` | Response for get by ID operations |

### Example: Create User via MQTT

```json
// Publish to: admins/users/create
{
  "request_id": "req-123-abc",
  "username": "new_user",
  "password": "secure_password_123",
  "is_superuser": false
}

// Response on: admins/users/create/response
{
  "request_id": "req-123-abc",
  "success": true,
  "message": "User created successfully",
  "data": {
    "id": 42,
    "username": "new_user",
    "is_superuser": false
  }
}
```

### Example: List Users with Pagination

```json
// Publish to: admins/users
{
  "request_id": "req-789-ghi",
  "page": 1,
  "page_size": 10
}

// Response on: admins/users/list/response
{
  "request_id": "req-789-ghi",
  "success": true,
  "message": "Users retrieved successfully",
  "data": {
    "users": [...],
    "pagination": {
      "total": 50,
      "page": 1,
      "page_size": 10,
      "total_pages": 5
    }
  }
}
```

For complete MQTT Admin API documentation, see [api_documentation.md](api_documentation.md#mqtt-admin-api).

## Environment Variables

| Variable             | Description                        | Required | Default     |
| -------------------- | ---------------------------------- | -------- | ----------- |
| `MYSQL_HOST`         | MySQL server host                  | Yes      | -           |
| `MYSQL_PORT`         | MySQL server port                  | Yes      | -           |
| `MYSQL_DATABASE`     | MySQL database name                | Yes      | -           |
| `MYSQL_USER`         | MySQL username                     | Yes      | -           |
| `MYSQL_PASSWORD`     | MySQL password                     | Yes      | -           |
| `SECRET_KEY`         | SHA256 hash for JWT signing        | Yes      | -           |
| `API_KEY`            | API key for request authentication | Yes      | -           |
| `LOG_LEVEL`          | Logging level (info, debug, warn)  | No       | `info`      |
| `MQTT_ADMIN_ENABLED` | Enable MQTT Admin API              | No       | `false`     |
| `MQTT_BROKER_HOST`   | MQTT broker host                   | No       | `localhost` |
| `MQTT_BROKER_PORT`   | MQTT broker port                   | No       | `1883`      |

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
├── services/                  # Business logic
├── repositories/              # Data access layer
├── middleware/                # HTTP middleware
├── infrastructure/            # RocksDB utilities
├── entities/                  # Domain models
├── dtos/                      # Data transfer objects
└── utils/                     # Utilities
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

For issues or questions, create an issue in the repository.
