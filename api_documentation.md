# EMQX Auth Service API Documentation

All endpoints (except the root health check) require authentication via an API Key.

## Authentication

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

## 6. List MQTT Clients (Dev Only)

Retrieves a list of all registered MQTT clients.

- **URL:** `/mqtt`
- **Method:** `GET`
- **Headers:**
  - `Authorization: Bearer <API_KEY>`
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
            "username": "client_id",
            "password": "hashed_password",
            "is_superuser": false,
            "is_deleted": false
          }
        ]
      }
    }
    ```
