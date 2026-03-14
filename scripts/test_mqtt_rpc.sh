#!/bin/bash
set -euo pipefail

API_KEY="<REDACTED_API_KEY>"
REQUESTED_BY="iotnet-backend"
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)

echo "=============================================="
echo "MQTT RPC API Test Suite"
echo "=============================================="
echo ""

# Test 1: users.create via MQTT RPC
echo "=== Test 1: MQTT RPC users.create ==="
echo "Request ID: $REQUEST_ID"
echo "Timestamp: $TIMESTAMP"
echo "Requested By: $REQUESTED_BY"

# Start subscriber in background
docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_sub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/replies/$REQUESTED_BY" -v -W 10 > /tmp/mqtt_rpc_response.txt &
SUB_PID=$!
sleep 2

# Publish create command
docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_pub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/commands/users.create" \
  -m "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"timestamp\":$TIMESTAMP,\"username\":\"rpc_user_001\",\"password\":\"rpc_pass_123\",\"is_superuser\":false}"

sleep 8
kill $SUB_PID 2>/dev/null || true

echo "Response:"
cat /tmp/mqtt_rpc_response.txt || echo "(no response)"
echo ""

# Test 2: users.get via MQTT RPC
echo "=== Test 2: MQTT RPC users.get ==="
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)

docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_sub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/replies/$REQUESTED_BY" -v -W 10 > /tmp/mqtt_rpc_response2.txt &
SUB_PID=$!
sleep 2

docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_pub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/commands/users.get" \
  -m "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"timestamp\":$TIMESTAMP,\"username\":\"test_client_001\"}"

sleep 8
kill $SUB_PID 2>/dev/null || true

echo "Response:"
cat /tmp/mqtt_rpc_response2.txt || echo "(no response)"
echo ""

# Test 3: users.delete via MQTT RPC
echo "=== Test 3: MQTT RPC users.delete ==="
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)

docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_sub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/replies/$REQUESTED_BY" -v -W 10 > /tmp/mqtt_rpc_response3.txt &
SUB_PID=$!
sleep 2

docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_pub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/commands/users.delete" \
  -m "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"timestamp\":$TIMESTAMP,\"username\":\"rpc_user_001\"}"

sleep 8
kill $SUB_PID 2>/dev/null || true

echo "Response:"
cat /tmp/mqtt_rpc_response3.txt || echo "(no response)"
echo ""

# Test 4: tokens.issue via MQTT RPC
echo "=== Test 4: MQTT RPC tokens.issue ==="
REQUEST_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date +%s%3N)

docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_sub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/replies/$REQUESTED_BY" -v -W 10 > /tmp/mqtt_rpc_response4.txt &
SUB_PID=$!
sleep 2

docker run --rm --network host eclipse-mosquitto:2.0 mosquitto_pub -h localhost -p 1883 \
  -u "test_client_001" -P "test_password_123" \
  -t "iotnet/auth/commands/tokens.issue" \
  -m "{\"schema_version\":1,\"request_id\":\"$REQUEST_ID\",\"reply_to\":\"iotnet/auth/replies/$REQUESTED_BY\",\"requested_by\":\"$REQUESTED_BY\",\"username\":\"test_client_001\",\"timestamp\":$TIMESTAMP}"

sleep 8
kill $SUB_PID 2>/dev/null || true

echo "Response:"
cat /tmp/mqtt_rpc_response4.txt || echo "(no response)"
echo ""

echo "=============================================="
echo "MQTT RPC Test Suite Complete"
echo "=============================================="
