use crate::{instruction, InitTokenParams, ID};
use anchor_lang::{prelude::*, InstructionData};
use solana_program::instruction::Instruction;

pub fn initialize(state: Pubkey, signer: Pubkey) -> Instruction {
    let data = instruction::Initialize {};
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(signer, true),
            AccountMeta::new(solana_program::system_program::ID, false),
        ],
    )
}

pub fn create_x404(
    name: String,
    symbol: String,
    redeem_max_deadline: u64,
    redeem_fee: u64,
    decimals: u8,
    uri: String,
    hub: Pubkey,
    source: Pubkey,
    state: Pubkey,
    metadata: Pubkey,
    nft_mint: Pubkey,
    nft_token: Pubkey,
    fungible_mint: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::CreateX404 {
        params: InitTokenParams {
            name,
            symbol,
            redeem_max_deadline,
            redeem_fee,
            decimals,
            uri,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new_readonly(hub, false),
            AccountMeta::new_readonly(source, false),
            AccountMeta::new(state, false),
            AccountMeta::new(metadata, false),
            AccountMeta::new(nft_mint, false),
            AccountMeta::new(nft_token, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(signer, true),
            // rendt
            AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
            // token
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
            // system
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            // metadata
            AccountMeta::new_readonly(anchor_spl::metadata::ID, false),
            // ata
            AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
        ],
    )
}
