#!/bin/bash
#===============================================================================
# E2E Test Script for JWT Authentication
# Tests JWT token generation and MQTT connection with EMQX
#===============================================================================

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:5501}"
API_KEY="${API_KEY:-}"
MQTT_BROKER="${MQTT_BROKER:-localhost}"
MQTT_PORT="${MQTT_PORT:-1883}"
TEST_USERNAME="${TEST_USERNAME:-test_jwt_user}"
TEST_PASSWORD="${TEST_PASSWORD:-test_password_123}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_test() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
    ((TESTS_PASSED++))
}

print_failure() {
    echo -e "${RED}✗ $1${NC}"
    ((TESTS_FAILED++))
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    if [ -z "$API_KEY" ]; then
        print_failure "API_KEY environment variable is not set"
        echo "Please set: export API_KEY='your-api-key'"
        exit 1
    fi
    print_success "API_KEY is set"
    
    if ! command -v curl &> /dev/null; then
        print_failure "curl is not installed"
        exit 1
    fi
    print_success "curl is available"
    
    if ! command -v jq &> /dev/null; then
        print_failure "jq is not installed"
        echo "Please install: sudo apt-get install jq"
        exit 1
    fi
    print_success "jq is available"
    
    if command -v mosquitto_sub &> /dev/null; then
        print_success "mosquitto_sub is available"
    else
        print_failure "mosquitto_sub is not available (optional)"
        echo "Install with: sudo apt-get install mosquitto-clients"
    fi
}

# Helper function to make API requests
api_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=$4
    
    local url="${BASE_URL}${endpoint}"
    
    if [ "$method" == "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$url" \
            -H "Content-Type: application/json" \
            -H "x-api-key: $API_KEY" \
            -d "$data")
    else
        response=$(curl -s -w "\n%{http_code}" -X GET "$url" \
            -H "x-api-key: $API_KEY")
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$http_code" == "$expected_status" ]; then
        echo "$body"
        return 0
    else
        echo "Expected status $expected_status, got $http_code"
        echo "Body: $body"
        return 1
    fi
}

#===============================================================================
# Test Suite 1: JWT Endpoint - Success Scenarios
#===============================================================================
test_jwt_success_scenarios() {
    print_header "Test Suite 1: JWT Endpoint - Success Scenarios"
    
    # Test 1.1: Generate JWT for existing user
    print_test "Test 1.1: Generate JWT for existing user ($TEST_USERNAME)"
    
    # First, create the user if not exists
    curl -s -X POST "${BASE_URL}/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$TEST_USERNAME\",\"password\":\"$TEST_PASSWORD\",\"is_superuser\":false}" > /dev/null || true
    
    response=$(api_request "POST" "/mqtt/jwt" "{\"username\":\"$TEST_USERNAME\"}" "200")
    
    if [ $? -eq 0 ]; then
        token=$(echo "$response" | jq -r '.data.token')
        expires_at=$(echo "$response" | jq -r '.data.expires_at')
        
        if [ -n "$token" ] && [ "$token" != "null" ]; then
            print_success "JWT token generated successfully"
            echo "  Token: ${token:0:50}..."
            echo "  Expires: $expires_at"
            export GENERATED_TOKEN="$token"
        else
            print_failure "Token is missing or null"
        fi
    else
        print_failure "Failed to generate JWT token"
    fi
    
    # Test 1.2: Verify JWT structure
    print_test "Test 1.2: Verify JWT token structure"
    
    if [ -n "$GENERATED_TOKEN" ]; then
        # JWT has 3 parts separated by dots
        IFS='.' read -ra PARTS <<< "$GENERATED_TOKEN"
        if [ ${#PARTS[@]} -eq 3 ]; then
            print_success "JWT has correct structure (header.payload.signature)"
            
            # Decode payload
            payload=$(echo "${PARTS[1]}" | base64 -d 2>/dev/null || echo "")
            if [ -n "$payload" ]; then
                iss=$(echo "$payload" | jq -r '.iss' 2>/dev/null)
                aud=$(echo "$payload" | jq -r '.aud' 2>/dev/null)
                sub=$(echo "$payload" | jq -r '.sub' 2>/dev/null)
                
                if [ "$iss" == "broker.i-ot.net" ]; then
                    print_success "  iss claim: $iss ✓"
                else
                    print_failure "  iss claim incorrect: $iss (expected: broker.i-ot.net)"
                fi
                
                if [ "$aud" == "mqtt" ]; then
                    print_success "  aud claim: $aud ✓"
                else
                    print_failure "  aud claim incorrect: $aud (expected: mqtt)"
                fi
                
                if [ "$sub" == "$TEST_USERNAME" ]; then
                    print_success "  sub claim: $sub ✓"
                else
                    print_failure "  sub claim incorrect: $sub (expected: $TEST_USERNAME)"
                fi
            fi
        else
            print_failure "JWT does not have correct structure"
        fi
    else
        print_failure "No token to verify"
    fi
}

#===============================================================================
# Test Suite 2: JWT Endpoint - Error Scenarios
#===============================================================================
test_jwt_error_scenarios() {
    print_header "Test Suite 2: JWT Endpoint - Error Scenarios"
    
    # Test 2.1: User not found
    print_test "Test 2.1: Generate JWT for non-existent user (should return 404)"
    
    response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/mqtt/jwt" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{"username":"nonexistent_user_xyz123"}')
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$http_code" == "404" ]; then
        message=$(echo "$body" | jq -r '.message')
        if [[ "$message" == *"not found"* ]]; then
            print_success "Correctly returns 404 for non-existent user"
            echo "  Message: $message"
        else
            print_failure "Error message doesn't contain 'not found': $message"
        fi
    else
        print_failure "Expected 404, got $http_code"
        echo "  Body: $body"
    fi
    
    # Test 2.2: Empty username
    print_test "Test 2.2: Generate JWT with empty username (should return 400)"
    
    response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/mqtt/jwt" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{"username":""}')
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$http_code" == "400" ]; then
        message=$(echo "$body" | jq -r '.message')
        if [[ "$message" == *"empty"* ]] || [[ "$message" == *"Validation"* ]]; then
            print_success "Correctly returns 400 for empty username"
            echo "  Message: $message"
        else
            print_failure "Error message doesn't mention validation: $message"
        fi
    else
        print_failure "Expected 400, got $http_code"
        echo "  Body: $body"
    fi
    
    # Test 2.3: Missing username field
    print_test "Test 2.3: Generate JWT with missing username field (should return 400)"
    
    response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/mqtt/jwt" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{}')
    
    http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" == "400" ]; then
        print_success "Correctly returns 400 for missing username field"
    else
        print_failure "Expected 400, got $http_code"
    fi
    
    # Test 2.4: Missing API key
    print_test "Test 2.4: Generate JWT without API key (should return 401)"
    
    response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/mqtt/jwt" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$TEST_USERNAME\"}")
    
    http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" == "401" ]; then
        print_success "Correctly returns 401 for missing API key"
    else
        print_failure "Expected 401, got $http_code"
    fi
    
    # Test 2.5: Invalid API key
    print_test "Test 2.5: Generate JWT with invalid API key (should return 401)"
    
    response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/mqtt/jwt" \
        -H "Content-Type: application/json" \
        -H "x-api-key: invalid_api_key_12345" \
        -d "{\"username\":\"$TEST_USERNAME\"}")
    
    http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" == "401" ]; then
        print_success "Correctly returns 401 for invalid API key"
    else
        print_failure "Expected 401, got $http_code"
    fi
    
    # Test 2.6: Username too long
    print_test "Test 2.6: Generate JWT with username > 64 chars (should return 400)"
    
    long_username=$(printf 'a%.0s' {1..65})
    response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/mqtt/jwt" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$long_username\"}")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$http_code" == "400" ]; then
        print_success "Correctly returns 400 for username too long"
    else
        print_failure "Expected 400, got $http_code"
        echo "  Body: $body"
    fi
}

#===============================================================================
# Test Suite 3: MQTT Connection with JWT
#===============================================================================
test_mqtt_connection() {
    print_header "Test Suite 3: MQTT Connection with JWT Token"
    
    if [ -z "$GENERATED_TOKEN" ]; then
        print_failure "No JWT token available for MQTT test"
        print_test "Skipping MQTT connection tests"
        return
    fi
    
    if ! command -v mosquitto_sub &> /dev/null; then
        print_failure "mosquitto_sub not available"
        print_test "Skipping MQTT connection tests"
        return
    fi
    
    # Test 3.1: Connect with valid JWT
    print_test "Test 3.1: MQTT connect with valid JWT token"
    
    # Try to connect and subscribe (timeout after 3 seconds)
    timeout 3 mosquitto_sub -h "$MQTT_BROKER" -p "$MQTT_PORT" \
        -u "$TEST_USERNAME" \
        -P "$GENERATED_TOKEN" \
        -t "test/topic" \
        -v 2>&1 | head -n 5 &
    
    SUB_PID=$!
    sleep 2
    
    if kill -0 $SUB_PID 2>/dev/null; then
        print_success "Successfully connected to MQTT broker with JWT"
        kill $SUB_PID 2>/dev/null || true
    else
        # Check exit code - 124 means timeout (connection succeeded but no messages)
        wait $SUB_PID
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ] || [ $EXIT_CODE -eq 0 ]; then
            print_success "Successfully connected to MQTT broker with JWT"
        else
            print_failure "Failed to connect to MQTT broker with JWT (exit code: $EXIT_CODE)"
        fi
    fi
    
    # Test 3.2: Connect with invalid JWT
    print_test "Test 3.2: MQTT connect with invalid JWT token"
    
    timeout 3 mosquitto_sub -h "$MQTT_BROKER" -p "$MQTT_PORT" \
        -u "$TEST_USERNAME" \
        -P "invalid.jwt.token" \
        -t "test/topic" \
        -v 2>&1 | head -n 5 &
    
    SUB_PID=$!
    sleep 2
    
    if kill -0 $SUB_PID 2>/dev/null; then
        print_failure "Connected with invalid JWT (should have been rejected)"
        kill $SUB_PID 2>/dev/null || true
    else
        wait $SUB_PID
        EXIT_CODE=$?
        if [ $EXIT_CODE -ne 0 ] && [ $EXIT_CODE -ne 124 ]; then
            print_success "Correctly rejected invalid JWT token"
        else
            kill $SUB_PID 2>/dev/null || true
            print_failure "Invalid JWT was accepted (security issue!)"
        fi
    fi
    
    # Test 3.3: Connect with expired JWT (if we can generate one)
    print_test "Test 3.3: MQTT connect with wrong username/password combo"
    
    timeout 3 mosquitto_sub -h "$MQTT_BROKER" -p "$MQTT_PORT" \
        -u "$TEST_USERNAME" \
        -P "wrong_password" \
        -t "test/topic" \
        -v 2>&1 | head -n 5 &
    
    SUB_PID=$!
    sleep 2
    
    if kill -0 $SUB_PID 2>/dev/null; then
        print_failure "Connected with wrong password"
        kill $SUB_PID 2>/dev/null || true
    else
        wait $SUB_PID
        EXIT_CODE=$?
        if [ $EXIT_CODE -ne 0 ] && [ $EXIT_CODE -ne 124 ]; then
            print_success "Correctly rejected wrong password"
        else
            kill $SUB_PID 2>/dev/null || true
            print_failure "Wrong password was accepted"
        fi
    fi
}

#===============================================================================
# Main Test Runner
#===============================================================================
main() {
    print_header "JWT Authentication E2E Test Suite"
    echo "Base URL: $BASE_URL"
    echo "MQTT Broker: $MQTT_BROKER:$MQTT_PORT"
    echo "Test Username: $TEST_USERNAME"
    
    check_prerequisites
    test_jwt_success_scenarios
    test_jwt_error_scenarios
    test_mqtt_connection
    
    print_header "Test Results"
    echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo ""
    
    if [ $TESTS_FAILED -gt 0 ]; then
        echo -e "${RED}Some tests failed!${NC}"
        exit 1
    else
        echo -e "${GREEN}All tests passed!${NC}"
        exit 0
    fi
}

# Run main function
main "$@"
