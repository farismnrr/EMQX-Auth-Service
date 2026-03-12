#!/bin/bash
#===============================================================================
# EMQX Auto-Configuration Script
# Runs from HOST, not inside container
#===============================================================================

EMQX_DASHBOARD_URL="${EMQX_DASHBOARD_URL:-http://localhost:18083}"
EMQX_ADMIN_USER="${EMQX_ADMIN_USER:-admin}"
EMQX_ADMIN_PASS="${EMQX_ADMIN_PASS:-public123}"
AUTH_SERVICE_URL="${AUTH_SERVICE_URL:-http://emqx-auth-service:5500}"
API_KEY="${API_KEY:-45514c53469b886e1cd1141a952ecf847061669dd26a06cdbcb08790461bb78018ac460bd1b54fe0a50efa3b7a79c591b85dc0919510b385114710d323fed9f5}"

EMQX_CONTAINER_NAME="${EMQX_CONTAINER_NAME:-emqx-broker}"

log() { echo "[EMQX Setup] $(date '+%Y-%m-%d %H:%M:%S') - $1"; }

log "Starting EMQX auto-configuration..."

# Wait for EMQX
log "Waiting for EMQX to be ready..."
for i in 1 2 3 4 5 6 7 8 9 10; do
    if docker exec "$EMQX_CONTAINER_NAME" emqx_ctl status 2>&1 | grep -q "is started"; then
        log "EMQX is running"
        break
    fi
    sleep 2
done

sleep 3

# Get token
log "Getting dashboard API token..."
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
            \"Authorization\": \"Bearer ${API_KEY}\",
            \"Content-Type\": \"application/json\"
        },
        \"body\": {
            \"username\": \"\${username}\",
            \"password\": \"\${password}\",
            \"method\": \"credentials\"
        }
    }" > /dev/null
log "Authentication configured"

# Configure authorization
log "Configuring HTTP authorization (ACL)..."
curl -s -X POST "${EMQX_DASHBOARD_URL}/api/v5/authorization" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${TOKEN}" \
    -d "{
        \"type\": \"http\",
        \"enable\": true,
        \"method\": \"post\",
        \"url\": \"${AUTH_SERVICE_URL}/mqtt/acl\",
        \"headers\": {
            \"Authorization\": \"Bearer ${API_KEY}\",
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

# Disable dashboard
log "Disabling dashboard..."
curl -s -X PUT "${EMQX_DASHBOARD_URL}/api/v5/configs/dashboard.listeners.http" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${TOKEN}" \
    -d '{"bind": 0}' > /dev/null 2>&1 || true

log "============================================"
log "EMQX auto-configuration completed!"
log "Authentication: ${AUTH_SERVICE_URL}/mqtt/check"
log "Authorization:  ${AUTH_SERVICE_URL}/mqtt/acl"
log "Dashboard:      Disabled"
log "============================================"
