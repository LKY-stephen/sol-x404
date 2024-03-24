pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use state::*;

use anchor_spl::{metadata::create_metadata_accounts_v3, token::MintTo};
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod x404 {
    use anchor_spl::{
        metadata::{mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3},
        token::{
            mint_to,
            spl_token::instruction::{set_authority, AuthorityType},
        },
    };

    use super::*;
    use crate::error::SolX404Error;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let hub = &mut ctx.accounts.state;
        hub.manager = *ctx.accounts.signer.key;
        hub.embergency_close = false;
        msg!(
            "Initialized new hub: {} with owner {}!",
            hub.to_account_info().key,
            hub.manager
        );
        Ok(())
    }

    pub fn create_x404(ctx: Context<CreateX404>, params: InitTokenParams) -> Result<()> {
        let state = &mut ctx.accounts.state;
        msg!("check permission for create x404");
        if ctx.accounts.signer.key() != ctx.accounts.hub.manager {
            msg!("signer: {}", ctx.accounts.signer.key());
            msg!("hub manager: {}", ctx.accounts.hub.manager);
            return err!(SolX404Error::OnlyCallByFactory);
        }
        msg!("initialize x404 state");
        state.source = ctx.accounts.source.to_account_info().key();
        state.decimal = params.decimals;
        state.redeem_fee = params.redeem_fee;
        state.redeem_max_deadline = params.redeem_max_deadline;
        state.owner = ctx.accounts.hub.to_account_info().key();

        msg!("initiate NFT collection");

        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        );

        mint_to(cpi_context, 1)?;
        msg!("NFT minted successfully.");

        let seeds = &[
            "404_contract_nft_mint".as_bytes(),
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.nft_mint],
        ];

        msg!("Creating NFT metadata.");
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    mint_authority: ctx.accounts.signer.to_account_info(),
                    payer: ctx.accounts.signer.to_account_info(),
                    update_authority: ctx.accounts.signer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            DataV2 {
                name: params.name,
                symbol: params.symbol,
                uri: params.uri,
                seller_fee_basis_points: 5,
                creators: None,
                collection: None,
                uses: None,
            },
            false,
            true,
            None,
        )?;
        msg!("Close NFT Account.");
        if set_authority(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.nft_mint.key(),
            None,
            AuthorityType::FreezeAccount,
            &ctx.accounts.nft_mint.key(),
            &[&ctx.accounts.nft_mint.key()],
        )
        .is_err()
        {
            return err!(SolX404Error::FailedToGenerateAccount);
        }

        msg!("NFT mint created successfully.");
        Ok(())
    }
}
