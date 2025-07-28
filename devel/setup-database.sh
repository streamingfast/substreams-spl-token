#!/bin/bash

echo "Setting up ClickHouse database for SPL Token Substreams..."

# Wait for ClickHouse to be ready
until clickhouse-client --host localhost --port 9000 --query "SELECT 1" > /dev/null 2>&1; do
    echo "Waiting for ClickHouse to be ready..."

    # There is a timing effect here when use within `db-reset` command
    # (docker compose down, remove volume, docker compose up). Use a sleep value of
    # 2 or 3 or more seems to always results in the script being killed by some timeout.
    # I'm not sure what sets this timeout with 1 it seemed to work in all cases on my computer.
    sleep 1
done

echo "ClickHouse is ready!"
set -e

# Create database if it doesn't exist
clickhouse-client --host localhost --port 9000 --query "CREATE DATABASE IF NOT EXISTS spl2"

echo "Database 'spl2' created successfully!"

# Test connection
clickhouse-client --host localhost --port 9000 --database spl2 --query "SELECT 'Database setup completed successfully!' as status"

