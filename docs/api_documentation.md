# EMQX Auth Service API Documentation

Complete API documentation for the EMQX Auth Service.

> **Note:** This documentation is based on the actual OpenAPI specification. For the most accurate and up-to-date API docs, visit `/openapi` (Scalar UI) or `/api-docs/openapi.json`.

## Table of Contents

1. [Authentication](#authentication)
2. [Endpoints](#endpoints)
   - [Health Check](#1-health-check)
   - [Create MQTT User](#2-create-mqtt-user)
   - [List MQTT Users](#3-list-mqtt-users)
   - [Get User by ID](#4-get-user-by-id)
   - [Get User by Username](#5-get-user-by-username)
   - [Delete MQTT User](#6-delete-mqtt-user)
   - [EMQX Auth](#7-emqx-auth)
   - [EMQX ACL](#8-emqx-acl)
3. [Schemas](#schemas)

---

## Authentication

All endpoints (except Health Check) require authentication via API Key.

**Header:** `x-api-key: <API_KEY>`

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
    "message": "User mqtt created successfully",
    "data": null
  }
  ```
- **Error Responses:**
  - `400 Bad Request` - Invalid input (empty username/password, invalid JSON)
  - `401 Unauthorized` - Missing or invalid API key
  - `409 Conflict` - Username already exists
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

### 4. Get User by ID

Retrieves a specific MQTT user by their ID.

- **URL:** `/mqtt/{id}`
- **Method:** `GET`
- **Authentication:** Required (`x-api-key`)
- **Path Parameters:**
  - `id` - The user ID (integer)
- **Success Response (200):**
  ```json
  {
    "success": true,
    "message": "User MQTT retrieved successfully",
    "data": {
      "id": 1,
      "username": "device_001",
      "is_superuser": false
    }
  }
  ```
- **Error Responses:**
  - `401 Unauthorized` - Missing or invalid API key
  - `404 Not Found` - User not found
  - `500 Internal Server Error` - Database error

**Example:**
```bash
curl http://localhost:5500/mqtt/1 \
  -H "x-api-key: YOUR_API_KEY"
```

---

### 5. Get User by Username

Retrieves a specific MQTT user by their username.

- **URL:** `/mqtt/users/{username}`
- **Method:** `GET`
- **Authentication:** Required (`x-api-key`)
- **Path Parameters:**
  - `username` - The username (string)
- **Success Response (200):**
  ```json
  {
    "success": true,
    "message": "User MQTT retrieved successfully",
    "data": {
      "id": 1,
      "username": "device_001",
      "is_superuser": false
    }
  }
  ```
- **Error Responses:**
  - `401 Unauthorized` - Missing or invalid API key
  - `404 Not Found` - User not found
  - `500 Internal Server Error` - Database error

**Example:**
```bash
curl http://localhost:5500/mqtt/users/device_001 \
  -H "x-api-key: YOUR_API_KEY"
```

---

### 6. Delete MQTT User

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
    "message": "User mqtt deleted successfully",
    "data": null
  }
  ```
- **Error Responses:**
  - `401 Unauthorized` - Missing or invalid API key
  - `404 Not Found` - User not found
  - `500 Internal Server Error` - Database error

**Example:**
```bash
curl -X DELETE http://localhost:5500/mqtt/device_001 \
  -H "x-api-key: YOUR_API_KEY"
```

---

### 7. EMQX Auth

EMQX HTTP authentication endpoint. Returns result in EMQX native format.

- **URL:** `/emqx/auth`
- **Method:** `POST`
- **Authentication:** None (called by EMQX broker)
- **Request Body:**
  ```json
  {
    "username": "device_001",
    "password": "secure_password"
  }
  ```
- **Success Response (200) - Allow:**
  ```json
  {
    "result": "allow",
    "is_superuser": false
  }
  ```
- **Success Response (200) - Deny:**
  ```json
  {
    "result": "deny"
  }
  ```

**Example:**
```bash
curl -X POST http://localhost:5500/emqx/auth \
  -H "Content-Type: application/json" \
  -d '{"username":"device_001","password":"secure_password"}'
```

---

### 8. EMQX ACL

EMQX HTTP authorization (ACL) endpoint. Returns result in EMQX native format.

- **URL:** `/emqx/acl`
- **Method:** `POST`
- **Authentication:** None (called by EMQX broker)
- **Request Body:**
  ```json
  {
    "username": "device_001",
    "topic": "sensor/data",
    "action": "publish"
  }
  ```
- **Success Response (200) - Allow:**
  ```json
  {
    "result": "allow"
  }
  ```
- **Success Response (200) - Deny:**
  ```json
  {
    "result": "deny"
  }
  ```

**Example:**
```bash
curl -X POST http://localhost:5500/emqx/acl \
  -H "Content-Type: application/json" \
  -d '{"username":"device_001","topic":"sensor/data","action":"publish"}'
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
    "is_superuser": { "type": "boolean" }
  }
}
```

### EmqxAuthRequest

```json
{
  "type": "object",
  "required": ["username", "password"],
  "properties": {
    "username": { "type": "string" },
    "password": { "type": "string" }
  }
}
```

### EmqxAuthResponse

```json
{
  "type": "object",
  "required": ["result"],
  "properties": {
    "result": { "type": "string", "enum": ["allow", "deny"] },
    "is_superuser": { "type": ["boolean", "null"] }
  }
}
```

### EmqxAclRequest

```json
{
  "type": "object",
  "required": ["username", "topic"],
  "properties": {
    "username": { "type": "string" },
    "topic": { "type": "string" },
    "action": { "type": "string", "enum": ["publish", "subscribe"] }
  }
}
```

### EmqxAclResponse

```json
{
  "type": "object",
  "required": ["result"],
  "properties": {
    "result": { "type": "string", "enum": ["allow", "deny"] }
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

### ErrorResponse

```json
{
  "type": "object",
  "required": ["success", "message"],
  "properties": {
    "success": { "type": "boolean" },
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

---

## Interactive API Docs

Access the interactive API documentation:

- **Scalar UI:** http://localhost:5500/openapi
- **OpenAPI JSON:** http://localhost:5500/api-docs/openapi.json
