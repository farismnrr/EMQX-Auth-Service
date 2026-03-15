# Small Makefile helpers to control services defined in docker-compose.yml

COMPOSE := docker compose
COMPOSE_DEV := docker compose -f docker-compose-dev.yml

.PHONY: help dev dev\ stop dev\ logs dev\ restart build push key clean kill emqx emqx\ setup emqx\ logs emqx\ restart prod prod\ up prod\ down prod\ logs prod\ restart

.DEFAULT_GOAL := help

help:
	@echo "EMQX Auth Service - Available Commands:"
	@echo ""
	@echo "  --- LOCAL DEV ---"
	@echo "  make dev              - Start auth service only with hot reload (auto cleanup on Ctrl+C)"
	@echo "  make dev stop         - Stop all dev services"
	@echo "  make dev logs         - View logs from all services"
	@echo "  make dev restart      - Restart all dev services"
	@echo "  make key              - Generate secure SHA256 hash"
	@echo "  make clean            - Clean build artifacts"
	@echo ""
	@echo "  --- EMQX BROKER ---"
	@echo "  make emqx             - Start EMQX broker only"
	@echo "  make emqx setup       - Run EMQX auto-configuration manually"
	@echo "  make emqx logs        - View EMQX logs"
	@echo "  make emqx restart     - Restart EMQX broker"
	@echo ""
	@echo "  --- PRODUCTION ---"
	@echo "  make prod             - Start production environment (pull + up)"
	@echo "  make prod up          - Start production services"
	@echo "  make prod down        - Stop production services"
	@echo "  make prod logs        - View production logs"
	@echo "  make prod restart     - Restart production services"
	@echo ""
	@echo "  --- DOCKER & DEPLOY ---"
	@echo "  make build            - Build Docker plugin image"
	@echo "  make push             - Push local image to GHCR"
	@echo "  make pull             - Pull latest images from GHCR"
	@echo ""

# ==============================================================================
# Development Environment
# ==============================================================================

## Start auth service only with hot reload (auto cleanup on Ctrl+C)
dev:
	@bash scripts/dev.sh

## Stop all development services
dev\ stop:
	@echo "🛑 Stopping all development services..."
	@pkill -9 -f "target/debug/emqx_auth" 2>/dev/null || true
	@pkill -9 -f "cargo.*run" 2>/dev/null || true
	@docker stop emqx-auth-service 2>/dev/null || true
	@docker rm -f emqx-auth-service 2>/dev/null || true
	@$(COMPOSE_DEV) down 2>/dev/null || true
	@echo "✅ All services stopped"

## View logs from all services
dev\ logs:
	@$(COMPOSE_DEV) logs -f

## Restart all development services
dev\ restart: dev\ stop dev

# ==============================================================================
# EMQX Broker Management
# ==============================================================================

## Start EMQX broker only
emqx:
	@echo "🚀 Starting EMQX broker..."
	@$(COMPOSE_DEV) up -d emqx --wait
	@sleep 10
	@echo "✅ EMQX started"
	@echo ""
	@echo "💡 To configure authentication, run: make emqx setup"

## Run EMQX auto-configuration manually
emqx\ setup:
	@echo "🔧 Running EMQX auto-configuration..."
	@bash scripts/emqx-setup.sh

## View EMQX logs
emqx\ logs:
	@docker logs -f dev-emqx

## Restart EMQX broker
emqx\ restart:
	@echo "🔄 Restarting EMQX..."
	@$(COMPOSE_DEV) restart emqx
	@sleep 10
	@echo "✅ EMQX restarted"

# ==============================================================================
# Utilities
# ==============================================================================

## Generate random SHA256 hash
key:
	@echo "Generated SHA256 hash:"
	@openssl rand -hex 32 | sha256sum | awk '{print $$1}'

## Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@docker rmi emqx-auth-service:latest 2>/dev/null || true
	@echo "✓ Clean complete"

## Kill process running on port 5500
kill:
	@echo "🔪 Killing processes on port 5500..."
	@lsof -ti:5500 | xargs -r kill -9 || true
	@pkill -9 -f "target/debug/emqx_auth" 2>/dev/null || true
	@pkill -9 -f "cargo.*run" 2>/dev/null || true
	@docker stop emqx-auth-service 2>/dev/null || true
	@docker rm -f emqx-auth-service 2>/dev/null || true
	@echo "✅ Cleanup complete"

# ==============================================================================
# MQTT User Management (Classic API)
# ==============================================================================

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

# Delete an MQTT user
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

# ==============================================================================
# Docker & Deploy
# ==============================================================================

## Build project and Docker plugin
build:
	@bash scripts/autobuild.sh

## Push to GHCR (AMD64 + ARM64)
push:
	@bash scripts/autobuild.sh --push --platform linux/amd64,linux/arm64

## Push AMD64 only to GHCR
push-amd:
	@bash scripts/autobuild.sh --push --platform linux/amd64

## Push ARM only (ARM64) to GHCR
push-arm:
	@bash scripts/autobuild.sh --push --platform linux/arm64

# ==============================================================================
# Production Environment
# ==============================================================================

## Start production environment (pull + up + setup)
prod: prod\ up
	@echo ""
	@echo "⏳ Waiting for services to stabilize..."
	@sleep 10
	@echo ""
	@echo "🔧 Running EMQX auto-configuration..."
	@bash scripts/emqx-setup.sh || echo "⚠️  Auto-configuration may have already run"
	@echo ""
	@echo "✅ Production environment ready!"
	@echo ""
	@echo "📋 Service Status:"
	@$(COMPOSE) ps

## Start production services
prod\ up:
	@echo "🚀 Starting production environment..."
	@echo ""
	@echo "📦 Pulling latest images..."
	@$(COMPOSE) pull
	@echo ""
	@echo "📦 Starting services..."
	@$(COMPOSE) up -d --wait --remove-orphans
	@echo ""
	@echo "✅ Production services started"

## Stop production services
prod\ down:
	@echo "🛑 Stopping production services..."
	@$(COMPOSE) down
	@echo "✅ Production services stopped"

## View production logs
prod\ logs:
	@$(COMPOSE) logs -f

## Restart production services
prod\ restart:
	@echo "🔄 Restarting production services..."
	@$(COMPOSE) restart
	@sleep 10
	@echo "✅ Production services restarted"
