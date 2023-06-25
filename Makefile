default: help

.PHONY: help
help: ## Print this help message
	@echo "Available make commands:"; grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: setup
setup: ## Install dev tooling
	cargo install sqlx-cli

.envrc: ## Setup .envrc
	ln -s .envrc.example .envrc

.PHONY: tools
tools: ## Install dev tools
	cargo install --version="~0.6" sqlx-cli --no-default-features --features rustls,postgres

.PHONY: postgres
postgres: ## Start a postgres container with high connections for testing purposes
	docker run \
      -e POSTGRES_USER=postgres \
      -e POSTGRES_PASSWORD=postgres \
      -e POSTGRES_DB=chain_registry \
      -p 5432:5432 \
      -d postgres \
      postgres -N 1000

.PHONY: psql
psql: ## Connect to the test postgres container
	psql -h localhost -U postgres -d chain_registry

.PHONY: prepare
prepare: ## Prepare sqlx offline data
	cargo sqlx prepare -- --tests

.PHONY: test
test: prepare ## Run unit and quick integration tests
	cargo test

.PHONY: test-all
test-all: prepare ## Run all tests including longer integration tests
	cargo test -- --include-ignored

.PHONY: watch
watch: prepare ## Watch for changes and run cargo
	cargo watch -x check