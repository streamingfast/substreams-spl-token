# Example Queries for SPL Token Data

This document provides practical examples of how to query the SPL token data stored in ClickHouse. All queries can be executed using the ClickHouse client through Docker.

## Prerequisites

Make sure your Docker environment is running:
```bash
docker-compose up -d
```

## Basic Query Commands

### Using Docker Exec
```bash
# Connect to ClickHouse and run a query
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "YOUR_QUERY_HERE"

# Interactive mode
docker exec -it spl-token-clickhouse clickhouse-client --database spl2
```

### Using HTTP Interface
```bash
# Query via HTTP (useful for scripts)
curl "http://localhost:8123/?database=spl2&query=YOUR_QUERY_HERE"
```

## Example Queries

### 1. Check Data Availability

**Check if tables exist and have data:**
```sql
-- List all tables in the database
SHOW TABLES FROM spl2;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "SHOW TABLES"
```

**Check row counts:**
```sql
-- Get row counts for all main tables
SELECT 
    'mints' as table_name, count() as row_count FROM spl2.mints
UNION ALL
SELECT 
    'burns' as table_name, count() as row_count FROM spl2.burns  
UNION ALL
SELECT 
    'transfers' as table_name, count() as row_count FROM spl2.transfers
UNION ALL
SELECT 
    'initialized_accounts' as table_name, count() as row_count FROM spl2.initialized_accounts;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 'mints' as table_name, count() as row_count FROM spl2.mints
UNION ALL SELECT 'burns' as table_name, count() as row_count FROM spl2.burns  
UNION ALL SELECT 'transfers' as table_name, count() as row_count FROM spl2.transfers
UNION ALL SELECT 'initialized_accounts' as table_name, count() as row_count FROM spl2.initialized_accounts
"
```

### 2. Token Supply Analysis

**Current total supply:**
```sql
-- Get current total supply from materialized view
SELECT * FROM spl2.mv_supply;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "SELECT * FROM spl2.mv_supply"
```

**Monthly mint/burn trends:**
```sql
-- Compare monthly mints vs burns
SELECT 
    m.month,
    m.total as mints,
    COALESCE(b.total, 0) as burns,
    m.total - COALESCE(b.total, 0) as net_change
FROM spl2.mv_mint_per_month m
LEFT JOIN spl2.mv_burn_per_month b ON m.month = b.month
ORDER BY m.month DESC
LIMIT 12;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    m.month,
    m.total as mints,
    COALESCE(b.total, 0) as burns,
    m.total - COALESCE(b.total, 0) as net_change
FROM spl2.mv_mint_per_month m
LEFT JOIN spl2.mv_burn_per_month b ON m.month = b.month
ORDER BY m.month DESC
LIMIT 12
"
```

### 3. Transaction Analysis

**Recent mint transactions:**
```sql
-- Get the 10 most recent mint transactions
SELECT 
    block_number,
    block_time,
    to_derive_address,
    to_owner_address,
    amount
FROM spl2.mv_all_mints
ORDER BY block_number DESC
LIMIT 10;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    block_number,
    block_time,
    to_derive_address,
    to_owner_address,
    amount
FROM spl2.mv_all_mints
ORDER BY block_number DESC
LIMIT 10
"
```

**Largest transactions by amount:**
```sql
-- Find the largest mint transactions
SELECT 
    block_number,
    block_time,
    to_derive_address,
    amount,
    amount / 1000000000 as amount_tokens  -- Assuming 9 decimals
FROM spl2.mv_all_mints
ORDER BY amount DESC
LIMIT 10;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    block_number,
    block_time,
    to_derive_address,
    amount,
    amount / 1000000000 as amount_tokens
FROM spl2.mv_all_mints
ORDER BY amount DESC
LIMIT 10
"
```

### 4. Account Analysis

**Most active accounts (by transaction count):**
```sql
-- Find accounts with the most mint transactions
SELECT 
    to_owner_address,
    count() as mint_count,
    sum(amount) as total_minted,
    sum(amount) / 1000000000 as total_tokens_minted
FROM spl2.mv_all_mints
WHERE to_owner_address IS NOT NULL
GROUP BY to_owner_address
ORDER BY mint_count DESC
LIMIT 10;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    to_owner_address,
    count() as mint_count,
    sum(amount) as total_minted,
    sum(amount) / 1000000000 as total_tokens_minted
FROM spl2.mv_all_mints
WHERE to_owner_address IS NOT NULL
GROUP BY to_owner_address
ORDER BY mint_count DESC
LIMIT 10
"
```

### 5. Time-based Analysis

**Daily activity over the last 30 days:**
```sql
-- Daily mint/burn activity
SELECT 
    DATE(block_time) as date,
    count() as transaction_count,
    sum(amount) as total_amount,
    sum(amount) / 1000000000 as total_tokens
FROM spl2.mv_all_mints
WHERE block_time >= now() - INTERVAL 30 DAY
GROUP BY DATE(block_time)
ORDER BY date DESC;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    DATE(block_time) as date,
    count() as transaction_count,
    sum(amount) as total_amount,
    sum(amount) / 1000000000 as total_tokens
FROM spl2.mv_all_mints
WHERE block_time >= now() - INTERVAL 30 DAY
GROUP BY DATE(block_time)
ORDER BY date DESC
"
```

### 6. Data Quality Checks

**Check for duplicate transactions:**
```sql
-- Find any duplicate instruction IDs (should be none)
SELECT 
    instruction_id,
    count() as duplicate_count
FROM spl2.mv_all_mints
GROUP BY instruction_id
HAVING count() > 1
ORDER BY duplicate_count DESC;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    instruction_id,
    count() as duplicate_count
FROM spl2.mv_all_mints
GROUP BY instruction_id
HAVING count() > 1
ORDER BY duplicate_count DESC
"
```

**Data freshness check:**
```sql
-- Check how recent the data is
SELECT 
    'mints' as table_name,
    max(block_time) as latest_timestamp,
    now() - max(block_time) as age
FROM spl2.mv_all_mints
UNION ALL
SELECT 
    'burns' as table_name,
    max(block_time) as latest_timestamp,
    now() - max(block_time) as age
FROM spl2.mv_all_burns;
```

```bash
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT 
    'mints' as table_name,
    max(block_time) as latest_timestamp,
    now() - max(block_time) as age
FROM spl2.mv_all_mints
UNION ALL
SELECT 
    'burns' as table_name,
    max(block_time) as latest_timestamp,
    now() - max(block_time) as age
FROM spl2.mv_all_burns
"
```

## Advanced Queries

### Custom Time Range Analysis
```sql
-- Analyze activity in a specific time range
SELECT 
    DATE_TRUNC('hour', block_time) as hour,
    count() as transactions,
    sum(amount) / 1000000000 as tokens_minted
FROM spl2.mv_all_mints
WHERE block_time BETWEEN '2024-01-01' AND '2024-01-02'
GROUP BY DATE_TRUNC('hour', block_time)
ORDER BY hour;
```

### Account Balance Tracking
```sql
-- Track balance changes for a specific account
SELECT 
    block_time,
    'mint' as operation,
    amount / 1000000000 as token_amount
FROM spl2.mv_all_mints
WHERE to_owner_address = 'YOUR_ACCOUNT_ADDRESS'
UNION ALL
SELECT 
    block_time,
    'burn' as operation,
    -amount / 1000000000 as token_amount
FROM spl2.mv_all_burns
WHERE from_owner_address = 'YOUR_ACCOUNT_ADDRESS'
ORDER BY block_time;
```

## Tips for Query Performance

1. **Use LIMIT**: Always use LIMIT for exploratory queries to avoid overwhelming results
2. **Index on time**: Queries filtering by `block_time` are generally fast due to indexing
3. **Materialized Views**: Use the `mv_*` tables for aggregated data when possible
4. **Decimal Conversion**: Remember to divide amounts by 10^9 (1000000000) to get human-readable token amounts

## Troubleshooting

If queries fail:
1. Check if the database is running: `docker-compose ps`
2. Verify tables exist: `docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "SHOW TABLES"`
3. Check if data is being ingested by substreams-sink-sql
4. Review ClickHouse logs: `docker-compose logs clickhouse`

