# EMQX Auth Plugin - Makefile for Development Automation

.PHONY: help dev start install-watch build build-docker start-docker push pull-docker start-compose stop-compose clean key

# Default target
help:
	@echo "EMQX Auth Plugin - Available Commands:"
	@echo ""
	@echo "  make dev              - Run development server with hot reload"
	@echo "  make start            - Run development server without hot reload"
	@echo "  make install-watch    - Install cargo-watch for hot reload"
	@echo "  make build            - Build release binary"
	@echo "  make build-docker     - Build Docker image"
	@echo "  make start-docker     - Run Docker image (with .env)"
	@echo "  make dev-docker       - Run development environment (Docke Fast Mode)"
	@echo "  make dev-docker-build - Run development environment (Docker Forced Rebuild)"
	@echo "  make dev-docker-stop  - Stop development environment (Docker)"
	@echo "  make push             - Push Docker image to GHCR"
	@echo "  make pull-docker      - Pull latest image for Docker Compose"
	@echo "  make start-compose    - Start Docker Compose stack"
	@echo "  make stop-compose     - Stop Docker Compose stack"
	@echo "  make clean            - Clean build artifacts"
	@echo "  make key              - Generate SHA256 hash"
	@echo ""

# Run development server with hot reload (requires cargo-watch)
dev:
	@echo "🚀 Starting development server with hot reload..."
	@echo "💡 Tip: Install cargo-watch with 'make install-watch' if not installed"
	@cargo watch -x run || (echo "❌ cargo-watch not found. Installing..." && cargo install cargo-watch && cargo watch -x run)

# Run development server with hot reload
start:
	@echo "🚀 Starting development server (no hot reload)..."
	cargo run

# Run development environment with Docker Compose (fast start, uses cache/existing images)
dev-docker:
	@echo "🚀 Starting development environment in Docker (Fast Mode)..."
	@trap 'echo "🛑 Cleaning up..."; docker compose --env-file .env.dev -f docker-compose.dev.yml down --remove-orphans --volumes' EXIT INT TERM; \
	docker compose --env-file .env.dev -f docker-compose.dev.yml down --remove-orphans --volumes; \
	docker compose --env-file .env.dev -f docker-compose.dev.yml up

# Run development environment with forced rebuild
dev-docker-build:
	@echo "🚀 Starting development environment in Docker (Forced Rebuild)..."
	@trap 'echo "🛑 Cleaning up..."; docker compose --env-file .env.dev -f docker-compose.dev.yml down --remove-orphans --volumes' EXIT INT TERM; \
	docker compose --env-file .env.dev -f docker-compose.dev.yml down --remove-orphans --volumes; \
	docker compose --env-file .env.dev -f docker-compose.dev.yml up --build

# Stop development environment and clean up
dev-docker-stop:
	@echo "🛑 Stopping development environment..."
	docker compose --env-file .env.dev -f docker-compose.dev.yml down --remove-orphans --volumes

# Install cargo-watch for hot reload
install-watch:
	@echo "📦 Installing cargo-watch..."
	cargo install cargo-watch
	@echo "✅ cargo-watch installed successfully"

# Build release binary
build:
	@echo "🔨 Building release binary..."
	cargo build --release

# --- Docker Configuration ---
DOCKER_IMAGE_NAME = emqx-auth-plugin
GHCR_REPO = ghcr.io/farismnrr/emqx-auth-plugin

# Build via Docker
docker: build-docker
build-docker:
	@read -p "Enter Docker tag (default: latest): " tag; \
	tag=$${tag:-latest}; \
	echo "🐳 Building Docker image with tag: $$tag..."; \
	docker build -t $(DOCKER_IMAGE_NAME):$$tag -t $(GHCR_REPO):$$tag .; \
	echo "✅ Image tagged as $(DOCKER_IMAGE_NAME):$$tag and $(GHCR_REPO):$$tag"

# Run via Docker (with .env)
start-docker:
	@read -p "Enter Docker tag to run (default: latest): " tag; \
	tag=$${tag:-latest}; \
	echo "🚀 Starting Docker container with tag: $$tag..."; \
	docker run --rm -it --network="host" --env-file .env $(DOCKER_IMAGE_NAME):$$tag

# Push to GHCR
push:
	@echo "🚀 Triggering GitHub Actions workflow for Docker push..."
	@gh workflow run build-emqx-auth-plugin.yml --ref main
	@echo "✅ Workflow dispatched. Track with 'gh run watch --latest'"

# --- Docker Compose Configuration ---

# Pull latest image for Docker Compose
pull-docker:
	@echo "📥 Pulling latest Docker image..."
	docker compose pull

# Start Docker Compose
start-compose: pull-docker
	@echo "🚀 Starting Docker Compose stack..."
	docker compose up -d

# Stop Docker Compose
stop-compose:
	@echo "🛑 Stopping Docker Compose stack..."
	docker compose down

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean completed"

# Generate random SHA256 hash
key:
	@echo "Generated SHA256 hash:"
	@openssl rand -hex 32 | sha256sum | awk '{print $$1}'
