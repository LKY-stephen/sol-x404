use std::collections::HashMap;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint as SPLMint, Token, TokenAccount as SPLTokenAccount},
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

pub const BANK_SIZE: usize = 8 + 8 + 32 + 32;

// validate incoming accounts here
#[account]
pub struct X404Hub {
    // emergent close
    pub emergency_close: bool,
    // manager who can create new X404 account or update underlying X404
    pub manager: Pubkey,
}

#[account]
pub struct NFTBank {
    // emergent close
    pub id: Pubkey,
    // deadline for redeem
    pub redeem_deadline: u64,
    // owner of this NFT
    pub owner: Pubkey,
}

#[account]
pub struct OwnerStore {
    // emergent close
    pub store: Vec<u8>,
}

impl OwnerStore {
    pub fn get_map(&self) -> HashMap<Pubkey, Vec<Pubkey>> {
        HashMap::<Pubkey, Vec<Pubkey>>::try_from_slice(self.store.as_slice()).unwrap()
    }

    pub fn update_map(&mut self, map: &HashMap<Pubkey, Vec<Pubkey>>) {
        self.store = map.try_to_vec().unwrap();
    }
}

#[account]
pub struct X404State {
    // liquidity source of X404
    pub source: Pubkey,
    // max waiting time for priority redeem
    pub redeem_max_deadline: u64,
    // redeem fee for x404
    pub redeem_fee: u64,
    // Hub Pubkey for this X404
    pub owner: Pubkey,
    // decimal for fungible token
    pub decimal: u8,
    // fungible mint for this X404
    pub fungible_mint: Pubkey,
    // hook for the fungible token
    pub fungible_hook: Pubkey,
    // nft mint for this X404
    pub collection_mint: Pubkey,
    //supply of nft
    pub nft_supply: u64,
    //supply of nft
    pub nft_in_use: u64,
    // fungible token per deposit/redeem
    pub fungible_supply: u64,
}
#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 1,
        seeds = [b"hub".as_ref()],
        bump,
    )]
    pub state: Account<'info, X404Hub>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    params: InitTokenParams
)]
pub struct CreateX404<'info> {
    #[account(
        mut,
        seeds = [b"hub".as_ref()],
        bump,
    )]
    pub hub: Box<Account<'info, X404Hub>>,
    // CHECK: New source for minting tokens
    pub source: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 8 + 8 + 32 + 1 + 32 + 32 + 32 + 8 + 8 + 8,
        seeds = [b"state".as_ref(), source.to_account_info().key.as_ref()],
        bump,
    )]
    pub state: Box<Account<'info, X404State>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"owner_store".as_ref(), state.to_account_info().key.as_ref()],
        bump,
        space = 8 + 4
    )]
    pub owner_store: Box<Account<'info, OwnerStore>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"collection_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::token_program = token_program,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,
    // CHECK: there is no decent way to init token2022 with hook,
    // manually initiate it for now
    #[account(
        seeds = [b"fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub fungible_mint: UncheckedAccount<'info>,
    #[account(mut, signer)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    params: InitCollectionParams
)]
pub struct MintCollection<'info> {
    #[account(mut,
        seeds = [b"state".as_ref(), params.source.as_ref()],
        bump)
    ]
    pub state: Box<Account<'info, X404State>>,
    #[account(mut,
        seeds = [b"collection_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = collection_mint,
        associated_token::authority = collection_mint,
        associated_token::token_program = token_program,
    )]
    pub collection_token: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, signer)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params:DepositParams)]
pub struct DepositSPLNFT<'info> {
    #[account(
        mut,
        seeds = [b"state".as_ref(), params.source.as_ref()],
        bump,)]
    pub state: Box<Account<'info, X404State>>,
    #[account(
        mut,
        seeds = [b"owner_store".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub owner_store: Box<Account<'info, OwnerStore>>,
    pub deposit_mint: Box<Account<'info, SPLMint>>,
    #[account(mut,
        associated_token::mint = deposit_mint,
        associated_token::authority = signer,
        associated_token::token_program = deposit_program,
    )]
    pub deposit_holder: Box<Account<'info, SPLTokenAccount>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = deposit_mint,
        associated_token::authority = state,
        associated_token::token_program = deposit_program,
    )]
    pub deposit_receiver: Box<Account<'info, SPLTokenAccount>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"nft_bank".as_ref(), deposit_mint.to_account_info().key.as_ref()],
        bump,
        space = BANK_SIZE,
    )]
    pub nft_bank: Box<Account<'info, NFTBank>>,
    // We do not init this mint in all cases, so we need to init is in program
    #[account(
        seeds = [b"nft_mint".as_ref(),state.to_account_info().key.as_ref(), state.nft_supply.to_le_bytes().as_ref()],
        bump,
    )]
    pub nft_mint: AccountInfo<'info>,
    // CHECKED: This is the associated token account for the NFT mint with authroity state
    pub nft_token: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub fungible_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = fungible_mint,
        associated_token::authority = signer,
    )]
    pub fungible_token: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub deposit_program: Program<'info, Token>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params:BindParams)]
pub struct BindNFT<'info> {
    #[account(mut,
        seeds = [b"state".as_ref(), params.source.as_ref()],
        bump,)]
    pub state: Box<Account<'info, X404State>>,
    // CHECKED: account to store the NFT owned by the owner, should be owned by
    // the program. Ideally should use init_if_needed, but because the store is
    // dynamic size, haven't figured out how to init through constraint.
    #[account(
        mut,
        seeds = [b"owner_store".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub owner_store: Box<Account<'info, OwnerStore>>,
    #[account(
        mut,
        seeds = [b"nft_mint".as_ref(),state.to_account_info().key.as_ref(), params.number.to_le_bytes().as_ref()],
        bump,
    )]
    pub bind_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = bind_mint,
        associated_token::authority = state,
    )]
    pub bind_holder: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = bind_mint,
        associated_token::authority = signer,
    )]
    pub bind_token: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
        seeds = [b"fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub fungible_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = fungible_mint,
        associated_token::authority = signer,
    )]
    pub fungible_token: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params:UnbindParams)]
pub struct UnbindNFT<'info> {
    #[account(mut,
        seeds = [b"state".as_ref(), params.source.as_ref()],
        bump,)]
    pub state: Box<Account<'info, X404State>>,
    // CHECKED: account to store the NFT owned by the owner, should be owned by
    // the program. Ideally should use init_if_needed, but because the store is
    // dynamic size, haven't figured out how to init through constraint.
    #[account(
        mut,
        seeds = [b"owner_store".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub owner_store: Box<Account<'info, OwnerStore>>,
    #[account(
        mut,
        seeds = [b"nft_mint".as_ref(),state.to_account_info().key.as_ref(), params.number.to_le_bytes().as_ref()],
        bump,
    )]
    pub bind_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = bind_mint,
        associated_token::authority = state,
    )]
    pub bind_token: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = bind_mint,
        associated_token::authority = signer,
    )]
    pub bind_receiver: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
        seeds = [b"fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub fungible_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = fungible_mint,
        associated_token::authority = signer,
    )]
    pub fungible_token: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params:RebalanceParams)]
pub struct Rebalance<'info> {
    pub state: Box<Account<'info, X404State>>,
    // CHECKED: account to store the NFT owned by the owner, should be owned by
    // the program. Ideally should use init_if_needed, but because the store is
    // dynamic size, haven't figured out how to init through constraint.
    #[account(
        mut,
        seeds = [b"owner_store".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub owner_store: Box<Account<'info, OwnerStore>>,
    #[account(
        mut,
        seeds = [b"fungible_mint".as_ref(),state.to_account_info().key.as_ref()],
        bump,
    )]
    pub fungible_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = fungible_mint,
        associated_token::authority = params.sender,
    )]
    pub sender_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = fungible_mint,
        associated_token::authority = params.receiver,
    )]
    pub receiver_account: InterfaceAccount<'info, TokenAccount>,
    #[account(signer)]
    pub hooker: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(params:RedeemParams)]
pub struct RedeemSPLNFT<'info> {
    #[account(mut,
        seeds = [b"state".as_ref(), params.source.as_ref()],
        bump)]
    pub state: Box<Account<'info, X404State>>,
    #[account(
        mut,
        seeds = [b"owner_store".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub owner_store: Box<Account<'info, OwnerStore>>,
    #[account(mut)]
    pub withdraw_mint: Box<Account<'info, SPLMint>>,
    #[account(mut,
        associated_token::mint = withdraw_mint,
        associated_token::authority = state,
        associated_token::token_program = withdrawal_program,)]
    pub withdraw_holder: Box<Account<'info, SPLTokenAccount>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = withdraw_mint,
        associated_token::authority = signer,
        associated_token::token_program = withdrawal_program,
    )]
    pub withdrawal_receiver: Box<Account<'info, SPLTokenAccount>>,
    #[account(mut,
        seeds = [b"nft_bank".as_ref(), withdraw_mint.to_account_info().key.as_ref()],
        bump,
    )]
    pub nft_bank: Box<Account<'info, NFTBank>>,
    #[account(mut,
        seeds = [b"fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub fungible_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut,
        associated_token::mint = fungible_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub fungible_token: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut,
        associated_token::mint = fungible_mint,
        associated_token::authority = nft_bank.owner,
        associated_token::token_program = token_program,
    )]
    pub original_owner: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub withdrawal_program: Program<'info, Token>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    // max waiting time for priority redeem
    pub redeem_max_deadline: u64,
    // redeem fee for x404
    pub redeem_fee: u64,
    // uri for fungible token
    pub decimals: u8,
    // deposit supply
    pub fungible_supply: u64,
    // hook address
    pub hook_extra_account: Pubkey,
    // hook program id
    pub hook_program: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitCollectionParams {
    // name for NFT token
    pub name: String,
    // symbol for NFT token
    pub symbol: String,
    // uri for fungible token
    pub uri: String,
    // pubkey of source
    pub source: Pubkey,
}

// impl InitCollectionParams {
//     pub fn data(&self) -> DataV2 {
//         DataV2 {
//             name: self.name.clone(),
//             symbol: self.symbol.clone(),
//             uri: self.uri.clone(),
//             seller_fee_basis_points: 5,
//             creators: None,
//             collection: None,
//             uses: None,
//         }
//     }
// }

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct DepositParams {
    //pubkey of source
    pub source: Pubkey,
    // dead line for redeem
    pub redeem_deadline: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BindParams {
    //pubkey of source
    pub source: Pubkey,
    // Token to bind
    pub number: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct UnbindParams {
    //pubkey of source
    pub source: Pubkey,
    // Token to bind
    pub number: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RedeemParams {
    // pubkey of source
    pub source: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RebalanceParams {
    // pubkey of sender
    pub sender: Pubkey,
    // pubkey of receiver
    pub receiver: Pubkey,
    // amount to rebalance
    pub amount: u64,
}
