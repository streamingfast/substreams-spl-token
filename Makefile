.PHONY: db-up
db-up:
	docker-compose up -d clickhouse
	$(MAKE)	db-setup

.PHONY: db-down
db-down:
	docker-compose down

.PHONY: db-setup
db-setup:
	# Initialize the ClickHouse database, setup-database.sh comes from ./setup-database.sh at root of project (mounted in docker-compose.yml)
	docker exec spl-token-clickhouse /docker-entrypoint-initdb.d/setup-database.sh
	sleep 1
	docker-compose up -d clickhouse

.PHONY: db-shell
db-shell:
	docker exec -it spl-token-clickhouse clickhouse-client --database spl2

.PHONY: db-query
db-query:
	docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "$(QUERY)"

.PHONY: db-logs
db-logs:
	docker-compose logs -f clickhouse

.PHONY: db-reset
db-reset: db-clean db-up
	rm -f cursor.txt
	rm -f ./spl2_schema_hash.txt

.PHONY: db-clean
db-clean:
	docker-compose down -v
	@echo ""
	@echo "Database volume removed. You can now run 'make db-up' to start with a fresh database."

.PHONY: help
help:
	@echo "Database:"
	@echo "  db-up           - Start ClickHouse database & initialize it"
	@echo "  db-down         - Stop ClickHouse database"
	@echo "  db-shell        - Open ClickHouse client shell"
	@echo "  db-query        - Execute SQL query (use: make db-query QUERY='SELECT * FROM table')"
	@echo "  db-logs         - Show ClickHouse logs"
	@echo "  db-reset        - Reset database (removes all data) and restart everything"
