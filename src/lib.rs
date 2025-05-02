mod constants;
mod pb;

use pb::sol::transactions::v1::Transactions as solTransactions;
use std::ops::Div;
use substreams::errors::Error;
use substreams_solana::block_view::InstructionView;
use substreams_solana::pb::sf::solana::r#type::v1::{ConfirmedTransaction, TokenBalance, TransactionStatusMeta};
use substreams_solana::Address;
use substreams_solana_program_instructions::token_instruction_2022::TokenInstruction;
use crate::pb::sf::solana::spl::v1::r#type::{Burn, InitializedAccount, Instruction, Mint, SplInstructions, Transfer};
use crate::pb::sf::solana::spl::v1::r#type::instruction::Item;

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
            item: Some(item)
        });

        self.ordinal += 1;
    }
}

#[substreams::handlers::map]
pub fn map_outputs(transactions: solTransactions) -> Result<SplInstructions, Error> {
    let mut instructions: Vec<Instruction> = vec![];

    for confirmed_trx in transactions_owned(transactions) {
        let hash = bs58::encode(confirmed_trx.hash()).into_string();

        let mut output_instructions = OutputInstructions::new(hash.clone());

        for instruction in confirmed_trx.compiled_instructions() {
            process_instruction(&mut output_instructions, &instruction);
        }
        
        
    }
    Ok(SplInstructions { instructions: instructions })

}

/// Iterates over successful transactions in given block and take ownership.
pub fn transactions_owned(transactions: solTransactions) -> impl Iterator<Item = ConfirmedTransaction> {
    transactions.transactions.into_iter().filter(|trx| -> bool {
        if let Some(meta) = &trx.meta {
            return meta.err.is_none();
        }
        false
    })
}

pub fn process_instruction(output: &mut OutputInstructions, compile_instruction: &InstructionView) {
    let trx_hash = &bs58::encode(compile_instruction.transaction().hash()).into_string();
    match compile_instruction.program_id().to_string().as_ref() {
        constants::SOLANA_TOKEN_PROGRAM => {
            match process_token_instruction(output, compile_instruction, compile_instruction.meta()) {
                Err(err) => {
                    panic!("trx_hash {} process token instructions: {}", trx_hash, err);
                }
                _ => {}
            }
        }
        _ => {
            process_inner_instruction(compile_instruction, trx_hash, compile_instruction.meta(), output);
        }
    }
}


fn process_inner_instruction(
    compile_instruction: &InstructionView,
    trx_hash: &String,
    meta: &TransactionStatusMeta,
    output: &mut OutputInstructions,
) {
    for inner in compile_instruction.inner_instructions() {
        match inner.program_id().to_string().as_ref() {
            constants::SOLANA_TOKEN_PROGRAM => {
                match process_token_instruction(output, &inner, meta) {
                    Err(err) => {
                        panic!("trx_hash {} process token instructions {}", trx_hash, err);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }
}


fn process_token_instruction(
    output: &mut OutputInstructions,
    instruction: &InstructionView,
    meta: &TransactionStatusMeta,
) -> Result<(), Error> {
    match TokenInstruction::unpack(&instruction.data()) {
        Err(err) => {
            return Err(anyhow::anyhow!("unpacking token instruction: {}", err));
        }
        Ok(token_instruction) => match token_instruction {
            TokenInstruction::Transfer { amount: amt } => {
                let authority = &instruction.accounts()[2];

                // let authority = &accounts[inst_accounts[2] as usize];
                if is_honey_token_transfer(&meta.pre_token_balances, authority) {
                    let source = &instruction.accounts()[0];
                    // let source = &accounts[inst_accounts[0] as usize];
                    let destination = &instruction.accounts()[1];
                    // let destination = &accounts[inst_accounts[1] as usize];

                    output.add(Item::Transfer(Transfer {
                        from: source.to_string(),
                        to: destination.to_string(),
                        amount: amount_to_decimals(amt as f64, constants::HONEY_TOKEN_DECIMALS as f64),
                    }));
                }
            }

            TokenInstruction::TransferChecked { amount: amt, .. } => {
                let mint = &instruction.accounts()[1];
                // let mint = &accounts[inst_accounts[1] as usize];
                if mint.to_string() == constants::HONEY_CONTRACT_ADDRESS {
                    let source = &instruction.accounts()[0];
                    // let source = &accounts[inst_accounts[0] as usize];
                    let destination = &instruction.accounts()[2];
                    // let destination = &accounts[inst_accounts[2] as usize];

                    output.add(Item::Transfer(Transfer {
                        from: source.to_string(),
                        to: destination.to_string(),
                        amount: amount_to_decimals(amt as f64, constants::HONEY_TOKEN_DECIMALS as f64),
                    }));
                }
            }

            TokenInstruction::MintTo { amount: amt } | TokenInstruction::MintToChecked { amount: amt, .. } => {
                let mint = &instruction.accounts()[0];
                if mint.to_string().as_str() != constants::HONEY_CONTRACT_ADDRESS {
                    return Ok(());
                }

                let account_to = &instruction.accounts()[1];
                output.add(Item::Mint(Mint {
                    to: account_to.to_string(),
                    amount: amount_to_decimals(amt as f64, constants::HONEY_TOKEN_DECIMALS as f64),
                }));
            }

            TokenInstruction::Burn { amount: amt } | TokenInstruction::BurnChecked { amount: amt, .. } => {
                let mint = &instruction.accounts()[1];
                if mint.to_string().as_str() != constants::HONEY_CONTRACT_ADDRESS {
                    return Ok(());
                }

                let account_from = &instruction.accounts()[0];
                output.add(Item::Burn(Burn {
                    from: account_from.to_string(),
                    amount: amount_to_decimals(amt as f64, constants::HONEY_TOKEN_DECIMALS as f64),
                }));
            }
            TokenInstruction::InitializeAccount {} => {
                let mint = &instruction.accounts()[1];
                if mint.to_string().as_str() != constants::HONEY_CONTRACT_ADDRESS {
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
                if mint.to_string().as_str() != constants::HONEY_CONTRACT_ADDRESS {
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

pub fn is_honey_token_transfer(pre_token_balances: &Vec<TokenBalance>, account: &Address) -> bool {
    for token_balance in pre_token_balances.iter() {
        if token_balance.owner.eq(account.to_string().as_str())
            && token_balance.mint.eq(constants::HONEY_CONTRACT_ADDRESS)
        {
            return true;
        }
    }
     false
}
