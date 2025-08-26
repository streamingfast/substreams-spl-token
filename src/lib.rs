mod constants;
mod pb;

use substreams_solana::pb::sf::solana::r#type::v1::Block;
use crate::pb::sf::solana::spl::v1::r#type::instruction::Item;
use crate::pb::sf::solana::spl::v1::r#type::{Burn, InitializedAccount, Instruction, Mint, SplInstructions, Transfer};
use pb::sol::transactions::v1::Transactions as solTransactions;
use std::collections::HashMap;
use std::ops::Div;
use substreams::errors::Error;
use prost::Message;

use substreams::pb::foundational_store::ResponseCode;
use pb::sf::solana::spl::foundational::v1::AccountOwner;
use substreams::store::FoundationalStore;

use substreams_solana::block_view::InstructionView;
use substreams_solana::pb::sf::solana::r#type::v1::{ConfirmedTransaction, TokenBalance, TransactionStatusMeta};
use substreams_solana::Address;
use substreams_solana_program_instructions::token_instruction_2022::TokenInstruction;

pub const SOLANA_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

struct OutputInstructions {
    transaction_hash: String,
    ordinal: i64,
    instructions: Vec<Instruction>,
}

impl OutputInstructions {
    pub fn new(transaction_hash: String) -> Self {
        Self {
            transaction_hash,
            ordinal: 0,
            instructions: vec![],
        }
    }

    pub fn add(&mut self, item: Item) {
        self.instructions.push(Instruction {
            transaction_hash: self.transaction_hash.to_string(),
            instruction_id: self.transaction_hash.to_string() + "-" + &self.ordinal.to_string(),
            item: Some(item),
        });

        self.ordinal += 1;
    }
}

#[substreams::handlers::map]
fn map_spl_instructions(params: String, transactions: solTransactions, block: Block, foundational_store: FoundationalStore) -> Result<SplInstructions, Error> {
    let mut instructions: Vec<Instruction> = vec![];

    let mut spl_token_address = String::new();
    let mut spl_token_decimal = 0;

    for param in params.split('|') {
        let parts: Vec<&str> = param.split('=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "spl_token_address" => spl_token_address = parts[1].to_string(),
                "spl_token_decimal" => spl_token_decimal = parts[1].parse().unwrap(),
                _ => {}
            }
        }
    }

    for confirmed_trx in transactions_owned(transactions) {
        let hash = bs58::encode(confirmed_trx.hash()).into_string();

        let mut output_instructions = OutputInstructions::new(hash.clone());

        for instruction in confirmed_trx.compiled_instructions() {
            process_instruction(
                &mut output_instructions,
                &spl_token_address,
                spl_token_decimal,
                &instruction,
            );
        }

        instructions.extend(output_instructions.instructions);
    }

    let mut accounts_to_lookup = Vec::<Vec<u8>>::new();

    for instruction in &instructions {
        if let Some(ref item) = instruction.item {
            match item {
                Item::Transfer(transfer) => {
                    if let Ok(from_bytes) = bs58::decode(&transfer.from).into_vec() {
                        accounts_to_lookup.push(from_bytes);
                    }
                    if let Ok(to_bytes) = bs58::decode(&transfer.to).into_vec() {
                        accounts_to_lookup.push(to_bytes);
                    }
                }
                Item::Mint(mint) => {
                    if let Ok(to_bytes) = bs58::decode(&mint.to).into_vec() {
                        accounts_to_lookup.push(to_bytes);
                    }
                }
                Item::Burn(burn) => {
                    if let Ok(from_bytes) = bs58::decode(&burn.from).into_vec() {
                        accounts_to_lookup.push(from_bytes);
                    }
                }
                _ => {}
            }
        }
    }

    // Remove duplicates, better way ?
    accounts_to_lookup.sort();
    accounts_to_lookup.dedup();

    let block_hash_bytes = bs58::decode(&block.blockhash).into_vec().unwrap_or_default();
    let owners = get_account_owners_all(&foundational_store, &accounts_to_lookup, &block_hash_bytes, block.slot);

    for instruction in &mut instructions {
        if let Some(ref mut item) = instruction.item {
            match item {
                Item::Transfer(ref mut transfer) => {
                    if let Some(from_owner) = owners.get(&transfer.from) {
                        transfer.from_owner = from_owner.clone();
                    }
                    if let Some(to_owner) = owners.get(&transfer.to) {
                        transfer.to_owner = to_owner.clone();
                    }
                }
                Item::Mint(ref mut mint) => {
                    if let Some(to_owner) = owners.get(&mint.to) {
                        mint.to_owner = to_owner.clone();
                    }
                }
                Item::Burn(ref mut burn) => {
                    if let Some(from_owner) = owners.get(&burn.from) {
                        burn.from_owner = from_owner.clone();
                    }
                }
                _ => {}
            }
        }
    }

    Ok(SplInstructions { instructions })
}

fn get_account_owners_all(
    foundational_store: &FoundationalStore,
    accounts: &[Vec<u8>],
    block_hash: &[u8],
    block_slot: u64,
) -> HashMap<String, String> {
    let mut results = HashMap::with_capacity(accounts.len());
    if accounts.is_empty() {
        return results;
    }

    let resp = foundational_store.get_all(block_hash, block_slot, accounts);

    for entry in resp.entries {
        let Some(get_response) = entry.response else { continue; };
        if get_response.response != ResponseCode::Found as i32 { continue; }
        let Some(value) = get_response.value else { continue; };
        let Ok(account_owner) = AccountOwner::decode(value.value.as_slice()) else { continue; };

        let owner_b58 = bs58::encode(&account_owner.owner).into_string();
        let account_b58 = bs58::encode(&entry.key).into_string();
        results.insert(account_b58, owner_b58);
    }

    results
}

/// Iterates over successful transactions in given block and take ownership.
fn transactions_owned(transactions: solTransactions) -> impl Iterator<Item = ConfirmedTransaction> {
    transactions.transactions.into_iter().filter(|trx| -> bool {
        if let Some(meta) = &trx.meta {
            return meta.err.is_none();
        }
        false
    })
}

fn process_instruction(
    output: &mut OutputInstructions,
    spl_token_address: &str,
    spl_token_decimal: i32,
    compile_instruction: &InstructionView,
) {
    let trx_hash = &bs58::encode(compile_instruction.transaction().hash()).into_string();
    match compile_instruction.program_id().to_string().as_ref() {
        SOLANA_TOKEN_PROGRAM => {
            match process_token_instruction(
                output,
                spl_token_address,
                spl_token_decimal,
                compile_instruction,
                compile_instruction.meta(),
            ) {
                Err(err) => {
                    panic!("trx_hash {} process token instructions: {}", trx_hash, err);
                }
                _ => {}
            }
        }
        _ => {
            process_inner_instruction(
                compile_instruction,
                spl_token_address,
                spl_token_decimal,
                trx_hash,
                compile_instruction.meta(),
                output,
            );
        }
    }
}

fn process_inner_instruction(
    compile_instruction: &InstructionView,
    spl_token_address: &str,
    spl_token_decimal: i32,
    trx_hash: &String,
    meta: &TransactionStatusMeta,
    output: &mut OutputInstructions,
) {
    for inner in compile_instruction.inner_instructions() {
        match inner.program_id().to_string().as_ref() {
            SOLANA_TOKEN_PROGRAM => {
                match process_token_instruction(output, spl_token_address, spl_token_decimal, &inner, meta) {
                    Err(err) => {
                        panic!("trx_hash {} process token instructions {}", trx_hash, err);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn process_token_instruction(
    output: &mut OutputInstructions,
    spl_token_address: &str,
    spl_token_decimal: i32,
    instruction: &InstructionView,
    meta: &TransactionStatusMeta,
) -> Result<(), Error> {
    match TokenInstruction::unpack(&instruction.data()) {
        Err(err) => {
            return Err(anyhow::anyhow!("unpacking token instruction: {}", err));
        }
        Ok(token_instruction) => match token_instruction {
            #[allow(deprecated)]
            TokenInstruction::Transfer { amount: amt } => {
                let authority = &instruction.accounts()[2];

                // let authority = &accounts[inst_accounts[2] as usize];
                if is_token_transfer(spl_token_address, &meta.pre_token_balances, authority) {
                    let source = &instruction.accounts()[0];
                    // let source = &accounts[inst_accounts[0] as usize];
                    let destination = &instruction.accounts()[1];
                    // let destination = &accounts[inst_accounts[1] as usize];

                    output.add(Item::Transfer(Transfer {
                        from: source.to_string(),
                        to: destination.to_string(),
                        amount: amount_to_decimals(amt as f64, spl_token_decimal as f64),
                        from_owner: String::new(),
                        to_owner: String::new(),
                    }));
                }
            }

            TokenInstruction::TransferChecked { amount: amt, .. } => {
                let mint = &instruction.accounts()[1];
                // let mint = &accounts[inst_accounts[1] as usize];
                if mint.to_string() == spl_token_address {
                    let source = &instruction.accounts()[0];
                    // let source = &accounts[inst_accounts[0] as usize];
                    let destination = &instruction.accounts()[2];
                    // let destination = &accounts[inst_accounts[2] as usize];

                    output.add(Item::Transfer(Transfer {
                        from: source.to_string(),
                        to: destination.to_string(),
                        amount: amount_to_decimals(amt as f64, spl_token_decimal as f64),
                        from_owner: String::new(),
                        to_owner: String::new(),
                    }));
                }
            }

            TokenInstruction::MintTo { amount: amt } | TokenInstruction::MintToChecked { amount: amt, .. } => {
                let mint = &instruction.accounts()[0];
                if mint.to_string().as_str() != spl_token_address {
                    return Ok(());
                }

                let account_to = &instruction.accounts()[1];
                output.add(Item::Mint(Mint {
                    to: account_to.to_string(),
                    amount: amount_to_decimals(amt as f64, spl_token_decimal as f64),
                    to_owner: String::new(),
                }));
            }

            TokenInstruction::Burn { amount: amt } | TokenInstruction::BurnChecked { amount: amt, .. } => {
                let mint = &instruction.accounts()[1];
                if mint.to_string().as_str() != spl_token_address {
                    return Ok(());
                }

                let account_from = &instruction.accounts()[0];
                output.add(Item::Burn(Burn {
                    from: account_from.to_string(),
                    amount: amount_to_decimals(amt as f64, spl_token_decimal as f64),
                    from_owner: String::new(),
                }));
            }
            TokenInstruction::InitializeAccount {} => {
                let mint = &instruction.accounts()[1];
                if mint.to_string().as_str() != spl_token_address {
                    return Ok(());
                }

                let account = &instruction.accounts()[0];
                let owner = &instruction.accounts()[2];

                output.add(Item::InitializedAccount(InitializedAccount {
                    account: account.to_string(),
                    mint: mint.to_string(),
                    owner: owner.to_string(),
                }));
            }
            TokenInstruction::InitializeAccount2 { owner: ow } | TokenInstruction::InitializeAccount3 { owner: ow } => {
                let mint = &instruction.accounts()[1];
                if mint.to_string().as_str() != spl_token_address {
                    return Ok(());
                }

                let account = &instruction.accounts()[0];

                output.add(Item::InitializedAccount(InitializedAccount {
                    account: account.to_string(),
                    mint: mint.to_string(),
                    owner: bs58::encode(ow).into_string(),
                }));
            }
            _ => {}
        },
    }

    Ok(())
}

fn amount_to_decimals(amount: f64, decimal: f64) -> f64 {
    let base: f64 = 10.0;
    amount.div(&(base.powf(decimal)))
}

pub fn is_token_transfer(spl_token_address: &str, pre_token_balances: &Vec<TokenBalance>, account: &Address) -> bool {
    for token_balance in pre_token_balances.iter() {
        if token_balance.owner.eq(account.to_string().as_str()) && token_balance.mint.eq(spl_token_address) {
            return true;
        }
    }
    false
}

