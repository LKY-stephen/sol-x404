use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    // Need to transfer to token_2022::Token2022 once supported,
    // https://github.com/coral-xyz/anchor/issues/2801
    token::{Mint, Token, TokenAccount},
};
use solana_program::pubkey::Pubkey;
// validate incoming accounts here
#[account]
#[derive(Default)]
pub struct X404Hub {
    // emergent close
    pub embergency_close: bool,
    // manager who can create new X404 account or update underlying X404
    pub manager: Pubkey,
}

#[account]
#[derive(Default)]
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
    pub hub: Account<'info, X404Hub>,
    // CHECK: mint account
    pub source: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 8 + 8 + 32 + 2,
        seeds = [b"404_contract".as_ref(), source.to_account_info().key.as_ref()],
        bump,
    )]
    pub state: Box<Account<'info, X404State>>,
    // CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"404_contract_nft_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
    )]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_mint,
    )]
    pub nft_token: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"404_contract_fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
        mint::decimals = params.decimals,
        mint::authority = fungible_mint,
    )]
    pub fungible_mint: Box<Account<'info, Mint>>,
    #[account(mut, signer)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    // name for NFT token
    pub name: String,
    // symbol for NFT token
    pub symbol: String,
    // max waiting time for priority redeem
    pub redeem_max_deadline: u64,
    // redeem fee for x404
    pub redeem_fee: u64,
    // decimal for fungible token
    pub decimals: u8,
    // uri for fungible token
    pub uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MintToken {
    // dead line for redeem
    pub redeem_deadline: u64,
}
