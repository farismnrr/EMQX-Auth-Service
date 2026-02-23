# Small Makefile helpers to control services defined in docker-compose.yml

COMPOSE := docker compose
COMPOSE_FILE := docker-compose.yml

.PHONY: help docker docker\ run docker\ stop docker\ ps build push key dev clean start-mysql-dev stop-mysql-dev kill
.DEFAULT_GOAL := help

help:
	@echo "EMQX Auth Service - Available Commands:"
	@echo ""
	@echo "  --- LOCAL DEV ---"
	@echo "  make dev                  - Run with hot reload (cargo watch)"
	@echo "  make key                  - Generate secure SHA256 hash"
	@echo "  make kill                 - Kill processes on port 5500"
	@echo "  make clean                - Clean build artifacts"
	@echo ""
	@echo "  --- DOCKER & DEPLOY ---"
	@echo "  make build                - Build Docker plugin image"
	@echo "  make push                 - Push local image to GHCR"
	@echo "  make pull                 - Pull latest images from GHCR"
	@echo "  make docker ps            - Show running containers"
	@echo ""
	@echo "  --- DATABASE ---"
	@echo "  make start-mysql-dev      - Start dev MySQL container"
	@echo "  make stop-mysql-dev       - Stop dev MySQL container"
	@echo ""

# Docker management
docker:
	@echo "Usage: make docker <command> [service...]"
	@echo "Commands: run, stop, ps"
	@exit 1

docker\ run:
	@services="$(filter-out docker run,$(MAKECMDGOALS))"; \
	if [ -z "$$services" ]; then \
		echo "Specify service(s): make docker run rocksdb"; exit 1; \
	fi; \
	for svc in $$services; do \
		echo "Starting $$svc..."; \
		$(COMPOSE) -f $(COMPOSE_FILE) up -d $$svc; \
	done

docker\ stop:
	@services="$(filter-out docker stop,$(MAKECMDGOALS))"; \
	if [ -z "$$services" ]; then \
		echo "Specify service(s): make docker stop rocksdb"; exit 1; \
	fi; \
	for svc in $$services; do \
		echo "Stopping $$svc..."; \
		$(COMPOSE) -f $(COMPOSE_FILE) stop $$svc; \
	done

docker\ ps:
	@services="$(filter-out docker ps,$(MAKECMDGOALS))"; \
	if [ -z "$$services" ]; then \
		$(COMPOSE) -f $(COMPOSE_FILE) ps; \
	else \
		$(COMPOSE) -f $(COMPOSE_FILE) ps $$services; \
	fi

# Build project and Docker plugin
build:
	@bash autobuild.sh

# Push to GHCR (no rebuild, just push local image)
push:
	@bash autobuild.sh --push

# Generate random SHA256 hash
key:
	@echo "Generated SHA256 hash:"
	@openssl rand -hex 32 | sha256sum | awk '{print $$1}'

# Start MySQL for development
start-mysql-dev:
	docker compose -f docker-compose-dev.yml up -d --wait --remove-orphans

# Stop MySQL for development
stop-mysql-dev:
	docker compose -f docker-compose-dev.yml down -v
	rm -rf ./rocksdb-data

# Run with hot reload
dev:
	@trap '$(MAKE) stop-mysql-dev' EXIT INT TERM; \
	set -e; \
	$(MAKE) start-mysql-dev; \
	echo "🟢 MySQL started"; \
	echo "⏳ Waiting for database to settle..."; \
	sleep 3; \
	echo "🚀 Starting development server with hot reload..."; \
	cargo watch -x run

# Kill process running on port 5500
kill:
	@echo "🔪 Killing processes on port 5500..."
	@lsof -ti:5500 | xargs -r kill -9 || echo "✅ No process running on port 5500"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@docker rmi emqx-auth-service:latest 2>/dev/null || true
	@echo "✓ Clean complete"

