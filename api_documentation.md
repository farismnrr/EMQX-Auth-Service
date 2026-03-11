# EMQX Auth Service API Documentation

Complete API documentation for the EMQX Auth Service, including HTTP REST API and MQTT-based Admin API.

## Table of Contents

1. [HTTP API](#http-api)
   - [Health Check](#1-health-check)
   - [Create MQTT Client](#2-create-mqtt-client)
   - [Authenticate / Check Client](#3-authenticate--check-client)
   - [Check ACL Permission](#4-check-acl-permission)
   - [Delete MQTT Client](#5-delete-mqtt-client)
   - [List MQTT Clients](#6-list-mqtt-clients)
   - [Get MQTT Client by ID](#7-get-mqtt-client-by-id)
2. [MQTT Admin API](#mqtt-admin-api)
   - [Authentication](#mqtt-authentication)
   - [Create User](#1-create-user)
   - [Delete User](#2-delete-user)
   - [List Users](#3-list-users)
   - [Get User by ID](#4-get-user-by-id)

---

## HTTP API

All HTTP endpoints (except the root health check) require authentication via an API Key.

### HTTP Authentication

**Header:** `Authorization: Bearer <API_KEY>` or `Authorization: <API_KEY>`
**Mode:** Bearer token or direct string.

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
  - `Authorization: Bearer <API_KEY>`
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
- **Error Response (e.g., Username taken):**
  - **Code:** `400 Bad Request` / `409 Conflict` (depending on service logic)
  - **Body:**
    ```json
    {
      "success": false,
      "message": "Error message",
      "details": "Specific details if available"
    }
    ```

---

## 3. Authenticate / Check Client

Verifies client credentials or JWT token.

- **URL:** `/mqtt/check`
- **Method:** `POST`
- **Headers:**
  - `Content-Type: application/json`
  - `Authorization: Bearer <API_KEY>`
- **Request Body:**
  ```json
  {
    "username": "client_id",
    "password": "secure_password",
    "method": "credentials"
  }
  ```
  _Note: `method` can be `"credentials"` or `"jwt"`. If `"jwt"`, password can be empty._
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
  - `Authorization: Bearer <API_KEY>`
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
    _Note: Result will be `"deny"` if access is not granted._

---

## 5. Delete MQTT Client (Soft Delete)

Removes an MQTT client from the system (marks as deleted).

- **URL:** `/mqtt/{username}`
- **Method:** `DELETE`
- **Headers:**
  - `Authorization: Bearer <API_KEY>`
- **URL Params:** `username` (string)
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
  - `Authorization: Bearer <API_KEY>`
- **Query Parameters:**
  - `page` (optional, default: `1`) - Page number
  - `page_size` (optional, default: `10`, max: `100`) - Number of items per page
- **Example:** `GET /mqtt?page=1&page_size=10`
- **Success Response:**
  - **Code:** `200 OK`
  - **Body:**
    ```json
    {
      "success": true,
      "message": "User MQTT list retrieved successfully",
      "data": {
        "users": [
          {
            "id": 1,
            "username": "client_id_1",
            "is_superuser": false,
            "is_deleted": false
          },
          {
            "id": 2,
            "username": "client_id_2",
            "is_superuser": true,
            "is_deleted": false
          }
        ],
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
  - `Authorization: Bearer <API_KEY>`
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
        "is_superuser": false,
        "is_deleted": false
      }
    }
    ```
- **Error Response (Not Found):**
  - **Code:** `404 Not Found`
  - **Body:**
    ```json
    {
      "success": false,
      "message": "User not found"
    }
    ```

---

## MQTT Admin API

The MQTT Admin API allows you to manage MQTT users through MQTT topics instead of HTTP. This is useful for administrative operations performed directly over MQTT.

### MQTT Authentication

To use the MQTT Admin API, your MQTT client must:
1. Be authenticated as a **superuser** in the system
2. Be subscribed to the appropriate response topics to receive operation results

### Multi-Instance Support (Shared Subscriptions)

For production deployments with multiple service instances, the API uses **shared subscriptions** to prevent race conditions:

- **Format**: `$share/{group_id}/{topic}`
- **Behavior**: Only one instance in the group receives each message
- **Load Balancing**: Messages are distributed across instances
- **Configuration**: Set `MQTT_USE_SHARED_SUB=true` (default)

Example topics with shared subscription:
- `$share/emqx_auth_service/admins/users/create`
- `$share/emqx_auth_service/admins/users/delete`
- `$share/emqx_auth_service/admins/users`
- `$share/emqx_auth_service/admins/users/+`

### Message Format

All MQTT messages use JSON format with the following structure:

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

### Request Topics

| Topic | Method | Description |
|-------|--------|-------------|
| `admins/users/create` | PUBLISH | Create a new MQTT user |
| `admins/users/delete` | PUBLISH | Delete an existing MQTT user |
| `admins/users` | PUBLISH | List users with pagination |
| `admins/users/{user_id}` | SUBSCRIBE | Get user by ID (subscribe to topic) |

### Response Topics

| Topic | Description |
|-------|-------------|
| `admins/users/create/response` | Response for create operations |
| `admins/users/delete/response` | Response for delete operations |
| `admins/users/list/response` | Response for list operations |
| `admins/users/detail/response` | Response for get by ID operations |

---

### 1. Create User

Creates a new MQTT user in the system.

- **Request Topic:** `admins/users/create`
- **Response Topic:** `admins/users/create/response`
- **QoS:** `1` (At least once)
- **Request Payload:**
  ```json
  {
    "request_id": "req-123-abc",
    "username": "new_user",
    "password": "secure_password_123",
    "is_superuser": false
  }
  ```
- **Success Response:**
  ```json
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
- **Error Response (Username taken):**
  ```json
  {
    "request_id": "req-123-abc",
    "success": false,
    "message": "Username already exists",
    "data": null
  }
  ```
- **Error Response (Validation error):**
  ```json
  {
    "request_id": "req-123-abc",
    "success": false,
    "message": "Validation failed",
    "data": {
      "errors": ["Username must be at least 3 characters", "Password must be at least 8 characters"]
    }
  }
  ```

---

### 2. Delete User

Deletes (soft delete) an existing MQTT user.

- **Request Topic:** `admins/users/delete`
- **Response Topic:** `admins/users/delete/response`
- **QoS:** `1` (At least once)
- **Request Payload:**
  ```json
  {
    "request_id": "req-456-def",
    "username": "user_to_delete"
  }
  ```
- **Success Response:**
  ```json
  {
    "request_id": "req-456-def",
    "success": true,
    "message": "User deleted successfully",
    "data": null
  }
  ```
- **Error Response (User not found):**
  ```json
  {
    "request_id": "req-456-def",
    "success": false,
    "message": "User not found",
    "data": null
  }
  ```

---

### 3. List Users

Retrieves a paginated list of MQTT users.

- **Request Topic:** `admins/users`
- **Response Topic:** `admins/users/list/response`
- **QoS:** `1` (At least once)
- **Request Payload:**
  ```json
  {
    "request_id": "req-789-ghi",
    "page": 1,
    "page_size": 10
  }
  ```
- **Success Response:**
  ```json
  {
    "request_id": "req-789-ghi",
    "success": true,
    "message": "Users retrieved successfully",
    "data": {
      "users": [
        {
          "id": 1,
          "username": "user_1",
          "is_superuser": false,
          "is_deleted": false
        },
        {
          "id": 2,
          "username": "user_2",
          "is_superuser": true,
          "is_deleted": false
        }
      ],
      "pagination": {
        "total": 50,
        "page": 1,
        "page_size": 10,
        "total_pages": 5
      }
    }
  }
  ```
- **Query Parameters:**
  - `page` (optional, default: `1`) - Page number
  - `page_size` (optional, default: `10`, max: `100`) - Number of items per page

---

### 4. Get User by ID

Retrieves a specific user by their ID.

- **Request Topic:** `admins/users/{user_id}` (e.g., `admins/users/42`)
- **Response Topic:** `admins/users/detail/response`
- **QoS:** `1` (At least once)
- **Request Payload:**
  ```json
  {
    "request_id": "req-000-jkl"
  }
  ```
- **Success Response:**
  ```json
  {
    "request_id": "req-000-jkl",
    "success": true,
    "message": "User retrieved successfully",
    "data": {
      "id": 42,
      "username": "user_42",
      "is_superuser": false,
      "is_deleted": false
    }
  }
  ```
- **Error Response (User not found):**
  ```json
  {
    "request_id": "req-000-jkl",
    "success": false,
    "message": "User not found",
    "data": null
  }
  ```

---

### MQTT Admin API - Error Codes

| Error | Description |
|-------|-------------|
| `Unauthorized` | Client is not a superuser |
| `Invalid Payload` | Request payload is malformed JSON |
| `Validation Error` | Request data fails validation |
| `Not Found` | Requested user does not exist |
| `Conflict` | Resource conflict (e.g., username taken) |
| `Internal Error` | Server-side error |

---

### Example: MQTT Admin Client (JavaScript)

```javascript
const mqtt = require('mqtt');

const client = mqtt.connect('mqtt://localhost:1883', {
  username: 'admin_user',
  password: 'admin_password',
  clientId: 'admin_client_' + Date.now()
});

client.on('connect', () => {
  console.log('Connected to MQTT broker');
  
  // Subscribe to response topics
  client.subscribe('admins/users/+/response');
  client.subscribe('admins/users/list/response');
  client.subscribe('admins/users/detail/response');
  
  // Create a new user
  const createRequest = {
    request_id: 'create-' + Date.now(),
    username: 'new_device_001',
    password: 'SecurePass123!',
    is_superuser: false
  };
  
  client.publish('admins/users/create', JSON.stringify(createRequest), { qos: 1 });
  
  // List users with pagination
  const listRequest = {
    request_id: 'list-' + Date.now(),
    page: 1,
    page_size: 20
  };
  
  client.publish('admins/users', JSON.stringify(listRequest), { qos: 1 });
});

client.on('message', (topic, message) => {
  const response = JSON.parse(message.toString());
  console.log(`Received response on ${topic}:`, response);
});
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
| `MQTT_ADMIN_ENABLED` | Enable MQTT Admin API              | No       | `false`     |
| `MQTT_BROKER_HOST`   | MQTT broker host                   | No       | `localhost` |
| `MQTT_BROKER_PORT`   | MQTT broker port                   | No       | `1883`      |
| `MQTT_USE_TLS`       | Use TLS for MQTT connection        | No       | `false`     |
| `MQTT_ADMIN_USERNAME`| MQTT client username (superuser)   | No       | -           |
| `MQTT_ADMIN_PASSWORD`| MQTT client password               | No       | -           |
| `MQTT_USE_SHARED_SUB`| Enable shared subscriptions        | No       | `true`      |
| `MQTT_PASS_ENCRYPTION_KEY` | AES-256 encryption key (64 hex chars) | Yes | -   |

---
