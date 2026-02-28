# Small Makefile helpers to control services defined in docker-compose.yml

COMPOSE := docker compose
COMPOSE_FILE := docker-compose.yml

.PHONY: help docker docker\ run docker\ stop docker\ ps build push key dev clean start-mysql-dev stop-mysql-dev kill mqtt-create mqtt-delete mqtt-create-superuser
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
	@echo "  --- MQTT USER MANAGEMENT ---"
	@echo "  make mqtt-create          - Create a regular MQTT user (auto SHA-512 password)"
	@echo "  make mqtt-delete          - Delete an MQTT user"
	@echo "  make mqtt-create-superuser - Create a superuser MQTT user (auto SHA-512 password)"
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

# Start Database for development
start-db:
	@DB_TYPE=$$(grep -E '^DB_TYPE=' .env 2>/dev/null | cut -d '=' -f2- | tr -d '"' | tr -d "'"); \
	DB_TYPE=$${DB_TYPE:-mysql}; \
	if [ "$$DB_TYPE" = "postgres" ]; then \
		echo "🐘 Starting PostgreSQL..."; \
		docker compose -f docker-compose-dev.yml up -d --wait --remove-orphans postgres; \
	else \
		echo "🐬 Starting MySQL..."; \
		docker compose -f docker-compose-dev.yml up -d --wait --remove-orphans mysql; \
	fi

# Stop Database for development
stop-db:
	docker compose -f docker-compose-dev.yml down -v
	rm -rf ./rocksdb-data

# Run with hot reload
dev:
	@trap '$(MAKE) stop-db' EXIT INT TERM; \
	set -e; \
	$(MAKE) start-db; \
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

# MQTT User Management
# Reads AUTH_SERVICE_URL and API_KEY from .env, falls back to defaults
AUTH_SERVICE_URL ?= $(shell grep -E '^AUTH_SERVICE_URL=' .env 2>/dev/null | cut -d '=' -f2- | tr -d '"' | tr -d "'")
AUTH_SERVICE_URL := $(if $(AUTH_SERVICE_URL),$(AUTH_SERVICE_URL),http://127.0.0.1:5500)
AUTH_API_KEY ?= $(shell grep -E '^API_KEY=' .env 2>/dev/null | cut -d '=' -f2- | tr -d '"' | tr -d "'")

# Create a regular MQTT user with auto-generated password
mqtt-create:
	@read -p "Enter MQTT username: " username; \
	password=$$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 32); \
	hashed=$$(echo -n "$$password" | openssl dgst -sha512 | awk '{print $$2}'); \
	echo ""; \
	echo "📋 Username : $$username"; \
	echo "🔑 Password : $$password"; \
	echo "🔐 SHA-512  : $$hashed"; \
	echo ""; \
	response=$$(curl -s -o /tmp/mqtt_resp.json -w "%{http_code}" \
		-X POST "$(AUTH_SERVICE_URL)/mqtt/create" \
		-H "Content-Type: application/json" \
		-H "Authorization: Bearer $(AUTH_API_KEY)" \
		-d "{\"username\":\"$$username\",\"password\":\"$$password\",\"is_superuser\":false}"); \
	if [ "$$response" = "200" ]; then \
		echo "✅ MQTT user '$$username' created successfully!"; \
	else \
		echo "❌ Failed to create user (HTTP $$response):"; \
		cat /tmp/mqtt_resp.json; echo; \
	fi

# Delete an MQTT user (soft delete)
mqtt-delete:
	@read -p "Enter MQTT username to delete: " username; \
	echo ""; \
	response=$$(curl -s -o /tmp/mqtt_resp.json -w "%{http_code}" \
		-X DELETE "$(AUTH_SERVICE_URL)/mqtt/$$username" \
		-H "Content-Type: application/json" \
		-H "Authorization: Bearer $(AUTH_API_KEY)"); \
	if [ "$$response" = "200" ]; then \
		echo "✅ MQTT user '$$username' deleted successfully!"; \
	else \
		echo "❌ Failed to delete user (HTTP $$response):"; \
		cat /tmp/mqtt_resp.json; echo; \
	fi

# Create a superuser MQTT user with auto-generated password
mqtt-create-superuser:
	@read -p "Enter MQTT superuser username: " username; \
	password=$$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 32); \
	hashed=$$(echo -n "$$password" | openssl dgst -sha512 | awk '{print $$2}'); \
	echo ""; \
	echo "📋 Username : $$username"; \
	echo "🔑 Password : $$password"; \
	echo "🔐 SHA-512  : $$hashed"; \
	echo "👑 Role     : superuser"; \
	echo ""; \
	response=$$(curl -s -o /tmp/mqtt_resp.json -w "%{http_code}" \
		-X POST "$(AUTH_SERVICE_URL)/mqtt/create" \
		-H "Content-Type: application/json" \
		-H "Authorization: Bearer $(AUTH_API_KEY)" \
		-d "{\"username\":\"$$username\",\"password\":\"$$password\",\"is_superuser\":true}"); \
	if [ "$$response" = "200" ]; then \
		echo "✅ MQTT superuser '$$username' created successfully!"; \
	else \
		echo "❌ Failed to create superuser (HTTP $$response):"; \
		cat /tmp/mqtt_resp.json; echo; \
	fi
