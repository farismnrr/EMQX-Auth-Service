#!/bin/bash
# =============================================================================
# EMQX Auth Service - Comprehensive E2E Test Script
# =============================================================================
# Tests all REST API endpoints based on actual OpenAPI specification
# =============================================================================

# Configuration
BASE_URL="${AUTH_SERVICE_URL:-http://localhost:5500}"
API_KEY="${API_KEY}"
TEST_PREFIX="e2e_test_$(date +%s)"

if [ -z "$API_KEY" ]; then
    echo -e "${RED}[ERROR]${NC} API_KEY environment variable is not set"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# =============================================================================
# Helper Functions
# =============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

# Test HTTP status code
test_status() {
    local expected=$1
    local actual=$2
    local test_name=$3

    if [ "$expected" == "$actual" ]; then
        log_success "$test_name (Status: $actual)"
        return 0
    else
        log_error "$test_name (Expected: $expected, Got: $actual)"
        return 1
    fi
}

# Test JSON response field
test_json_field() {
    local json=$1
    local field=$2
    local expected=$3
    local test_name=$4

    local actual=$(echo "$json" | jq -r "$field" 2>/dev/null)

    if [ "$expected" == "$actual" ]; then
        log_success "$test_name"
        return 0
    else
        log_error "$test_name (Expected: $expected, Got: $actual)"
        return 1
    fi
}

# =============================================================================
# Test Functions
# =============================================================================

test_health_check() {
    log_section "1. Health Check Endpoint"

    # Test 1: GET / (no auth required)
    log_info "Testing GET / (no authentication)"
    local response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "Health check returns 200"
    if [ "$body" == "OK" ]; then
        log_success "Health check returns 'OK'"
    else
        log_error "Health check body (Expected: OK, Got: $body)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi
}

test_create_user() {
    log_section "2. Create User Endpoint"

    local test_username="${TEST_PREFIX}_user1"
    local test_password="test_password_123"

    # Test 1: Create user successfully
    log_info "Testing POST /mqtt/create (valid data)"
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$test_username\",\"password\":\"$test_password\",\"is_superuser\":false}")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "Create user returns 200"
    test_json_field "$body" ".success" "true" "Create user success field"

    # Test 2: Create duplicate user (should fail with 409 or 500)
    log_info "Testing POST /mqtt/create (duplicate user)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$test_username\",\"password\":\"$test_password\",\"is_superuser\":false}")
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)

    # Accept either 409 (Conflict) or success=false
    if [ "$status" == "409" ]; then
        log_success "Duplicate user returns 409 Conflict"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    elif [ "$status" == "200" ]; then
        test_json_field "$body" ".success" "false" "Duplicate user returns success=false"
    else
        log_error "Duplicate user (Expected: 409 or success=false, Got: $status)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi

    # Test 3: Create user without API key (should fail)
    log_info "Testing POST /mqtt/create (no API key)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"${TEST_PREFIX}_user2\",\"password\":\"test\",\"is_superuser\":false}")
    status=$(echo "$response" | tail -n 1)

    # Accept 400 or 401 for missing auth
    if [ "$status" == "400" ] || [ "$status" == "401" ]; then
        log_success "No API key returns $status"
    else
        log_error "No API key (Expected: 400 or 401, Got: $status)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi

    # Test 4: Create user with invalid API key (should fail)
    log_info "Testing POST /mqtt/create (invalid API key)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: invalid_key" \
        -d "{\"username\":\"${TEST_PREFIX}_user3\",\"password\":\"test\",\"is_superuser\":false}")
    status=$(echo "$response" | tail -n 1)

    if [ "$status" == "400" ] || [ "$status" == "401" ]; then
        log_success "Invalid API key returns $status"
    else
        log_error "Invalid API key (Expected: 400 or 401, Got: $status)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi

    # Test 5: Create user with empty username (should fail)
    log_info "Testing POST /mqtt/create (empty username)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"\",\"password\":\"test\",\"is_superuser\":false}")
    status=$(echo "$response" | tail -n 1)

    test_status "400" "$status" "Empty username returns 400"

    # Test 6: Create user with empty password (should fail)
    log_info "Testing POST /mqtt/create (empty password)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"${TEST_PREFIX}_user4\",\"password\":\"\",\"is_superuser\":false}")
    status=$(echo "$response" | tail -n 1)

    test_status "400" "$status" "Empty password returns 400"

    # Test 7: Create user with invalid JSON (should fail)
    log_info "Testing POST /mqtt/create (invalid JSON)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "invalid json")
    status=$(echo "$response" | tail -n 1)

    test_status "400" "$status" "Invalid JSON returns 400"
}

test_list_users() {
    log_section "3. List Users Endpoint"

    # Test 1: List all users
    log_info "Testing GET /mqtt (list all users)"
    local response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt" \
        -H "x-api-key: $API_KEY")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "List users returns 200"
    test_json_field "$body" ".success" "true" "List users success field"

    # Test 2: List users with pagination
    log_info "Testing GET /mqtt?limit=10&offset=0"
    response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt?limit=10&offset=0" \
        -H "x-api-key: $API_KEY")
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "List users with pagination returns 200"

    # Test 3: List users without API key
    log_info "Testing GET /mqtt (no API key)"
    response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt")
    status=$(echo "$response" | tail -n 1)

    if [ "$status" == "400" ] || [ "$status" == "401" ]; then
        log_success "List users without API key returns $status"
    else
        log_error "List users without API key (Expected: 400 or 401, Got: $status)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi
}

test_get_user_by_id() {
    log_section "4. Get User By ID Endpoint"

    # Get first user ID from list
    log_info "Getting first user ID"
    local response=$(curl -s -X GET "$BASE_URL/mqtt" -H "x-api-key: $API_KEY")
    local user_id=$(echo "$response" | jq -r '.data.users[0].id' 2>/dev/null)

    if [ "$user_id" != "null" ] && [ -n "$user_id" ] && [ "$user_id" != "" ]; then
        # Test 1: Get user by valid ID
        log_info "Testing GET /mqtt/$user_id (valid ID)"
        response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt/$user_id" \
            -H "x-api-key: $API_KEY")
        local body=$(echo "$response" | head -n -1)
        local status=$(echo "$response" | tail -n 1)

        test_status "200" "$status" "Get user by ID returns 200"
        test_json_field "$body" ".success" "true" "Get user by ID success field"

        # Test 2: Get user by invalid ID
        log_info "Testing GET /mqtt/99999 (invalid ID)"
        response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt/99999" \
            -H "x-api-key: $API_KEY")
        status=$(echo "$response" | tail -n 1)

        test_status "404" "$status" "Get user by invalid ID returns 404"
    else
        log_warn "No users found to test get by ID"
    fi

    # Test 3: Get user without API key
    log_info "Testing GET /mqtt/1 (no API key)"
    response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt/1")
    status=$(echo "$response" | tail -n 1)

    if [ "$status" == "400" ] || [ "$status" == "401" ]; then
        log_success "Get user without API key returns $status"
    else
        log_error "Get user without API key (Expected: 400 or 401, Got: $status)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi
}

test_get_user_by_username() {
    log_section "5. Get User By Username Endpoint"

    local test_username="${TEST_PREFIX}_get_user"
    local test_password="get_user_password"

    # Create user first
    log_info "Creating test user for get by username"
    curl -s -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$test_username\",\"password\":\"$test_password\",\"is_superuser\":false}" > /dev/null

    # Test 1: Get user by valid username
    log_info "Testing GET /mqtt/users/$test_username (valid username)"
    local response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt/users/$test_username" \
        -H "x-api-key: $API_KEY")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "Get user by username returns 200"
    test_json_field "$body" ".success" "true" "Get user by username success field"

    # Test 2: Get user by non-existent username
    log_info "Testing GET /mqtt/users/nonexistent (non-existent)"
    response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/mqtt/users/nonexistent" \
        -H "x-api-key: $API_KEY")
    status=$(echo "$response" | tail -n 1)

    test_status "404" "$status" "Get user by non-existent username returns 404"
}

test_delete_user() {
    log_section "6. Delete User Endpoint"

    local test_username="${TEST_PREFIX}_delete_user"
    local test_password="delete_password"

    # Create user first
    log_info "Creating test user for delete"
    curl -s -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$test_username\",\"password\":\"$test_password\",\"is_superuser\":false}" > /dev/null

    # Test 1: Delete user successfully
    log_info "Testing DELETE /mqtt/$test_username (valid user)"
    local response=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/mqtt/$test_username" \
        -H "x-api-key: $API_KEY")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "Delete user returns 200"
    test_json_field "$body" ".success" "true" "Delete user success field"

    # Test 2: Delete already deleted user
    log_info "Testing DELETE /mqtt/$test_username (already deleted)"
    response=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/mqtt/$test_username" \
        -H "x-api-key: $API_KEY")
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)

    test_status "404" "$status" "Delete non-existent user returns 404"

    # Test 3: Delete user without API key
    log_info "Testing DELETE /mqtt/someuser (no API key)"
    response=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/mqtt/someuser")
    status=$(echo "$response" | tail -n 1)

    if [ "$status" == "400" ] || [ "$status" == "401" ]; then
        log_success "Delete user without API key returns $status"
    else
        log_error "Delete user without API key (Expected: 400 or 401, Got: $status)"
        TESTS_TOTAL=$((TESTS_TOTAL + 1))
    fi
}

test_emqx_endpoints() {
    log_section "7. EMQX Native Endpoints"

    local test_username="${TEST_PREFIX}_emqx_user"
    local test_password="emqx_password"

    # Create user first
    log_info "Creating test user for EMQX endpoints"
    curl -s -X POST "$BASE_URL/mqtt/create" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "{\"username\":\"$test_username\",\"password\":\"$test_password\",\"is_superuser\":false}" > /dev/null

    # Test 1: EMQX auth endpoint with valid credentials
    log_info "Testing POST /emqx/auth (valid credentials)"
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/emqx/auth" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_username\",\"password\":\"$test_password\"}")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "EMQX auth returns 200"
    test_json_field "$body" ".result" "allow" "EMQX auth returns allow for valid credentials"

    # Test 2: EMQX auth endpoint with invalid credentials
    log_info "Testing POST /emqx/auth (invalid credentials)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/emqx/auth" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_username\",\"password\":\"wrong_password\"}")
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "EMQX auth invalid credentials returns 200"
    test_json_field "$body" ".result" "deny" "EMQX auth returns deny for invalid credentials"

    # Test 3: EMQX auth with non-existent user
    log_info "Testing POST /emqx/auth (non-existent user)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/emqx/auth" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"nonexistent_user\",\"password\":\"test\"}")
    body=$(echo "$response" | head -n -1)

    test_json_field "$body" ".result" "deny" "EMQX auth returns deny for non-existent user"

    # Test 4: EMQX ACL endpoint
    log_info "Testing POST /emqx/acl (valid user)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/emqx/acl" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_username\",\"topic\":\"sensor/data\",\"action\":\"publish\"}")
    status=$(echo "$response" | tail -n 1)

    test_status "200" "$status" "EMQX ACL returns 200"

    # Test 5: EMQX ACL with missing username
    log_info "Testing POST /emqx/acl (missing username)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/emqx/acl" \
        -H "Content-Type: application/json" \
        -d "{\"topic\":\"sensor/data\",\"action\":\"publish\"}")
    status=$(echo "$response" | tail -n 1)

    test_status "400" "$status" "EMQX ACL missing username returns 400"

    # Test 6: EMQX ACL with missing topic
    log_info "Testing POST /emqx/acl (missing topic)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/emqx/acl" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_username\",\"action\":\"publish\"}")
    status=$(echo "$response" | tail -n 1)

    test_status "400" "$status" "EMQX ACL missing topic returns 400"
}

cleanup() {
    log_section "Cleanup Test Users"
    log_info "Cleaning up test users..."

    # Delete all test users created during this test run
    local response=$(curl -s -X GET "$BASE_URL/mqtt" -H "x-api-key: $API_KEY")
    local users=$(echo "$response" | jq -r ".data.users[] | select(.username | startswith(\"$TEST_PREFIX\")) | .username" 2>/dev/null)

    for username in $users; do
        log_info "Deleting test user: $username"
        curl -s -X DELETE "$BASE_URL/mqtt/$username" -H "x-api-key: $API_KEY" > /dev/null
    done

    log_success "Cleanup completed"
}

print_summary() {
    log_section "Test Summary"
    echo -e "Total Tests: $TESTS_TOTAL"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        return 1
    fi
}

# =============================================================================
# Main Execution
# =============================================================================

main() {
    log_section "EMQX Auth Service E2E Test Suite"
    log_info "Base URL: $BASE_URL"
    log_info "API Key: ${API_KEY:0:10}..."
    log_info "Test Prefix: $TEST_PREFIX"

    # Check if service is running
    log_info "Checking if service is running..."
    if ! curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/" | grep -q "200"; then
        log_error "Service is not running at $BASE_URL"
        exit 1
    fi
    log_success "Service is running"

    # Run all tests
    test_health_check
    test_create_user
    test_list_users
    test_get_user_by_id
    test_get_user_by_username
    test_delete_user
    test_emqx_endpoints

    # Cleanup
    cleanup

    # Print summary
    print_summary
}

# Run main function
main "$@"
