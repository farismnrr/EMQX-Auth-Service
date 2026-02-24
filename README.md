# EMQX Auth Service - Client Management Service

A high-performance authentication and authorization service for MQTT clients in the IoTNet ecosystem. Built with Rust and Actix-web.

## Features

- MQTT client credential management (create, list, delete)
- Client authentication with fast password hashing
- JWT token generation for authenticated sessions
- Access Control List (ACL) validation
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

## Environment Variables

| Variable         | Description                        | Required |
| ---------------- | ---------------------------------- | -------- |
| `MYSQL_HOST`     | MySQL server host                  | Yes      |
| `MYSQL_PORT`     | MySQL server port                  | Yes      |
| `MYSQL_DATABASE` | MySQL database name                | Yes      |
| `MYSQL_USER`     | MySQL username                     | Yes      |
| `MYSQL_PASSWORD` | MySQL password                     | Yes      |
| `SECRET_KEY`     | SHA256 hash for JWT signing        | Yes      |
| `API_KEY`        | API key for request authentication | Yes      |
| `LOG_LEVEL`      | Logging level (info, debug, warn)  | No       |

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
