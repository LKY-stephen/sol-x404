use crate::{instruction, InitCollectionParams, InitTokenParams, ID};
use anchor_lang::{prelude::*, solana_program::sysvar::rent, system_program, InstructionData};
use anchor_spl::{associated_token, token_2022};
use solana_program::instruction::Instruction;

pub fn initialize(state: Pubkey, signer: Pubkey) -> Instruction {
    let data = instruction::Initialize {};
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(signer, true),
            AccountMeta::new(system_program::ID, false),
        ],
    )
}

pub fn create_x404(
    redeem_max_deadline: u64,
    redeem_fee: u64,
    decimals: u8,
    hub: Pubkey,
    source: Pubkey,
    state: Pubkey,
    nft_mint: Pubkey,
    fungible_mint: Pubkey,
    signer: Pubkey,
    fungible_supply: u64,
) -> Instruction {
    let data = instruction::CreateX404 {
        params: InitTokenParams {
            redeem_max_deadline,
            redeem_fee,
            decimals,
            fungible_supply,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(hub, false),
            AccountMeta::new(source, false),
            AccountMeta::new(state, false),
            AccountMeta::new(nft_mint, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(signer, true),
            // rendt
            AccountMeta::new_readonly(rent::ID, false),
            // token
            AccountMeta::new_readonly(anchor_spl::token_2022::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn create_collection(
    name: String,
    symbol: String,
    uri: String,
    source: Pubkey,
    state: Pubkey,
    nft_mint: Pubkey,
    nft_token: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::MintCollection {
        _params: InitCollectionParams {
            name,
            symbol,
            uri,
            source,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(nft_mint, false),
            AccountMeta::new(nft_token, false),
            AccountMeta::new(signer, true),
            // rent
            AccountMeta::new_readonly(rent::ID, false),
            // token
            AccountMeta::new_readonly(token_2022::ID, false),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}
