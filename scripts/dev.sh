#!/bin/bash
# Development runner with proper cleanup on Ctrl+C

set -e

# Cleanup function
cleanup() {
    echo ""
    echo "🛑  Stopping..."
    
    # Kill cargo watch and child processes
    pkill -9 -f "target/debug/emqx_auth" 2>/dev/null || true
    pkill -9 -f "cargo.*run" 2>/dev/null || true
    
    # Stop docker container if running
    docker stop emqx-auth-service 2>/dev/null || true
    
    echo "✅ Cleanup complete"
    exit 0
}

# Trap signals
trap cleanup INT TERM

# Run cargo watch
echo "🚀 Starting Auth Service with hot reload..."
echo "📌 Press Ctrl+C to stop and cleanup all processes"
echo ""

cargo watch -x run
