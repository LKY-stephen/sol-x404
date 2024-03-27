pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use state::*;

use anchor_spl::token_2022::MintTo;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod x404 {
    use anchor_spl::token_2022::{
        freeze_account, mint_to, set_authority, spl_token_2022::instruction::AuthorityType,
        FreezeAccount, SetAuthority,
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
        msg!("check permission for create x404");
        if ctx.accounts.signer.key() != ctx.accounts.hub.manager {
            msg!("signer: {}", ctx.accounts.signer.key());
            msg!("hub manager: {}", ctx.accounts.hub.manager);
            return err!(SolX404Error::OnlyCallByOwner);
        }

        msg!("initialize x404 state");
        let state = &mut ctx.accounts.state;
        state.source = ctx.accounts.source.to_account_info().key();
        state.decimal = params.decimals;
        state.redeem_fee = params.redeem_fee;
        state.redeem_max_deadline = params.redeem_max_deadline;
        state.owner = ctx.accounts.signer.key();
        state.fungible_mint = ctx.accounts.fungible_mint.to_account_info().key();
        state.nft_mint = ctx.accounts.nft_mint.to_account_info().key();
        state.nft_supply = 0;
        state.nft_in_use = 0;
        state.nft_instore = [0; 128];
        Ok(())
    }

    // mint collection should be done together with initiate state
    // however, it seems too many operations at the same time will break
    // the stack, so split to two. Should add sufficient integrity check
    // here to make sure not initiate state twice.
    pub fn mint_collection(
        ctx: Context<MintCollection>,
        params: InitCollectionParams,
    ) -> Result<()> {
        msg!("check permission for create collection");

        if ctx.accounts.signer.key() != ctx.accounts.state.owner {
            msg!("signer: {}", ctx.accounts.signer.key());
            msg!("owner: {}", ctx.accounts.state.owner);
            return err!(SolX404Error::OnlyCallByOwner);
        }

        if ctx.accounts.nft_mint.key() != ctx.accounts.state.nft_mint {
            msg!("signer: {}", ctx.accounts.nft_mint.key());
            msg!("owner: {}", ctx.accounts.state.nft_mint);
            return err!(SolX404Error::InvalidNFTAddress);
        }

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

        // metaplex is too annoying for this poc, left it for future
        // msg!("Creating NFT metadata.");

        let seeds = [
            b"404_contract_nft_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.nft_mint],
        ];

        let pad_signer = [seeds.as_ref()];

        //

        // create_metadata_accounts_v3(
        //     CpiContext::new_with_signer(
        //         ctx.accounts.token_metadata_program.to_account_info(),
        //         CreateMetadataAccountsV3 {
        //             metadata: ctx.accounts.metadata.to_account_info(),
        //             mint: ctx.accounts.nft_mint.to_account_info(),
        //             mint_authority: ctx.accounts.signer.to_account_info(),
        //             payer: ctx.accounts.signer.to_account_info(),
        //             update_authority: ctx.accounts.signer.to_account_info(),
        //             system_program: ctx.accounts.system_program.to_account_info(),
        //             rent: ctx.accounts.rent.to_account_info(),
        //         },
        //         [seeds.as_slice()].as_slice(),
        //     ),
        //     params.data(),
        //     false,
        //     true,
        //     None,
        // )?;
        msg!("Close NFT Account.");
        let close_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            FreezeAccount {
                current_authority: ctx.accounts.nft_mint.to_account_info(),
                account_or_mint: ctx.accounts.nft_token.to_account_info(),
            },
            pad_signer.as_slice(),
        );
        if freeze_account(close_context).is_err() {
            return err!(SolX404Error::FailedToGenerateAccount);
        }
        Ok(())
    }

    // pub fn deposit(ctx: Context<DepositToken>, params: DepositParams) -> Result<()> {
    //     // msg!("check permission for create collection");

    //     // if ctx.accounts.signer.key() != ctx.accounts.state.owner {
    //     //     msg!("signer: {}", ctx.accounts.signer.key());
    //     //     msg!("owner: {}", ctx.accounts.state.owner);
    //     //     return err!(SolX404Error::OnlyCallByOwner);
    //     // }

    //     // if ctx.accounts.nft_mint.key() != ctx.accounts.state.nft_mint {
    //     //     msg!("signer: {}", ctx.accounts.nft_mint.key());
    //     //     msg!("owner: {}", ctx.accounts.state.nft_mint);
    //     //     return err!(SolX404Error::InvalidNFTAddress);
    //     // }

    //     // let cpi_context = CpiContext::new(
    //     //     ctx.accounts.token_program.to_account_info(),
    //     //     MintTo {
    //     //         mint: ctx.accounts.nft_mint.to_account_info(),
    //     //         to: ctx.accounts.nft_token.to_account_info(),
    //     //         authority: ctx.accounts.signer.to_account_info(),
    //     //     },
    //     // );

    //     // mint_to(cpi_context, 1)?;
    //     // msg!("NFT minted successfully.");
    //     // msg!("Creating NFT metadata.");

    //     // let _seeds = &[
    //     //     "404_contract_nft_mint".as_bytes(),
    //     //     ctx.accounts.state.to_account_info().key.as_ref(),
    //     //     &[ctx.bumps.nft_mint],
    //     // ];

    //     // let _metadata = params.data();
    //     // let _account_data = CreateMetadataAccountsV3 {
    //     //     metadata: ctx.accounts.metadata.to_account_info(),
    //     //     mint: ctx.accounts.nft_mint.to_account_info(),
    //     //     mint_authority: ctx.accounts.signer.to_account_info(),
    //     //     payer: ctx.accounts.signer.to_account_info(),
    //     //     update_authority: ctx.accounts.signer.to_account_info(),
    //     //     system_program: ctx.accounts.system_program.to_account_info(),
    //     //     rent: ctx.accounts.rent.to_account_info(),
    //     // };

    //     // unit test cannot find the program properly, so skip metadata for now.
    //     // create_metadata_accounts_v3(
    //     //     CpiContext::new_with_signer(
    //     //         ctx.accounts.token_metadata_program.to_account_info(),
    //     //         account_data,
    //     //         &[&seeds[..]],
    //     //     ),
    //     //     metadata,
    //     //     false,
    //     //     true,
    //     //     None,
    //     // )?;
    //     // msg!("Close NFT Account.");
    //     // if set_authority(
    //     //     &ctx.accounts.token_program.key(),
    //     //     &ctx.accounts.nft_mint.key(),
    //     //     None,
    //     //     AuthorityType::FreezeAccount,
    //     //     &ctx.accounts.nft_mint.key(),
    //     //     &[&ctx.accounts.nft_mint.key()],
    //     // )
    //     // .is_err()
    //     // {
    //     //     return err!(SolX404Error::FailedToGenerateAccount);
    //     // }
    //     Ok(())
    // }
}
