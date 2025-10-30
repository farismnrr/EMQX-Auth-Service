#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default DB path (host-side)
: "${ROCKSDB_PATH:="$SCRIPT_DIR/rocksdb-data/iotnet"}"
if [ ! -d "$ROCKSDB_PATH" ]; then
  ROCKSDB_PATH="$SCRIPT_DIR"
fi

echo "[*] Checking RocksDB connection..."

find_local_ldb() {
  if command -v rocksdb_ldb >/dev/null 2>&1; then
    echo "rocksdb_ldb"
  elif command -v ldb >/dev/null 2>&1; then
    echo "ldb"
  else
    return 1
  fi
}

COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
docker_available() { command -v docker >/dev/null 2>&1; }

if LDB_LOCAL="$(find_local_ldb 2>/dev/null || true)" && [ -n "$LDB_LOCAL" ]; then
  MODE="local"
  LDB_CMD="$LDB_LOCAL"
else
  if docker_available && [ -f "$COMPOSE_FILE" ]; then
    MODE="docker"
    if docker ps --format '{{.Names}}' | grep -q '^rocksdb$'; then
      DOCKER_MODE="exec"
    else
      DOCKER_MODE="compose-run"
    fi
  else
    echo "❌ Could not find a local 'ldb' and Docker Compose is not available or $COMPOSE_FILE is missing."
    exit 1
  fi
fi

echo "[*] Using DB path: $ROCKSDB_PATH (mode: $MODE${DOCKER_MODE:+, docker_mode: $DOCKER_MODE})"

# If in Docker mode, translate the host path to the container path
if [ "$MODE" = "docker" ]; then
  # automatic translation: assumes volume ./rocksdb-data:/data
  CONTAINER_DB_PATH="/data/iotnet"
else
  CONTAINER_DB_PATH="$ROCKSDB_PATH"
fi

run_ldb() {
  if [ "$MODE" = "local" ]; then
    "$LDB_CMD" "$@"
  else
    if [ "$DOCKER_MODE" = "exec" ]; then
      docker exec rocksdb /usr/local/bin/ldb "$@"
    else
      if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
        docker compose -f "$COMPOSE_FILE" run --rm rocksdb /usr/local/bin/ldb "$@"
      else
        docker-compose -f "$COMPOSE_FILE" run --rm rocksdb /usr/local/bin/ldb "$@"
      fi
    fi
  fi
}

TEST_KEY="check_connection_test_key_$$"
TEST_VALUE="check_connection_test_value"

# Use translated path in Docker mode
if ! run_ldb --db="$CONTAINER_DB_PATH" put "$TEST_KEY" "$TEST_VALUE" >/dev/null 2>&1; then
  echo "❌ Failed to write to RocksDB (mode: $MODE)."
  echo "If using Docker, ensure the 'rocksdb' service is defined in $COMPOSE_FILE and that artifacts/bin 'ldb' is available inside the container."
  exit 1
fi

read_out="$(run_ldb --db="$CONTAINER_DB_PATH" get "$TEST_KEY" 2>/dev/null || true)"
if [[ -z "$read_out" || "$read_out" != *"$TEST_VALUE"* ]]; then
  echo "❌ RocksDB read test failed (got: ${read_out:-<empty>})."
  exit 1
fi

run_ldb --db="$CONTAINER_DB_PATH" delete "$TEST_KEY" >/dev/null 2>&1 || true

echo "✅ RocksDB connection and read/write test OK (DB: $ROCKSDB_PATH, mode: $MODE)"
exit 0
