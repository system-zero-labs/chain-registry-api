default: help

.PHONY: help
help: ## Print this help message
	@echo "Available make commands:"; grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.envrc: ## Setup .envrc
	ln -s .envrc.example .envrc

.PHONY: tools
tools: ## Install dev tools
	cargo install --version="~0.6" sqlx-cli --no-default-features --features rustls,postgres

.PHONY: postgres
postgres: ## Start a local postgres db
	brew services start postgresql@15

.PHONY: stop-postgres
stop-postgres: ## Stop the postgres db
	brew services stop postgresql@15

.PHONY: initdb
initdb: ## Initialize the local postgres db
	psql postgres -c "CREATE ROLE postgres with SUPERUSER LOGIN"
	createdb chain_registry

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