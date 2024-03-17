use anchor_lang::prelude::*;
use solana_program::pubkey::{self, Pubkey};
use std::collections::HashMap;

// PDA of NFT tokens
#[account]
pub struct DepositsAccount {
    // name for NFT collections
    pub name: String,
    // NFT Account and redeem deadline
    pub tokens: HashMap<Pubkey, u64>,
    // mint account for fungible token
    pub mint: Pubkey,
    // balance of fungible token
    pub balance: HashMap<Pubkey, u64>,
    // redeem fee
    pub redeem_fee: u64,
    // maximum redeem deadline
    pub max_redeem_deadeline: u64,
}

#[account]
pub struct GlobalStateAccount {
    // contract manager
    pub owner: String,
    // token symbol
    pub symbol: String,
    // boolean for emergency stop
    pub emergency_stop: bool,
    // map between collection and Deposit Account
    pub approved: HashMap<Pubkey, Pubkey>,
}
