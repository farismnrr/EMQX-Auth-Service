#!/bin/bash
set -euo pipefail

# Configuration
API_KEY="${API_KEY:-}"
if [ -z "$API_KEY" ]; then
  echo "Error: API_KEY environment variable is not set"
  exit 1
fi

REQUESTED_BY="iotnet-backend"

echo "=============================================="
echo "MQTT RPC API Test Suite"
echo "=============================================="
echo ""

run_test() {
  local test_name="$1"
  local topic="$2"
  local payload="$3"
  local reply_to="${4:-iotnet/auth/replies/$REQUESTED_BY}"

  echo "=== $test_name ==="
  echo "Topic: $topic"
  
  # Start subscriber in background (QoS 2)
  docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_sub -h localhost -p 1883 \
    -u "test_client_001" -P "test_password_123" \
    -t "$reply_to" -q 2 -v -W 10 > "/tmp/${test_name// /_}_response.txt" &
  SUB_PID=$!
  sleep 2

  # Publish command (QoS 2)
  docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_pub -h localhost -p 1883 \
    -u "test_client_001" -P "test_password_123" \
    -t "$topic" -q 2 \
    -m "$payload"

  sleep 8
  kill $SUB_PID 2>/dev/null || true

  echo "Response:"
  cat "/tmp/${test_name// /_}_response.txt" || echo "(no response)"
  echo ""
}

# Standard test variables
TIMESTAMP=$(date +%s%3N)
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)

# Test 1: users.create
run_test "Test 1: MQTT RPC users.create" \
  "iotnet/auth/commands/users.create" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"rpc_user_001\",\"password\":\"rpc_pass_123\",\"is_superuser\":false}"

# Test 2: users.get
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 2: MQTT RPC users.get" \
  "iotnet/auth/commands/users.get" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}"

# Test 3: users.delete
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 3: MQTT RPC users.delete" \
  "iotnet/auth/commands/users.delete" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"rpc_user_001\"}"

# Test 4: tokens.issue
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 4: MQTT RPC tokens.issue" \
  "iotnet/auth/commands/tokens.issue" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}"

# Test 5: users.list
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 5: MQTT RPC users.list" \
  "iotnet/auth/commands/users.list" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"limit\":10,\"offset\":0}"

# Test 6: tokens.verify (Valid)
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 6: MQTT RPC tokens.verify (Valid)" \
  "iotnet/auth/commands/tokens.verify" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\",\"password\":\"test_password_123\"}"

# Test 7a: Negative - Invalid API Key
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 7a: Negative - Invalid API Key" \
  "iotnet/auth/commands/users.get" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"INVALID_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}"

# Test 7b: Negative - Expired Timestamp
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=1000 # very old timestamp
run_test "Test 7b: Negative - Expired Timestamp" \
  "iotnet/auth/commands/users.get" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}"

# Test 7c: Negative - Invalid Reply Topic
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 7c: Negative - Invalid Reply Topic" \
  "iotnet/auth/commands/users.get" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/wrong-backend\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}" \
  "iotnet/auth/replies/wrong-backend"

# Test 7d: Negative - Invalid Credentials (verify)
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 7d: Negative - Invalid Credentials" \
  "iotnet/auth/commands/tokens.verify" \
  "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\",\"password\":\"wrong_password_123\"}"

# Test 7e: Negative - Invalid Schema Version
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)
run_test "Test 7e: Negative - Invalid Schema Version" \
  "iotnet/auth/commands/users.get" \
  "{\"schema_version\":2,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"api_key\":\"$API_KEY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}"

echo "=============================================="
echo "MQTT RPC Test Suite Complete"
echo "=============================================="