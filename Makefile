# Small Makefile helpers to control services defined in docker-compose.yml

COMPOSE := docker compose
COMPOSE_FILE := docker-compose.yml

.PHONY: help docker docker\ run docker\ stop docker\ ps build push key clean
.DEFAULT_GOAL := help

help:
	@echo "Usage: make <target>"
	@echo
	@echo "Build & Push:"
	@echo "  build                     Build locally (Rust + Docker image)"
	@echo "  push                      Push local image to GHCR"
	@echo
	@echo "Docker Services:"
	@echo "  docker run [service...]   Start one or more services"
	@echo "  docker stop [service...]  Stop one or more services"
	@echo "  docker ps [service...]    Show docker compose ps"
	@echo
	@echo "Utilities:"
	@echo "  key                       Generate SHA256 hash"
	@echo "  clean                     Clean build artifacts"

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

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@docker rmi emqx-auth-plugin:latest 2>/dev/null || true
	@echo "✓ Clean complete"

