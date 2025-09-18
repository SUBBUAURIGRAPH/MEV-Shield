# MEV Shield Makefile
# Provides convenient commands for development and deployment

.PHONY: help build deploy clean test lint check dev stop logs shell backup restore

# Default target
help: ## Show this help message
	@echo "MEV Shield - Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Examples:"
	@echo "  make deploy          # Build and deploy locally"
	@echo "  make dev             # Start development environment"
	@echo "  make clean           # Clean rebuild"
	@echo "  make logs            # View service logs"

# Build commands
build: ## Build Rust backend and React dashboard
	@echo "🔨 Building MEV Shield..."
	cargo build --release
	cd dashboard && npm ci && npm run build

build-docker: ## Build Docker images
	@echo "🐳 Building Docker images..."
	docker build -t mev-shield:local .
	docker build -f dashboard/Dockerfile.admin -t mev-shield-admin:local ./dashboard
	docker build -f dashboard/Dockerfile.user -t mev-shield-user:local ./dashboard

# Deployment commands
deploy: ## Deploy MEV Shield locally with Docker Compose
	@echo "🚀 Deploying MEV Shield locally..."
	./build-and-deploy-local.sh

deploy-clean: ## Clean deployment (removes old Docker images)
	@echo "🧹 Clean deployment..."
	./build-and-deploy-local.sh --clean

rebuild: ## Rebuild and redeploy
	@echo "🔄 Rebuilding and redeploying..."
	./build-and-deploy-local.sh --rebuild

# Development commands
dev: ## Start development environment
	@echo "💻 Starting development environment..."
	docker-compose -f docker-compose.local.yml up -d postgres redis
	@echo "Database and cache services started. Run 'cargo run' and 'cd dashboard && npm start' in separate terminals."

dev-full: ## Start full development environment
	@echo "💻 Starting full development environment..."
	docker-compose -f docker-compose.local.yml up -d

# Service management
stop: ## Stop all services
	@echo "⏹️ Stopping services..."
	docker-compose -f docker-compose.local.yml down

restart: ## Restart all services
	@echo "🔄 Restarting services..."
	docker-compose -f docker-compose.local.yml restart

# Monitoring and debugging
logs: ## View service logs
	@echo "📋 Viewing service logs..."
	docker-compose -f docker-compose.local.yml logs -f

logs-core: ## View MEV Shield core logs
	docker-compose -f docker-compose.local.yml logs -f mev-shield-core

logs-admin: ## View admin dashboard logs
	docker-compose -f docker-compose.local.yml logs -f admin-dashboard

logs-user: ## View user dashboard logs
	docker-compose -f docker-compose.local.yml logs -f user-dashboard

status: ## Show service status
	@echo "📊 Service Status:"
	docker-compose -f docker-compose.local.yml ps

shell: ## Open shell in MEV Shield core container
	docker-compose -f docker-compose.local.yml exec mev-shield-core /bin/sh

shell-db: ## Connect to PostgreSQL database
	docker-compose -f docker-compose.local.yml exec postgres psql -U mev_user -d mev_shield

# Testing commands
test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test
	cd dashboard && npm test -- --watchAll=false

test-rust: ## Run Rust tests only
	@echo "🦀 Running Rust tests..."
	cargo test

test-js: ## Run JavaScript tests only
	@echo "⚛️ Running React tests..."
	cd dashboard && npm test -- --watchAll=false

# Code quality
lint: ## Run linters
	@echo "🔍 Running linters..."
	cargo clippy -- -D warnings
	cd dashboard && npm run lint

format: ## Format code
	@echo "✨ Formatting code..."
	cargo fmt
	cd dashboard && npm run format

check: ## Run all checks (lint + test)
	@echo "✅ Running all checks..."
	make lint
	make test

# Cleanup commands
clean: ## Clean build artifacts and Docker resources
	@echo "🧹 Cleaning up..."
	cargo clean
	cd dashboard && rm -rf node_modules build
	docker-compose -f docker-compose.local.yml down --volumes --remove-orphans
	docker system prune -f

clean-docker: ## Clean Docker images and containers
	@echo "🐳 Cleaning Docker resources..."
	docker-compose -f docker-compose.local.yml down --volumes --remove-orphans
	docker image prune -f
	docker container prune -f
	docker volume prune -f

# Database commands
db-migrate: ## Run database migrations
	@echo "📊 Running database migrations..."
	docker-compose -f docker-compose.local.yml exec mev-shield-core diesel migration run

db-reset: ## Reset database
	@echo "🔄 Resetting database..."
	docker-compose -f docker-compose.local.yml down postgres
	docker volume rm mev-shield_postgres_data || true
	docker-compose -f docker-compose.local.yml up -d postgres

backup: ## Backup database
	@echo "💾 Backing up database..."
	mkdir -p backups
	docker-compose -f docker-compose.local.yml exec -T postgres pg_dump -U mev_user mev_shield > backups/mev_shield_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "Backup saved to backups/"

restore: ## Restore database from backup (usage: make restore BACKUP=filename)
	@echo "📥 Restoring database from $(BACKUP)..."
	docker-compose -f docker-compose.local.yml exec -T postgres psql -U mev_user -d mev_shield < backups/$(BACKUP)

# Documentation
docs: ## Generate documentation
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --open
	cd dashboard && npm run build-storybook

# Installation commands
install: ## Install dependencies
	@echo "📦 Installing dependencies..."
	cargo fetch
	cd dashboard && npm ci

install-tools: ## Install development tools
	@echo "🔧 Installing development tools..."
	cargo install diesel_cli --no-default-features --features postgres
	cargo install cargo-watch
	cargo install cargo-audit
	npm install -g @storybook/cli

# Health checks
health: ## Check service health
	@echo "🏥 Checking service health..."
	@curl -f http://localhost:8080/health && echo "✅ Core API: Healthy" || echo "❌ Core API: Unhealthy"
	@curl -f http://localhost:3001 && echo "✅ Admin Dashboard: Healthy" || echo "❌ Admin Dashboard: Unhealthy"
	@curl -f http://localhost:3002 && echo "✅ User Dashboard: Healthy" || echo "❌ User Dashboard: Unhealthy"

# Quick start
quick-start: install build deploy health ## Quick start - install, build, and deploy everything
	@echo ""
	@echo "🎉 MEV Shield is ready!"
	@echo "📖 Visit http://localhost:3002 for the user dashboard"
	@echo "🔧 Visit http://localhost:3001 for the admin dashboard"
	@echo "📊 Visit http://localhost:3000 for Grafana monitoring"