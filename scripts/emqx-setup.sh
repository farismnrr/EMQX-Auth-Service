#!/bin/bash
#===============================================================================
# EMQX Auto-Configuration Script (Tunneling Server Configuration)
# Matches production config: SSL (8883), WSS (8084), Dashboard (18083)
#===============================================================================

# Load .env if it exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

EMQX_DASHBOARD_URL="http://localhost:18083"
EMQX_ADMIN_USER="${EMQX_ADMIN_USER:-admin}"
EMQX_CONTAINER_NAME="${EMQX_CONTAINER_NAME:-dev-emqx}"
# IMPORTANT: Use internal Docker port 5500 for EMQX-to-AuthService communication
AUTH_SERVICE_URL="${AUTH_SERVICE_URL:-http://emqx-auth-service:5500}"
API_KEY="${API_KEY}"

# Certificate paths (matching tunneling server)
SSL_CERT_FILE="${SSL_CERT_FILE:-/etc/emqx/certs/broker.i-ot.net.crt}"
SSL_KEY_FILE="${SSL_KEY_FILE:-/etc/emqx/certs/broker.i-ot.net.key}"

log() { echo -e "[\033[0;34mEMQX Setup\033[0m] $(date '+%Y-%m-%d %H:%M:%S') - $1"; }
warn() { echo -e "[\033[0;33mWARN\033[0m] $1"; }
error() { echo -e "[\033[0;31mERROR\033[0m] $1"; exit 1; }

log "Starting EMQX auto-configuration..."

# 1. Wait for EMQX Node
log "Waiting for EMQX node to be responsive..."
until docker exec "$EMQX_CONTAINER_NAME" emqx_ctl status 2>&1 | grep -q "is started"; do
    sleep 2
done

# 2. Handle Dashboard Lifecycle (Full CLI approach)
log "Ensuring Dashboard API is available for configuration..."
if ! curl -s "$EMQX_DASHBOARD_URL" > /dev/null; then
    log "Dashboard is disabled. Enabling it temporarily on port 18083..."
    # Using temp file inside container to load config string
    docker exec "$EMQX_CONTAINER_NAME" sh -c "echo 'dashboard.listeners.http.bind = 18083' > /tmp/setup_dash.conf"
    docker exec "$EMQX_CONTAINER_NAME" emqx_ctl conf load /tmp/setup_dash.conf > /dev/null
    DASHBOARD_TEMPORARY=true
    sleep 5 # Wait for API to warm up
else
    DASHBOARD_TEMPORARY=false
    log "Dashboard already accessible."
fi

# 3. Authenticate & Get Token
log "Authenticating with Dashboard API..."
# Try credentials from env, then default 'public'
PASSWORDS=("${MQTT_ADMIN_PASSWORD}" "public" "public123")
TOKEN=""

for PASS in "${PASSWORDS[@]}"; do
    [ -z "$PASS" ] && continue
    RESPONSE=$(curl -s -X POST "${EMQX_DASHBOARD_URL}/api/v5/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"${EMQX_ADMIN_USER}\",\"password\":\"${PASS}\"}")

    TOKEN=$(echo "$RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    if [ -n "$TOKEN" ]; then
        log "Authenticated successfully."
        break
    fi
done

if [ -z "$TOKEN" ]; then
    warn "Authentication failed. Attempting to sync admin password from .env..."
    docker exec "$EMQX_CONTAINER_NAME" emqx_ctl admins passwd "$EMQX_ADMIN_USER" "${MQTT_ADMIN_PASSWORD}"
    sleep 5
    RESPONSE=$(curl -s -X POST "${EMQX_DASHBOARD_URL}/api/v5/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"${EMQX_ADMIN_USER}\",\"password\":\"${MQTT_ADMIN_PASSWORD}\"}")
    TOKEN=$(echo "$RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
fi

[ -z "$TOKEN" ] && error "Final authentication attempt failed. Check credentials."

# 4. Configure SSL/TLS Listeners (Matching Tunneling Server)
log "Configuring SSL listener on port 8883..."
curl -s -X PUT "${EMQX_DASHBOARD_URL}/api/v5/listeners/ssl:default/enable/true" -H "Authorization: Bearer ${TOKEN}" > /dev/null
curl -s -X PUT "${EMQX_DASHBOARD_URL}/api/v5/listeners/ssl:default/bind/8883" -H "Authorization: Bearer ${TOKEN}" > /dev/null

log "Configuring WSS listener on port 8084..."
curl -s -X PUT "${EMQX_DASHBOARD_URL}/api/v5/listeners/wss:default/enable/true" -H "Authorization: Bearer ${TOKEN}" > /dev/null
curl -s -X PUT "${EMQX_DASHBOARD_URL}/api/v5/listeners/wss:default/bind/8084" -H "Authorization: Bearer ${TOKEN}" > /dev/null

log "Secure listeners (SSL 8883, WSS 8084) enabled."

# 5. Configure HTTP Authentication
log "Configuring HTTP Authentication..."
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
            \"url\": \"${AUTH_SERVICE_URL}/emqx/auth\",
            \"method\": \"post\",
            \"headers\": {
                \"x-api-key\": \"${API_KEY}\",
                \"Content-Type\": \"application/json\"
            },
            \"body\": {
                \"username\": \"\${username}\",
                \"password\": \"\${password}\"
            }
        }" > /dev/null
    log "Authentication configured ✅"
fi

# 6. Configure HTTP Authorization (ACL)
log "Configuring HTTP Authorization (ACL)..."
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
            \"url\": \"${AUTH_SERVICE_URL}/emqx/acl\",
            \"headers\": {
                \"x-api-key\": \"${API_KEY}\",
                \"Content-Type\": \"application/json\"
            },
            \"body\": {
                \"username\": \"\${username}\",
                \"clientid\": \"\${clientid}\",
                \"topic\": \"\${topic}\",
                \"action\": \"\${action}\"
            }
        }" > /dev/null
    log "Authorization configured ✅"
fi

# 7. Keep Dashboard enabled (matching tunneling server)
log "Dashboard remains enabled on port 18083 (matching tunneling server config)."

log "============================================"
log "EMQX auto-configuration completed!"
log "Listeners Active:"
log "  - 1883 (MQTT)"
log "  - 8083 (WebSocket)"
log "  - 8883 (SSL/TLS)"
log "  - 8084 (WSS)"
log "Dashboard Status: ENABLED on 18083"
log "============================================"
