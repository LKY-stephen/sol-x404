pub mod error;
pub mod instructions;
pub mod state;

mod utils;

use anchor_lang::prelude::*;
use state::*;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod x404 {

    use super::*;
    use crate::{
        error::SolX404Error,
        utils::{
            add_to_owner_store, initiate_owner_store, mint_nft, mint_token, transfer_spl_token,
        },
    };

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
        state.fungible_supply = params.fungible_supply;
        Ok(())
    }

    // mint collection should be done together with initiate state
    // however, it seems too many operations at the same time will break
    // the stack, so split to two. Should add sufficient integrity check
    // here to make sure not initiate state twice.
    pub fn mint_collection(
        ctx: Context<MintCollection>,
        _params: InitCollectionParams,
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

        msg!("start to mint");
        // seeds for state account
        let seeds = [
            b"nft_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.nft_mint],
        ];

        let nft_signer = [seeds.as_ref()];
        mint_nft(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.nft_mint.clone(),
            ctx.accounts.nft_token.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            nft_signer.as_slice(),
        )?;

        msg!("NFT minted successfully.");
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositSPLNFT>, params: DepositParams) -> Result<()> {
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

        // TODO check deposit mint's metadata against the state source

        msg!("init bank");

        transfer_spl_token(
            ctx.accounts.deposit_program.to_account_info(),
            ctx.accounts.deposit_mint.to_account_info(),
            ctx.accounts.deposit.to_account_info(),
            ctx.accounts.nft_bank.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.state.decimal,
        )?;

        ctx.accounts.nft_bank.id = ctx.accounts.deposit_mint.to_account_info().key();
        ctx.accounts.nft_bank.owner = ctx.accounts.signer.to_account_info().key();
        ctx.accounts.nft_bank.redeem_deadline = params.redeem_deadline;

        msg!("start to mint");
        // seeds for state account
        let seeds = [
            b"nft_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            ctx.accounts.deposit_mint.to_account_info().key.as_ref(),
            &[ctx.bumps.nft_mint],
        ];

        let nft_signer = [seeds.as_ref()];
        mint_nft(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.nft_mint.clone(),
            ctx.accounts.nft_token.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            nft_signer.as_slice(),
        )?;

        msg!("NFT minted successfully.");

        let rent = Rent::get()?;
        if ctx.accounts.owner_store.get_lamports() == 0 {
            initiate_owner_store(
                ctx.accounts.owner_store.clone(),
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.state.to_account_info(),
                rent,
                ctx.accounts.deposit_mint.to_account_info().key(),
            )?;
        } else {
            add_to_owner_store(
                ctx.accounts.owner_store.clone(),
                rent,
                ctx.accounts.deposit_mint.to_account_info().key(),
            )?;
        }

        msg!("NFT recorded");

        let seeds = [
            b"fungible_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.fungible_mint],
        ];

        let fungible_signer = [seeds.as_ref()];

        mint_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_token.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.fungible_mint.to_account_info(),
            fungible_signer.as_slice(),
        )?;

        msg!("Fungible Token minted successfully.");
        Ok(())
    }
}
