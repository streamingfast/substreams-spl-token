# Build targets
.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

# Database targets
.PHONY: db-up
db-up:
	docker-compose up -d clickhouse

.PHONY: db-down
db-down:
	docker-compose down

.PHONY: db-setup
db-setup: db-up
	@echo "Waiting for ClickHouse to be ready..."
	@sleep 10
	docker exec spl-token-clickhouse /docker-entrypoint-initdb.d/setup-database.sh

.PHONY: db-shell
db-shell:
	docker exec -it spl-token-clickhouse clickhouse-client --database spl2

.PHONY: db-logs
db-logs:
	docker-compose logs -f clickhouse

.PHONY: db-reset
db-reset: db-down
	docker volume rm substreams-spl-token_clickhouse_data || true
	$(MAKE) db-up

# Substreams targets
.PHONY: stream_local
stream_local: build
	substreams run substreams.yaml map_spl_instructions --plaintext -e localhost:9000 -s $(START_BLOCK) -t +1

.PHONY: stream_fleet_payments
stream_fleet_payments: build
	substreams run substreams.yaml map_spl_instructions -e mainnet.sol.streamingfast.io:443 -s 180279461 -t +1

.PHONY: stream_regular_payments
stream_regular_payments: build
	substreams run substreams.yaml map_spl_instructions -e mainnet.sol.streamingfast.io:443 -s 200974959 -t +1

.PHONY: stream_ai_trainer_payments
stream_ai_trainer_payments: build
	substreams run substreams.yaml map_spl_instructions -e mainnet.sol.streamingfast.io:443 -s 200975925 -t +1

# Sink targets
.PHONY: sink-run
sink-run: build db-up
	substreams-sink-sql run clickhouse://default@localhost:9000/spl2 substreams.yaml

.PHONY: sink-setup
sink-setup: db-up
	substreams-sink-sql setup clickhouse://default@localhost:9000/spl2 substreams.yaml

# Package targets
.PHONY: package
package: build
	substreams pack substreams.yaml

# Development targets
.PHONY: dev-setup
dev-setup: db-setup sink-setup
	@echo "Development environment ready!"
	@echo "Run 'make sink-run' to start streaming data"
	@echo "Run 'make db-shell' to query the database"

.PHONY: clean
clean:
	cargo clean
	docker-compose down -v

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build           - Build the Rust WASM module"
	@echo "  package         - Create .spkg package"
	@echo ""
	@echo "Database:"
	@echo "  db-up           - Start ClickHouse database"
	@echo "  db-down         - Stop ClickHouse database"
	@echo "  db-setup        - Initialize database schema"
	@echo "  db-shell        - Open ClickHouse client shell"
	@echo "  db-logs         - Show ClickHouse logs"
	@echo "  db-reset        - Reset database (removes all data)"
	@echo ""
	@echo "Streaming:"
	@echo "  stream_local    - Stream to local endpoint"
	@echo "  stream_*        - Stream specific payment types"
	@echo ""
	@echo "Sink:"
	@echo "  sink-setup      - Setup database tables for sink"
	@echo "  sink-run        - Run the SQL sink"
	@echo ""
	@echo "Development:"
	@echo "  dev-setup       - Complete development setup"
	@echo "  clean           - Clean build artifacts and containers"
