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
