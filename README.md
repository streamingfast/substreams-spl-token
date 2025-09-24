# Solana SPL Token

This Substreams module extracts SPL token instructions from Solana transactions, resolving account ownership through the SPL Initialized Account Foundational Store to provide complete transfer information.

If you are a Substreams developer, jump to [Using SPL Token Module](#using-spl-token-module) for details about how to consume SPL token instruction data with resolved ownership information.

## Using SPL Token Module

This module processes SPL token instructions and enriches them with account ownership data to create complete transfer records. The ownership resolution is critical since SPL transfer instructions only contain account addresses, not the actual wallet owners.

> [!NOTE]
> Ensure your Cargo `substreams` and `substreams-solana` dependencies are up to date `cargo update substreams substreams-solana`.

First, add the SPL Token import to your `substreams.yaml` manifest:

```yaml
...

imports:
  ...
  spl_token: solana-spl-token@v0.1.3

modules:
  ...

  - name: <your-module>
    kind: map
    inputs:
      ...
      - map: spl_token:map_spl_instructions
    output:
      ...
```

Then in your code, use the module output:

```rust
...
use sf::solana::spl::v1::type::SplInstructions;

#[substreams::handlers::map]
fn process_spl_data(spl_instructions: SplInstructions) -> Result<YourOutputType, Error> {
    ...

    // Process transfers with resolved ownership
    for transfer in spl_instructions.transfers {
        substreams::log::info!("Transfer from {} to {}",
            Address(&transfer.from_owner).to_string(),
            Address(&transfer.to_owner).to_string()
        );
    }

    // Process mints
    for mint in spl_instructions.mints {
        substreams::log::info!("Mint to owner {}",
            Address(&mint.to_owner).to_string()
        );
    }

    // Process burns
    for burn in spl_instructions.burns {
        substreams::log::info!("Burn from owner {}",
            Address(&burn.from_owner).to_string()
        );
    }
}
```

See https://github.com/streamingfast/substreams-foundational-modules/tree/main/solana/spl-initialized-account for the foundational store that resolves account ownership.

## Data Source

The module processes SPL token instructions and extracts information from these instruction types:
- `Transfer` - Token transfers between accounts (with resolved sender/receiver ownership)
- `Mint` - New token creation (with destination owner)
- `Burn` - Token destruction (with source owner)
- `InitializeAccount*` - Account initialization events

For each instruction, it resolves account ownership using the SPL Initialized Account Foundational Store to provide:
- **Account address** - The token account involved in the operation
- **Owner** - The actual wallet/program that controls the account
- **Amount** - The token amount for transfers, mints, and burns
- **Mint** - The SPL token mint address

### Configuration

Configure the module to track a specific SPL token by editing the parameters in `substreams.yaml`:

```yaml
params:
  map_spl_instructions: "spl_token_address=YOUR_TOKEN_ADDRESS|spl_token_decimal=DECIMALS"
  solana_common:transactions_by_programid_and_account_without_votes: "program:TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA && account:YOUR_TOKEN_ADDRESS"
```

The module supports both SPL Token programs:
- `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` (original)
- `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` (Token-2022)

## Quick Start with ClickHouse

This project includes a complete data pipeline from Solana blockchain to ClickHouse database with pre-built materialized views for analytics.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- [Rust](https://rustup.rs/) (for building the Substreams module)
- [Substreams CLI](https://substreams.streamingfast.io/getting-started/installing-the-cli)
- [substreams-sink-sql](https://github.com/streamingfast/substreams-sink-sql) (for ClickHouse integration)

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

## Database Schema

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

## Example Queries

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

## Development Commands

```bash
# Database management
make db-up          # Start ClickHouse
make db-down        # Stop ClickHouse
make db-setup       # Initialize database
make db-shell       # Open database shell
make db-reset       # Reset database (removes all data)
```

## Installing substreams-sink-sql

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
