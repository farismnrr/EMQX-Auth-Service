#!/bin/bash
#===============================================================================
# EMQX Auto-Configuration Script
# Runs from HOST, not inside container
#===============================================================================

EMQX_DASHBOARD_URL="${EMQX_DASHBOARD_URL:-http://localhost:18083}"
EMQX_ADMIN_USER="${EMQX_ADMIN_USER:-admin}"
EMQX_ADMIN_PASS="${EMQX_ADMIN_PASS:-public123}"
AUTH_SERVICE_URL="${AUTH_SERVICE_URL:-http://localhost:5500}"
API_KEY="${API_KEY}"

if [ -z "$API_KEY" ]; then
    echo "[EMQX Setup] ERROR: API_KEY is not set. Please check your .env file."
    exit 1
fi

EMQX_CONTAINER_NAME="${EMQX_CONTAINER_NAME:-dev-emqx}"

log() { echo "[EMQX Setup] $(date '+%Y-%m-%d %H:%M:%S') - $1"; }

log "Starting EMQX auto-configuration..."

# Wait for EMQX
log "Waiting for EMQX to be ready..."
for i in {1..15}; do
    if docker exec "$EMQX_CONTAINER_NAME" emqx_ctl status 2>&1 | grep -q "is started"; then
        log "EMQX is running"
        break
    fi
    if [ $i -eq 15 ]; then
        log "ERROR: EMQX failed to start in time"
        exit 1
    fi
    sleep 3
done

sleep 2

# Get token
log "Getting dashboard API token..."
# Ensure admin user exists or update password
docker exec "$EMQX_CONTAINER_NAME" emqx_ctl admins passwd "$EMQX_ADMIN_USER" "$EMQX_ADMIN_PASS" 2>/dev/null || true

TOKEN=$(curl -s -X POST "${EMQX_DASHBOARD_URL}/api/v5/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"${EMQX_ADMIN_USER}\",\"password\":\"${EMQX_ADMIN_PASS}\"}" \
    | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    log "ERROR: Failed to get dashboard token"
    exit 1
fi
log "Dashboard token obtained"

# Configure authentication
log "Configuring HTTP authentication..."
# First check if it already exists
AUTH_LIST=$(curl -s -X GET "${EMQX_DASHBOARD_URL}/api/v5/authentication" -H "Authorization: Bearer ${TOKEN}")
if echo "$AUTH_LIST" | grep -q "password_based:http"; then
    log "Authentication already configured, skipping..."
else
    curl -s -X POST "${EMQX_DASHBOARD_URL}/api/v5/authentication" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer ${TOKEN}" \
        -d "{
            \"mechanism\": \"password_based\",
            \"backend\": \"http\",
            \"enable\": true,
            \"url\": \"${AUTH_SERVICE_URL}/mqtt/check\",
            \"method\": \"post\",
            \"headers\": {
                \"Authorization\": \"${API_KEY}\",
                \"Content-Type\": \"application/json\"
            },
            \"body\": {
                \"username\": \"\${username}\",
                \"password\": \"\${password}\",
                \"method\": \"credentials\"
            }
        }" > /dev/null
    log "Authentication configured"
fi

# Configure authorization
log "Configuring HTTP authorization (ACL)..."
# First check if it already exists
AUTHZ_LIST=$(curl -s -X GET "${EMQX_DASHBOARD_URL}/api/v5/authorization/sources" -H "Authorization: Bearer ${TOKEN}")
if echo "$AUTHZ_LIST" | grep -q "http"; then
    log "Authorization already configured, skipping..."
else
    curl -s -X POST "${EMQX_DASHBOARD_URL}/api/v5/authorization/sources" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer ${TOKEN}" \
        -d "{
            \"type\": \"http\",
            \"enable\": true,
            \"method\": \"post\",
            \"url\": \"${AUTH_SERVICE_URL}/mqtt/acl\",
            \"headers\": {
                \"Authorization\": \"${API_KEY}\",
                \"Content-Type\": \"application/json\"
            },
            \"body\": {
                \"username\": \"\${username}\",
                \"clientid\": \"\${clientid}\",
                \"topic\": \"\${topic}\",
                \"action\": \"\${action}\"
            }
        }" > /dev/null
    log "Authorization configured"
fi

# Disable dashboard (optional, keep enabled for dev if needed, but here we follow original script)
# log "Disabling dashboard..."
# curl -s -X PUT "${EMQX_DASHBOARD_URL}/api/v5/configs/dashboard.listeners.http" \
#     -H "Content-Type: application/json" \
#     -H "Authorization: Bearer ${TOKEN}" \
#     -d '{"bind": 0}' > /dev/null 2>&1 || true

log "============================================"
log "EMQX auto-configuration completed!"
log "Authentication: ${AUTH_SERVICE_URL}/mqtt/check"
log "Authorization:  ${AUTH_SERVICE_URL}/mqtt/acl"
log "Dashboard:      ${EMQX_DASHBOARD_URL}"
log "============================================"
