#!/bin/bash

# Setup script for ClickHouse database initialization
# This script creates the necessary database and prepares for table creation

set -e

echo "Setting up ClickHouse database for SPL Token Substreams..."

# Wait for ClickHouse to be ready
until clickhouse-client --host localhost --port 9000 --query "SELECT 1" > /dev/null 2>&1; do
    echo "Waiting for ClickHouse to be ready..."
    sleep 2
done

echo "ClickHouse is ready!"

# Create database if it doesn't exist
clickhouse-client --host localhost --port 9000 --query "CREATE DATABASE IF NOT EXISTS spl2"

echo "Database 'spl2' created successfully!"
echo "Note: Tables and materialized views will be created automatically by substreams-sink-sql"
echo "You can manually create views using the views.sql file if needed"

# Test connection
clickhouse-client --host localhost --port 9000 --database spl2 --query "SELECT 'Database setup completed successfully!' as status"

