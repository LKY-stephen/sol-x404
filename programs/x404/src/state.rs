use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
// validate incoming accounts here
#[account]
pub struct X404Hub {
    // emergent close
    pub embergency_close: bool,
    // manager who can create new X404 account or update underlying X404
    pub manager: Pubkey,
}

#[account]
pub struct NFTBank {
    // emergent close
    pub id: Pubkey,
    // deadline for redeem
    pub redeem_deadline: Pubkey,
    // owner of this NFT
    pub owner: Pubkey,
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
    // nft mint for this X404
    pub nft_mint: Pubkey,
    //supply of nft
    pub nft_supply: u64,
    //minted nft
    pub nft_in_use: u64,
    //in store nft, at most 1024
    pub nft_instore: [u8; 128],
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
        space = 8 + 32 + 8 + 8 + 32 + 1 + 32 + 32 + 8 + 8 + 128,
        seeds = [b"state".as_ref(), source.to_account_info().key.as_ref()],
        bump,
    )]
    pub state: Box<Account<'info, X404State>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"404_contract_nft_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
        mint::token_program = token_program,
    )]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"404_contract_fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
        mint::decimals = params.decimals,
        mint::authority = fungible_mint,
        mint::token_program = token_program,
    )]
    pub fungible_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, signer)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
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
        seeds = [b"404_contract_nft_mint".as_ref(), state.to_account_info().key.as_ref()],
        bump,
    )]
    pub nft_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_mint,
        associated_token::token_program = token_program,
    )]
    pub nft_token: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, signer)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// #[derive(Accounts)]
// #[instruction(params:DepositParams)]
// pub struct DepositToken<'info> {
//     #[account(mut)]
//     pub state: Account<'info, X404State>,
//     #[account(
//         init,
//         payer = signer,
//         space = 8 + 32 + 32 + 32,
//         seeds = [b"nft_bank".as_ref(), params.id.as_ref()],
//         bump,
//     )]
//     pub nft_bank: Account<'info, NFTBank>,
//     #[account(
//         init,
//         payer = signer, // TODO, Should be the contract to pay for the rent
//         seeds = [b"404_contract_nft_mint".as_ref()],
//         bump,
//         mint::decimals = 0,
//         mint::authority = state.key(),
//         mint::freeze_authority = signer.key(),
//     )]
//     pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
//     #[account(
//         init_if_needed,
//         payer = signer,
//         associated_token::mint = nft_mint,
//         associated_token::authority = nft_mint,
//     )]
//     pub nft_token: Box<InterfaceAccount<'info, TokenAccount>>,
//     #[account(mut,
//         seeds = [b"404_contract_fungible_mint".as_ref(), state.to_account_info().key.as_ref()],
//         bump,
//     )]
//     pub fungible_mint: Box<InterfaceAccount<'info, Mint>>,
//     #[account(
//         init_if_needed,
//         payer = signer,
//         associated_token::mint = fungible_mint,
//         associated_token::authority = fungible_mint,
//     )]
//     pub fungible_token: Box<InterfaceAccount<'info, TokenAccount>>,
//     #[account(mut)]
//     pub signer: Signer<'info>,
//     pub rent: Sysvar<'info, Rent>,
//     pub token_program: Program<'info, Token2022>,
//     pub system_program: Program<'info, System>,
//     pub token_metadata_program: Program<'info, Metadata>,
//     pub associated_token_program: Program<'info, AssociatedToken>,
// }

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    // max waiting time for priority redeem
    pub redeem_max_deadline: u64,
    // redeem fee for x404
    pub redeem_fee: u64,
    // uri for fungible token
    pub decimals: u8,
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
    // id of deposied NFT
    pub id: Pubkey,
    // dead line for redeem
    pub redeem_deadline: u64,
}
