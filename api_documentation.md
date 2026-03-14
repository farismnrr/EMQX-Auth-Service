# EMQX Auth Service API Documentation

Complete API documentation for the EMQX Auth Service, including HTTP REST API and MQTT RPC API.

## Table of Contents

1. [HTTP API](#http-api)
   - [Health Check](#1-health-check)
   - [Create MQTT Client](#2-create-mqtt-client)
   - [Authenticate / Check Client](#3-authenticate--check-client)
   - [Check ACL Permission](#4-check-acl-permission)
   - [Delete MQTT Client](#5-delete-mqtt-client)
   - [List MQTT Clients](#6-list-mqtt-clients)
   - [Get MQTT Client by ID](#7-get-mqtt-client-by-id)
2. [MQTT RPC API](#mqtt-rpc-api-backend-integration)
   - [Overview](#overview)
   - [Message Format](#message-format)
   - [Security Model](#security-model)
   - [Commands](#commands)

---

## HTTP API

All HTTP endpoints (except the root health check) require authentication via an API Key.

### HTTP Authentication

**Header:** `x-api-key: <API_KEY>`

---

## 1. Health Check

Checks if the service is up and running.

- **URL:** `/`
- **Method:** `GET`
- **Authentication:** None
- **Response:**
  - **Code:** `200 OK`
  - **Body:** `OK` (text/plain)

---

## 2. Create MQTT Client

Registers a new MQTT client in the system.

- **URL:** `/mqtt/create`
- **Method:** `POST`
- **Headers:**
  - `Content-Type: application/json`
  - `x-api-key: <API_KEY>`
- **Request Body:**
  ```json
  {
    "username": "client_id",
    "password": "secure_password",
    "is_superuser": false
  }
  ```
- **Success Response:**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User mqtt created successfully"
    }
    ```

---

## 3. Authenticate / Check Client

Verifies client credentials or JWT token.

- **URL:** `/mqtt/check`
- **Method:** `POST`
- **Headers:**
  - `Content-Type: application/json`
  - `x-api-key: <API_KEY>`
- **Request Body (Credentials):**
  ```json
  {
    "username": "client_id",
    "password": "secure_password",
    "method": "credentials"
  }
  ```
- **Request Body (JWT):**
  ```json
  {
    "username": "client_id",
    "password": "",
    "method": "jwt"
  }
  ```
- **Success Response (Credentials):**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User MQTT is active",
      "result": "allow"
    }
    ```
- **Success Response (JWT Method):**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User MQTT is active",
      "result": "allow",
      "data": {
        "token": "generated_jwt_token"
      }
    }
    ```

---

## 4. Check ACL Permission

Checks if a user has permission to access a specific topic.

- **URL:** `/mqtt/acl`
- **Method:** `POST`
- **Headers:**
  - `Content-Type: application/json`
  - `x-api-key: <API_KEY>`
- **Request Body:**
  ```json
  {
    "username": "client_id",
    "topic": "sensor/data"
  }
  ```
- **Success Response:**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User has access",
      "result": "allow"
    }
    ```

---

## 5. Delete MQTT Client

Removes an MQTT client from the system.

- **URL:** `/mqtt/{username}`
- **Method:** `DELETE`
- **Headers:**
  - `x-api-key: <API_KEY>`
- **Success Response:**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User mqtt deleted successfully"
    }
    ```

---

## 6. List MQTT Clients

Retrieves a list of all registered MQTT clients with pagination support.

- **URL:** `/mqtt`
- **Method:** `GET`
- **Headers:**
  - `x-api-key: <API_KEY>`
- **Query Parameters:**
  - `page` (optional, default: `1`) - Page number
  - `page_size` (optional, default: `10`, max: `100`) - Number of items per page
- **Success Response:**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User MQTT list retrieved successfully",
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

---

## 7. Get MQTT Client by ID

Retrieves a specific MQTT client by their ID.

- **URL:** `/mqtt/{id}`
- **Method:** `GET`
- **Headers:**
  - `x-api-key: <API_KEY>`
- **URL Params:** `id` (integer) - The user ID
- **Success Response:**
  - **Code:** `200 OK`
  - **Body:**
    ```json
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

The MQTT RPC API is designed for backend-to-auth-service communication. It uses a request-reply pattern with per-requester reply topics.

### Overview

- **Requester**: Backend service (e.g., IoTNet backend)
- **Responder**: EMQX Auth Service
- **Pattern**: Request-Reply over MQTT
- **QoS**: `1` (At least once)
- **Retain**: `false` for all RPC responses
- **Correlation**: Via `request_id` field

### Message Format

#### Request Envelope

All RPC requests follow this envelope structure:

```json
{
  "schema_version": 1,
  "request_id": "uuid-here",
  "reply_to": "iotnet/auth/replies/backend-instance-1",
  "requested_by": "iotnet-backend",
  "timestamp": 1234567890,
  // Command-specific data fields follow
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | number | Yes | Protocol version (currently `1`) |
| `request_id` | string | Yes | Unique correlation ID (UUID recommended) |
| `reply_to` | string | Yes | Topic where response should be published |
| `requested_by` | string | Yes | Identifier of the requesting service |
| `timestamp` | number | Yes | Unix timestamp in milliseconds |
| `target_user` | string | For tokens.issue | Target user for the operation (must match `requested_by` for token issuance) |

#### Response Envelope

All RPC responses follow this structure:

```json
{
  "schema_version": 1,
  "request_id": "uuid-here",
  "success": true,
  "message": "Operation completed successfully",
  "code": null,
  "data": { /* operation-specific data */ }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | number | Yes | Protocol version (currently `1`) |
| `request_id` | string | Yes | Echo of the request's `request_id` |
| `success` | boolean | Yes | `true` for success, `false` for error |
| `message` | string | Yes | Human-readable description |
| `code` | string | No | Error code (only on failure) |
| `data` | object | No | Operation-specific response data |

### Security Model

1. **Allowed Requesters**: Only requesters in `MQTT_ADMIN_ALLOWED_REQUESTERS` environment variable can issue commands
2. **Reply Topic Binding**: `reply_to` must exactly match `iotnet/auth/replies/{requested_by}` - prevents routing attacks
3. **Token Issuance Protection**: For `tokens.issue`, `target_user` must match `requested_by` - prevents minting tokens for other users
4. **Timestamp Validation**: Requests older than 5 minutes are rejected - prevents replay attacks
5. **Broker ACLs**: Primary authorization is enforced via MQTT broker ACLs on `iotnet/auth/commands/*` topics

### Error Codes

| Code | HTTP Equivalent | Description |
|------|----------------|-------------|
| `USER_ALREADY_EXISTS` | 409 Conflict | Username already exists |
| `USER_NOT_FOUND` | 404 Not Found | User does not exist |
| `VALIDATION_ERROR` | 400 Bad Request | Request validation failed |
| `JWT_ISSUE_FAILED` | 500 Internal Error | Failed to generate JWT token |
| `INTERNAL_ERROR` | 500 Internal Error | Generic internal server error |
| `UNAUTHORIZED_COMMAND` | 401 Unauthorized | Requester not authorized or target_user mismatch |

### Commands

---

### 1. `users.create`

Creates a new MQTT user.

- **Request Topic:** `iotnet/auth/commands/users.create`
- **Reply Topic:** `iotnet/auth/replies/{requested_by}`
- **QoS:** `1`
- **Retain:** `false`

#### Request Payload

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "reply_to": "iotnet/auth/replies/backend-prod-1",
  "requested_by": "iotnet-backend",
  "timestamp": 1234567890,
  "username": "device_001",
  "password": "secure_password_123",
  "is_superuser": false
}
```

#### Success Response

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true,
  "message": "User created successfully",
  "code": null,
  "data": null
}
```

---

### 2. `users.delete`

Deletes an existing MQTT user.

- **Request Topic:** `iotnet/auth/commands/users.delete`
- **Reply Topic:** `iotnet/auth/replies/{requested_by}`
- **QoS:** `1`
- **Retain:** `false`

#### Request Payload

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440001",
  "reply_to": "iotnet/auth/replies/backend-prod-1",
  "requested_by": "iotnet-backend",
  "timestamp": 1234567890,
  "username": "device_001"
}
```

#### Success Response

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440001",
  "success": true,
  "message": "User deleted successfully",
  "code": null,
  "data": null
}
```

---

### 3. `users.get`

Retrieves user metadata by username.

- **Request Topic:** `iotnet/auth/commands/users.get`
- **Reply Topic:** `iotnet/auth/replies/{requested_by}`
- **QoS:** `1`
- **Retain:** `false`

#### Request Payload

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440002",
  "reply_to": "iotnet/auth/replies/backend-prod-1",
  "requested_by": "iotnet-backend",
  "timestamp": 1234567890,
  "username": "device_001"
}
```

#### Success Response

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440002",
  "success": true,
  "message": "User retrieved successfully",
  "code": null,
  "data": {
    "id": 42,
    "username": "device_001",
    "is_superuser": false
  }
}
```

---

### 4. `tokens.issue`

Issues a JWT token for an existing user.

- **Request Topic:** `iotnet/auth/commands/tokens.issue`
- **Reply Topic:** `iotnet/auth/replies/{requested_by}`
- **QoS:** `1`
- **Retain:** `false`

#### Request Payload

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440003",
  "reply_to": "iotnet/auth/replies/backend-prod-1",
  "requested_by": "iotnet-backend",
  "target_user": "iotnet-backend",
  "timestamp": 1234567890,
  "username": "device_001"
}
```

**Note:** `target_user` must match `requested_by` for security. This prevents a backend from minting tokens for arbitrary users.

#### Success Response

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440003",
  "success": true,
  "message": "Token issued successfully",
  "code": null,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

---

### 5. `tokens.verify`

Verifies a user's password.

- **Request Topic:** `iotnet/auth/commands/tokens.verify`
- **Reply Topic:** `iotnet/auth/replies/{requested_by}`
- **QoS:** `1`
- **Retain:** `false`

#### Request Payload

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440004",
  "reply_to": "iotnet/auth/replies/backend-prod-1",
  "requested_by": "iotnet-backend",
  "timestamp": 1234567890,
  "username": "device_001",
  "password": "secure_password_123"
}
```

#### Success Response (Valid Password)

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440004",
  "success": true,
  "message": "Password verified successfully",
  "code": null,
  "data": {
    "valid": true
  }
}
```

#### Success Response (Invalid Password)

```json
{
  "schema_version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440004",
  "success": true,
  "message": "Invalid password",
  "code": null,
  "data": {
    "valid": false
  }
}
```

---

### Example: Backend MQTT RPC Client (TypeScript)

```typescript
import * as mqtt from 'mqtt';
import { v4 as uuidv4 } from 'uuid';

const client = mqtt.connect('mqtts://broker.example.com:8883', {
  clientId: 'backend-prod-1',
  username: 'backend_user',
  password: 'backend_password'
});

const REPLY_TOPIC = 'iotnet/auth/replies/backend-prod-1';
const COMMAND_PREFIX = 'iotnet/auth/commands';

// Pending requests tracking
const pendingRequests = new Map<string, {
  resolve: (value: any) => void;
  reject: (reason: Error) => void;
  timeoutId: NodeJS.Timeout;
}>();

client.on('connect', () => {
  console.log('Connected to MQTT broker');

  // Subscribe to reply topic
  client.subscribe(REPLY_TOPIC, { qos: 1 }, (err) => {
    if (err) {
      console.error('Failed to subscribe to reply topic:', err);
    } else {
      console.log('Subscribed to reply topic:', REPLY_TOPIC);
    }
  });
});

client.on('message', (topic, message) => {
  if (topic !== REPLY_TOPIC) return;

  try {
    const response = JSON.parse(message.toString());
    const pending = pendingRequests.get(response.request_id);

    if (!pending) {
      console.warn('Received response for unknown request:', response.request_id);
      return;
    }

    clearTimeout(pending.timeoutId);
    pendingRequests.delete(response.request_id);

    if (response.success) {
      pending.resolve(response);
    } else {
      pending.reject(new Error(`${response.code}: ${response.message}`));
    }
  } catch (error) {
    console.error('Failed to parse response:', error);
  }
});

async function sendRpcCommand<T>(command: string, data: any): Promise<T> {
  return new Promise((resolve, reject) => {
    const requestId = uuidv4();
    const topic = `${COMMAND_PREFIX}/${command}`;

    const envelope = {
      schema_version: 1,
      request_id: requestId,
      reply_to: REPLY_TOPIC,
      requested_by: 'iotnet-backend',
      timestamp: Date.now(),
      ...data
    };

    // Add target_user for token operations
    if (command === 'tokens.issue' || command === 'tokens.verify') {
      envelope.target_user = data.username;
    }

    const timeoutId = setTimeout(() => {
      pendingRequests.delete(requestId);
      reject(new Error('Request timeout'));
    }, 5000);

    pendingRequests.set(requestId, { resolve, reject, timeoutId });

    client.publish(topic, JSON.stringify(envelope), { qos: 1 }, (err) => {
      if (err) {
        clearTimeout(timeoutId);
        pendingRequests.delete(requestId);
        reject(err);
      }
    });
  });
}

// Usage example
async function createDeviceUser(deviceId: string, password: string) {
  const response = await sendRpcCommand('users.create', {
    username: deviceId,
    password: password,
    is_superuser: false
  });

  console.log('User created:', response);
}

async function getDeviceToken(deviceId: string) {
  const response = await sendRpcCommand('tokens.issue', {
    username: deviceId,
    target_user: 'iotnet-backend'  // Must match requested_by
  });

  console.log('Token received:', response.data.token);
  return response.data.token;
}
```

---

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
| `MQTT_ADMIN_ENABLED` | Enable MQTT RPC API                | No       | `false`     |
| `MQTT_BROKER_HOST`   | MQTT broker host                   | No       | `localhost` |
| `MQTT_BROKER_PORT`   | MQTT broker port                   | No       | `1883`      |
| `MQTT_USE_TLS`       | Use TLS for MQTT connection        | No       | `false`     |
| `MQTT_ADMIN_USERNAME`| MQTT client username (superuser)   | No       | -           |
| `MQTT_ADMIN_PASSWORD`| MQTT client password               | No       | -           |
| `MQTT_ADMIN_ALLOWED_REQUESTERS` | Comma-separated list of allowed requesters | No | `iotnet-backend` |
| `MQTT_PASS_ENCRYPTION_KEY` | AES-256 encryption key (64 hex chars) | Yes | -   |

---
