# EMQX Auth Service API Documentation

Complete API documentation for the EMQX Auth Service.

> **Note:** This documentation is based on the actual OpenAPI specification. For the most accurate and up-to-date API docs, visit `/openapi` (Scalar UI) or `/api-docs/openapi.json`.

## Table of Contents

1. [Authentication](#authentication)
2. [Endpoints](#endpoints)
   - [Health Check](#1-health-check)
   - [Create MQTT User](#2-create-mqtt-user)
   - [List MQTT Users](#3-list-mqtt-users)
   - [Delete MQTT User](#4-delete-mqtt-user)
   - [Generate JWT Token](#5-generate-jwt-token)
3. [Schemas](#schemas)

---

## Authentication

All endpoints (except Health Check) require authentication via API Key.

**Header:** `x-api-key: <API_KEY>`

**Error Response (401 Unauthorized):**
```json
{
  "success": false,
  "message": "Unauthorized"
}
```

---

## Endpoints

### 1. Health Check

Checks if the service is up and running.

- **URL:** `/`
- **Method:** `GET`
- **Authentication:** None
- **Response:**
  - **Code:** `200 OK`
  - **Body:** `OK` (text/plain)

**Example:**
```bash
curl http://localhost:5500/
```

---

### 2. Create MQTT User

Registers a new MQTT user in the system.

- **URL:** `/mqtt/create`
- **Method:** `POST`
- **Authentication:** Required (`x-api-key`)
- **Request Body:**
  ```json
  {
    "username": "device_001",
    "password": "secure_password",
    "is_superuser": false
  }
  ```
- **Success Response (200):**
  ```json
  {
    "success": true,
    "message": "User created successfully",
    "data": null
  }
  ```
- **Error Responses:**
  - `400 Bad Request` - Invalid input (empty username/password, invalid JSON)
    ```json
    {
      "success": false,
      "message": "Validation error: username cannot be empty"
    }
    ```
  - `401 Unauthorized` - Missing or invalid API key
  - `409 Conflict` - Username already exists
    ```json
    {
      "success": false,
      "message": "User already exists"
    }
    ```
  - `500 Internal Server Error` - Database error

**Example:**
```bash
curl -X POST http://localhost:5500/mqtt/create \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{"username":"device_001","password":"secure_password","is_superuser":false}'
```

---

### 3. List MQTT Users

Retrieves a list of all registered MQTT users with pagination.

- **URL:** `/mqtt`
- **Method:** `GET`
- **Authentication:** Required (`x-api-key`)
- **Query Parameters:**
  - `limit` (optional, default: 100) - Maximum number of users to return
  - `offset` (optional, default: 0) - Number of users to skip
- **Success Response (200):**
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
- **Error Responses:**
  - `401 Unauthorized` - Missing or invalid API key
  - `500 Internal Server Error` - Database error

**Example:**
```bash
curl http://localhost:5500/mqtt \
  -H "x-api-key: YOUR_API_KEY"

# With pagination
curl "http://localhost:5500/mqtt?limit=10&offset=0" \
  -H "x-api-key: YOUR_API_KEY"
```

---

### 4. Delete MQTT User

Removes an MQTT user from the system.

- **URL:** `/mqtt/{username}`
- **Method:** `DELETE`
- **Authentication:** Required (`x-api-key`)
- **Path Parameters:**
  - `username` - The username to delete (string)
- **Success Response (200):**
  ```json
  {
    "success": true,
    "message": "User deleted successfully",
    "data": null
  }
  ```
- **Error Responses:**
  - `401 Unauthorized` - Missing or invalid API key
  - `404 Not Found` - User not found
    ```json
    {
      "success": false,
      "message": "User not found"
    }
    ```
  - `500 Internal Server Error` - Database error

**Example:**
```bash
curl -X DELETE http://localhost:5500/mqtt/device_001 \
  -H "x-api-key: YOUR_API_KEY"
```

---

### 5. Generate JWT Token

Generates a JWT token for the specified user. The token can be used as the password when connecting to the EMQX MQTT broker.

- **URL:** `/mqtt/jwt`
- **Method:** `POST`
- **Authentication:** Required (`x-api-key`)
- **Request Body:**
  ```json
  {
    "username": "device_001"
  }
  ```
- **Success Response (200):**
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
- **Error Responses:**
  - `400 Bad Request` - Invalid input (empty username)
    ```json
    {
      "success": false,
      "message": "Validation error: username cannot be empty"
    }
    ```
  - `401 Unauthorized` - Missing or invalid API key
  - `404 Not Found` - User not found
    ```json
    {
      "success": false,
      "message": "User not found: device_001"
    }
    ```
  - `500 Internal Server Error` - Server configuration error (invalid SECRET_KEY)

**JWT Token Configuration:**
- Algorithm: HS256 (HMAC-SHA256)
- Issuer: `broker.i-ot.net`
- Audience: `mqtt`
- Expiration: 24 hours
- Secret: Configured via `SECRET_KEY` environment variable

**Example:**
```bash
curl -X POST http://localhost:5500/mqtt/jwt \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{"username":"device_001"}'
```

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

---

## Schemas

### CreateUserRequest

```json
{
  "type": "object",
  "required": ["username", "password"],
  "properties": {
    "username": { "type": "string" },
    "password": { "type": "string" },
    "is_superuser": { "type": "boolean", "default": false }
  }
}
```

### JwtRequest

```json
{
  "type": "object",
  "required": ["username"],
  "properties": {
    "username": { "type": "string" }
  }
}
```

### JwtResponseData

```json
{
  "type": "object",
  "required": ["token", "expires_at"],
  "properties": {
    "token": { "type": "string" },
    "expires_at": { "type": "string", "format": "date-time" }
  }
}
```

### UserDTO

```json
{
  "type": "object",
  "required": ["id", "username", "is_superuser"],
  "properties": {
    "id": { "type": "integer", "format": "int64" },
    "username": { "type": "string" },
    "is_superuser": { "type": "boolean" }
  }
}
```

### UserListDTO

```json
{
  "type": "object",
  "required": ["users", "total", "limit", "offset"],
  "properties": {
    "users": {
      "type": "array",
      "items": { "$ref": "#/definitions/UserDTO" }
    },
    "total": { "type": "integer" },
    "limit": { "type": "integer" },
    "offset": { "type": "integer" }
  }
}
```

### SuccessResponse<T>

```json
{
  "type": "object",
  "required": ["success", "message"],
  "properties": {
    "success": { "type": "boolean" },
    "message": { "type": "string" },
    "data": { "type": ["object", "array", "null"] }
  }
}
```

### ErrorResponse

```json
{
  "type": "object",
  "required": ["success", "message"],
  "properties": {
    "success": { "type": "boolean", "enum": [false] },
    "message": { "type": "string" }
  }
}
```

---

## OpenTelemetry

This service is instrumented with OpenTelemetry for observability.

- **Traces:** Sent to OTLP endpoint (gRPC)
- **Metrics:** HTTP request counts, latency histograms
- **Logs:** Structured JSON logs with trace context

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint | `http://localhost:4317` |
| `OTEL_SERVICE_NAME` | Service name for telemetry | `emqx-auth-service` |

For detailed integration instructions, see [OPENTELEMETRY_GUIDE.md](OPENTELEMETRY_GUIDE.md).

---

## Interactive API Documentation

Access the interactive API documentation:

- **Scalar UI:** http://localhost:5500/openapi
- **OpenAPI JSON:** http://localhost:5500/api-docs/openapi.json

These endpoints provide the most up-to-date API documentation, automatically generated from the code.
