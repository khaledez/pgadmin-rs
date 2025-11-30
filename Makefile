.PHONY: help build dev prod stop clean logs shell test

# Colors for output
CYAN := \033[36m
RESET := \033[0m

help:
	@echo "$(CYAN)pgAdmin-rs Development Commands$(RESET)"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start development environment with Docker Compose"
	@echo "  make build        - Build Docker image"
	@echo "  make logs         - View application logs"
	@echo "  make shell        - Open shell in running container"
	@echo "  make stop         - Stop Docker Compose services"
	@echo "  make clean        - Remove containers and volumes"
	@echo ""
	@echo "Production:"
	@echo "  make prod         - Start production environment"
	@echo "  make prod-build   - Build production Docker image"
	@echo ""
	@echo "Local Development:"
	@echo "  make test         - Run tests"
	@echo "  make check        - Run cargo check"
	@echo "  make clippy       - Run clippy linter"
	@echo "  make fmt          - Format code"
	@echo ""

# ============================================================================
# Development Commands
# ============================================================================

dev:
	@echo "$(CYAN)Starting development environment...$(RESET)"
	docker-compose up

dev-detached:
	@echo "$(CYAN)Starting development environment (detached)...$(RESET)"
	docker-compose up -d

build:
	@echo "$(CYAN)Building Docker image...$(RESET)"
	docker build -t pgadmin-rs:latest .
	@echo "$(CYAN)Image built successfully!$(RESET)"

logs:
	docker-compose logs -f app

shell:
	docker exec -it pgadmin-rs-app /bin/bash

stop:
	@echo "$(CYAN)Stopping services...$(RESET)"
	docker-compose down

clean:
	@echo "$(CYAN)Removing containers and volumes...$(RESET)"
	docker-compose down -v
	@echo "$(CYAN)Cleanup complete!$(RESET)"

# ============================================================================
# Production Commands
# ============================================================================

prod:
	@echo "$(CYAN)Starting production environment...$(RESET)"
	docker-compose -f docker-compose.prod.yml up -d

prod-build:
	@echo "$(CYAN)Building production Docker image...$(RESET)"
	docker build -t pgadmin-rs:latest .

# ============================================================================
# Local Development Commands
# ============================================================================

test:
	@echo "$(CYAN)Running tests...$(RESET)"
	cargo test

check:
	@echo "$(CYAN)Running cargo check...$(RESET)"
	cargo check

clippy:
	@echo "$(CYAN)Running clippy...$(RESET)"
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	@echo "$(CYAN)Formatting code...$(RESET)"
	cargo fmt --all

fmt-check:
	@echo "$(CYAN)Checking code format...$(RESET)"
	cargo fmt --all -- --check

# ============================================================================
# Utility Commands
# ============================================================================

# Show image size
size:
	@echo "$(CYAN)Docker image size:$(RESET)"
	@docker images pgadmin-rs:latest --format "table {{.Repository}}\t{{.Size}}"

# Run health check
health:
	curl -f http://localhost:3000/health || echo "Health check failed"

# View image layers
layers:
	docker history pgadmin-rs:latest

# Inspect running container
inspect:
	docker inspect pgadmin-rs-app

.DEFAULT_GOAL := help
