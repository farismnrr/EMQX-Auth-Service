# Small Makefile helpers to control services defined in docker-compose.yml

COMPOSE := docker compose
COMPOSE_DEV := docker compose -f docker-compose-dev.yml

.PHONY: help dev dev\ stop dev\ logs dev\ restart build push key clean start-mysql-dev stop-mysql-dev kill emqx emqx\ setup emqx\ logs emqx\ restart prod prod\ up prod\ down prod\ logs prod\ restart

.DEFAULT_GOAL := help

help:
	@echo "EMQX Auth Service - Available Commands:"
	@echo ""
	@echo "  --- LOCAL DEV ---"
	@echo "  make dev              - Start full dev environment (EMQX + Auth Service + MySQL)"
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
	@echo "  --- DATABASE ---"
	@echo "  make start-mysql-dev  - Start dev MySQL container"
	@echo "  make stop-mysql-dev   - Stop dev MySQL container"
	@echo ""
	@echo "  --- DOCKER & DEPLOY ---"
	@echo "  make build            - Build Docker plugin image"
	@echo "  make push             - Push local image to GHCR"
	@echo "  make pull             - Pull latest images from GHCR"
	@echo ""

# ==============================================================================
# Development Environment
# ==============================================================================

## Start full development environment (EMQX + Auth Service + MySQL)
dev:
	@echo "🚀 Starting development environment..."
	@echo ""
	@echo "📦 Starting MySQL..."
	@$(COMPOSE_DEV) up -d mysql --wait
	@sleep 3
	@echo ""
	@echo "📦 Starting EMQX Broker..."
	@$(COMPOSE_DEV) up -d emqx
	@echo ""
	@echo "⏳ Waiting for EMQX to initialize..."
	@sleep 15
	@echo ""
	@echo "🔧 Running EMQX auto-configuration..."
	@bash scripts/emqx-setup.sh
	@echo ""
	@echo "⏳ Waiting for Auth Service to start..."
	@sleep 5
	@echo ""
	@echo "🚀 Starting Auth Service with hot reload..."
	@cargo watch -x run

## Stop all development services
dev\ stop:
	@echo "🛑 Stopping all development services..."
	@pkill -f "cargo.*run" 2>/dev/null || true
	@$(COMPOSE_DEV) down
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
# Database Management
# ==============================================================================

## Start dev MySQL container
start-mysql-dev:
	@$(COMPOSE_DEV) up -d mysql --wait
	@echo "✅ MySQL started"

## Stop dev MySQL container
stop-mysql-dev:
	@$(COMPOSE_DEV) down mysql
	@echo "✅ MySQL stopped"

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
	@lsof -ti:5500 | xargs -r kill -9 || echo "✅ No process running on port 5500"

# ==============================================================================
# Docker & Deploy
# ==============================================================================

## Build project and Docker plugin
build:
	@bash autobuild.sh

## Push to GHCR (no rebuild, just push local image)
push:
	@bash autobuild.sh --push

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
