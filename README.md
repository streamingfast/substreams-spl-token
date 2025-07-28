# Substreams SPL Token

A comprehensive Substreams module for extracting and analyzing Solana SPL token data, specifically designed for tracking Mint, Burn, Transfer, and InitializeAccount operations. This project provides a complete data pipeline from Solana blockchain to ClickHouse database with pre-built materialized views for analytics.

## 🎯 What This Does

This Substreams extracts SPL token events from the Solana blockchain and stores them in a ClickHouse database for analysis. It's particularly useful for:

- **Token Supply Tracking**: Monitor mints, burns, and net supply changes over time
- **Account Analysis**: Track token holder behavior and distribution
- **Transaction Analytics**: Analyze token transfer patterns and volumes
- **Compliance & Reporting**: Generate reports on token activity

## 🚀 Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- [Rust](https://rustup.rs/) (for building the Substreams module)
- [Substreams CLI](https://substreams.streamingfast.io/getting-started/installing-the-cli)
- [substreams-sink-sql](https://github.com/streamingfast/substreams-sink-sql) (for Clickhouse integration)

### 1. Clone and Setup

```bash
git clone https://github.com/streamingfast/substreams-spl-token.git
cd substreams-spl-token
```

### 2. Start Local ClickHouse Database

```bash
# Start ClickHouse database
make db-up
```

This will:
- Start a ClickHouse instance on `localhost:8123` (HTTP) and `localhost:9000` (native)
- Create the `spl2` database

### 3. Build and Package

```bash
substreams build
```

### 4. Run the Data Pipeline

```bash
# Consume a subset of the chain just for demonstration purposes
substreams-sink-sql from-proto clickhouse://default:@localhost:9000/spl2 ./solana-spl-token-v0.1.0.spkg -s 356312000 -t +10000
```

### 5. Apply Materialized Views

```bash
# Apply views using the db-query command
docker exec -i spl-token-clickhouse clickhouse-client --database spl2 --multiquery < views.sql
```

### 6. Query Your Data

```bash
# Open ClickHouse client and enter query there (SELECT count() FROM mints)
make db-shell

# Or run example queries
make db-query QUERY="SELECT count() FROM mints"
```

## 📊 Database Schema

The pipeline creates the following tables in ClickHouse:

### Core Tables
- **`mints`**: Token mint operations
- **`burns`**: Token burn operations
- **`transfers`**: Token transfer operations
- **`initialized_accounts`**: Account initialization events
- **`instructions`**: Instruction metadata
- **`_blocks_`**: Block information

### Materialized Views
- **`mv_all_mints`**: Enhanced mint data with account owner information
- **`mv_all_burns`**: Enhanced burn data with account owner information
- **`mv_mint_per_month`**: Monthly mint aggregations
- **`mv_burn_per_month`**: Monthly burn aggregations
- **`mv_supply`**: Current total supply calculation

## 🔧 Configuration

### Token Configuration

Edit the `substreams.yaml` file to track a different SPL token:

```yaml
params:
  map_spl_instructions: "spl_token_address=YOUR_TOKEN_ADDRESS|spl_token_decimal=DECIMALS"
  solana_common:transactions_by_programid_and_account_without_votes: "program:TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA && account:YOUR_TOKEN_ADDRESS"
```

## 📈 Example Queries

See [example-queries.md](./example-queries.md) for comprehensive query examples. Here are a few quick ones:

### Current Token Supply
```sql
SELECT * FROM spl2.mv_supply;
```

### Recent Mint Transactions
```sql
SELECT
    block_time,
    to_derive_address,
    amount / 1000000000 as tokens_minted
FROM spl2.mv_all_mints
ORDER BY block_number DESC
LIMIT 10;
```

### Monthly Supply Changes
```sql
SELECT
    m.month,
    m.total / 1000000000 as tokens_minted,
    COALESCE(b.total, 0) / 1000000000 as tokens_burned,
    (m.total - COALESCE(b.total, 0)) / 1000000000 as net_change
FROM spl2.mv_mint_per_month m
LEFT JOIN spl2.mv_burn_per_month b ON m.month = b.month
ORDER BY m.month DESC;
```

## 🛠 Development Commands

```bash
# Database management
make db-up          # Start ClickHouse
make db-down        # Stop ClickHouse
make db-setup       # Initialize database
make db-shell       # Open database shell
make db-reset       # Reset database (removes all data)
```

## 📦 Installing substreams-sink-sql

The `substreams-sink-sql` tool is required to stream data from Substreams to ClickHouse.

### Installation Options

**Option 1: Download Pre-built Binary**
```bash
# Download from GitHub releases
curl -L https://github.com/streamingfast/substreams-sink-sql/releases/latest/download/substreams-sink-sql-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz | tar -xz
sudo mv substreams-sink-sql /usr/local/bin/
```

**Option 2: Build from Source**
```bash
git clone https://github.com/streamingfast/substreams-sink-sql.git
cd substreams-sink-sql
go build -o substreams-sink-sql ./cmd/substreams-sink-sql
```

For detailed installation instructions, see the [official documentation](https://github.com/streamingfast/substreams-sink-sql#installation).

## 🔍 Monitoring and Troubleshooting

### Check Data Pipeline Status

```bash
# View ClickHouse logs
make db-logs

# Check if data is flowing
docker exec -it spl-token-clickhouse clickhouse-client --database spl2 --query "
SELECT
    'mints' as table_name,
    count() as row_count,
    max(block_number) as latest_block
FROM spl2.mints
"
```

### Common Issues

**Database Connection Issues:**
- Ensure ClickHouse is running: `docker-compose ps`
- Check logs: `make db-logs`
- Verify port availability: `netstat -ln | grep 8123`

**No Data Flowing:**
- Verify Substreams endpoint connectivity
- Check API token configuration
- Ensure correct token address in configuration
- Review substreams-sink-sql logs

**Performance Issues:**
- Monitor ClickHouse resource usage
- Consider adjusting batch sizes in sink configuration
- Review materialized view refresh intervals

## 🏗 Architecture

```
Solana Blockchain
       ↓
   Substreams
   (Rust WASM)
       ↓
substreams-sink-sql
       ↓
   ClickHouse
   (Tables + Views)
       ↓
   Analytics & Queries
```

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Projects

- [Substreams](https://github.com/streamingfast/substreams) - The core streaming engine
- [substreams-sink-sql](https://github.com/streamingfast/substreams-sink-sql) - SQL database sink
- [Firehose](https://github.com/streamingfast/firehose) - Blockchain data extraction
- [StreamingFast](https://streamingfast.io/) - Blockchain infrastructure platform

## 📞 Support

- [StreamingFast Discord](https://discord.gg/streamingfast)
- [Documentation](https://substreams.streamingfast.io/)
- [Sink SQL GitHub Issues](https://github.com/streamingfast/substreams-sink-sql/issues)
