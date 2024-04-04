pub mod error;
pub mod instructions;
pub mod state;

mod utils;

use anchor_lang::prelude::*;
use state::*;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod x404 {

    use std::cmp::min;

    use anchor_spl::{
        associated_token::get_associated_token_address_with_program_id,
        token_2022::spl_token_2022::{
            extension::{transfer_hook::instruction::initialize as hook_initialize, ExtensionType},
            state::Mint,
        },
    };
    use solana_program::program::invoke;
    use utils::transfer_from_owner_store;

    use super::*;
    use crate::{error::SolX404Error, utils::*};

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let hub = &mut ctx.accounts.state;
        hub.manager = *ctx.accounts.signer.key;
        hub.emergency_close = false;
        msg!(
            "Initialized new hub: {} with owner {}!",
            hub.to_account_info().key,
            hub.manager
        );
        Ok(())
    }

    pub fn create_x404(ctx: Context<CreateX404>, params: InitTokenParams) -> Result<()> {
        msg!("check permission for create x404");
        require!(
            ctx.accounts.signer.key() == ctx.accounts.hub.manager,
            SolX404Error::OnlyCallByOwner
        );

        msg!("initialize x404 state");
        let state = &mut ctx.accounts.state;
        state.source = ctx.accounts.source.to_account_info().key();
        state.decimal = params.decimals;
        state.redeem_fee = params.redeem_fee;
        state.redeem_max_deadline = params.redeem_max_deadline;
        state.owner = ctx.accounts.signer.key();
        state.fungible_mint = ctx.accounts.fungible_mint.to_account_info().key();
        state.collection_mint = ctx.accounts.collection_mint.to_account_info().key();
        state.fungible_hook = params.hook_extra_account;
        state.nft_supply = 0;
        state.nft_in_use = 0;
        state.fungible_supply = params.fungible_supply;

        msg!("create fungible mint");

        let seeds = [
            b"fungible_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.fungible_mint],
        ];

        let mint_size =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook])?;

        create_new_account(
            seeds.as_ref(),
            Rent::get()?,
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            mint_size as u64,
            ctx.accounts.token_program.key,
        )?;

        msg!("initialize transfer hook");
        // init transfer hook
        let extra_init = hook_initialize(
            ctx.accounts.token_program.key,
            ctx.accounts.fungible_mint.key,
            None,
            Some(params.hook_program),
        )?;

        invoke(&extra_init, &[ctx.accounts.fungible_mint.to_account_info()])?;

        // init fungible mint

        msg!("initiate fungible mint");
        initiate_mint_account(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            params.decimals,
        )?;
        msg!("fungible mint created successfully");
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

        require!(
            ctx.accounts.signer.key() == ctx.accounts.state.owner,
            SolX404Error::OnlyCallByOwner
        );

        require!(
            ctx.accounts.collection_mint.key() == ctx.accounts.state.collection_mint,
            SolX404Error::InvalidNFTAddress
        );

        msg!("start to mint");
        // seeds for state account
        let seeds = [
            b"collection_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.collection_mint],
        ];

        let nft_signer = [seeds.as_ref()];
        require!(
            ctx.accounts.collection_mint.supply == 0,
            SolX404Error::NFTAlreadyMinted
        );
        mint_nft(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.collection_token.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            nft_signer.as_slice(),
        )?;

        msg!("NFT minted successfully.");
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositSPLNFT>, params: DepositParams) -> Result<()> {
        msg!("check permission for deposit nft");

        // TODO check deposit mint's metadata against the state source
        require!(
            params.redeem_deadline < ctx.accounts.state.redeem_max_deadline,
            SolX404Error::InvaildRedeemDeadline
        );
        // create associated account for the nft
        let state_seeds = [
            b"state",
            ctx.accounts.state.source.as_ref(),
            &[ctx.bumps.state],
        ];

        // deposit the spl nft to state.
        transfer_spl_token(
            ctx.accounts.deposit_program.to_account_info(),
            ctx.accounts.deposit_mint.to_account_info(),
            ctx.accounts.deposit_holder.to_account_info(),
            ctx.accounts.deposit_receiver.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            1,
            0,
            None,
        )?;

        // close account to save rent
        close_spl_account(
            ctx.accounts.deposit_program.to_account_info(),
            ctx.accounts.deposit_holder.to_account_info(),
            ctx.accounts.signer.to_account_info(),
        )?;

        msg!("init bank");

        ctx.accounts.nft_bank.id = ctx.accounts.deposit_mint.to_account_info().key();
        ctx.accounts.nft_bank.owner = ctx.accounts.signer.to_account_info().key();
        ctx.accounts.nft_bank.redeem_deadline = params.redeem_deadline + Clock::get()?.epoch;

        if ctx.accounts.state.nft_supply > ctx.accounts.state.nft_in_use {
            // if sufficient store, just keep use the existing nft

            msg!("use existed nft");
            transfer_from_owner_store(
                &mut ctx.accounts.owner_store,
                ctx.accounts.state.key(),
                ctx.accounts.signer.key(),
                1,
            )?;
            ctx.accounts.state.nft_in_use += 1;
        } else {
            msg!("mint new nft");
            let expected_key = get_associated_token_address_with_program_id(
                &ctx.accounts.state.key(),
                ctx.accounts.nft_mint.key,
                ctx.accounts.deposit_program.key,
            );
            require!(
                expected_key == ctx.accounts.nft_token.to_account_info().key(),
                SolX404Error::InvalidNFTAddress
            );

            msg!("start to mint");
            // seeds for state account
            let rent = Rent::get()?;

            let nft_id = ctx.accounts.state.nft_supply.to_le_bytes();
            let mint_seeds = [
                b"nft_mint",
                ctx.accounts.state.source.as_ref(),
                nft_id.as_ref(),
                &[ctx.bumps.nft_mint],
            ];

            let nft_signer = [mint_seeds.as_ref()];

            // we let state to pay for the rent for fair play
            create_new_account(
                mint_seeds.as_ref(),
                rent.clone(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                MINT_SIZE,
                ctx.accounts.token_program.key,
            )?;

            initiate_mint_account(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                0,
            )?;

            create_token_account(
                ctx.accounts.associated_token_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.state.to_account_info(),
                ctx.accounts.state.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.nft_token.to_account_info(),
                state_seeds.as_ref(),
            )?;
            // mint nft first to state, later, the user can bind the nft by fungible tokens for trading

            mint_nft(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.nft_token.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                nft_signer.as_slice(),
            )?;
            ctx.accounts.state.nft_supply += 1;
            ctx.accounts.state.nft_in_use += 1;

            add_to_owner_store(
                &mut ctx.accounts.owner_store,
                rent,
                ctx.accounts.nft_mint.key(),
                ctx.accounts.signer.key(),
            )?;

            msg!("NFT minted successfully.");
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

    pub fn redeem(ctx: Context<RedeemSPLNFT>, _params: RedeemParams) -> Result<()> {
        msg!("check permission for create collection");

        // redeem check
        if ctx.accounts.signer.key() != ctx.accounts.nft_bank.owner {
            require!(
                ctx.accounts.nft_bank.redeem_deadline < Clock::get()?.epoch,
                SolX404Error::NFTCannotRedeem
            );

            require!(
                ctx.accounts.fungible_token.amount
                    >= ctx.accounts.state.redeem_fee + ctx.accounts.state.fungible_supply,
                SolX404Error::InsufficientFee
            );
        } else {
            require!(
                ctx.accounts.fungible_token.amount >= ctx.accounts.state.fungible_supply,
                SolX404Error::InsufficientFee
            );
        }

        // Charge Fee
        let old_sender: usize =
            (ctx.accounts.fungible_token.amount / ctx.accounts.state.fungible_supply) as usize;

        if ctx.accounts.signer.key() != ctx.accounts.nft_bank.owner {
            // charge fee
            let old_receiver: usize =
                (ctx.accounts.original_owner.amount / ctx.accounts.state.fungible_supply) as usize;
            transfer_token(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.fungible_token.to_account_info(),
                ctx.accounts.fungible_mint.to_account_info(),
                ctx.accounts.original_owner.to_account_info(),
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.state.redeem_fee,
                ctx.accounts.state.decimal,
                None,
            )?;

            // there should always been sufficient nft for this operation
            transfer_from_owner_store(
                &mut ctx.accounts.owner_store,
                ctx.accounts.state.key(),
                ctx.accounts.original_owner.key(),
                (ctx.accounts.fungible_token.amount / ctx.accounts.state.fungible_supply) as usize
                    - old_receiver,
            )?;

            msg!("redeem fee charged.");
        }

        // burn fungible token
        burn_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_token.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.signer.to_account_info(),
        )?;

        transfer_from_owner_store(
            &mut ctx.accounts.owner_store,
            ctx.accounts.signer.key(),
            ctx.accounts.state.key(),
            (ctx.accounts.fungible_token.amount / ctx.accounts.state.fungible_supply) as usize
                - old_sender,
        )?;
        ctx.accounts.state.nft_in_use -= 1;

        msg!("Fungible Token burned successfully.");

        // Transfer NFT to the signer
        let seeds = [
            b"state",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.state],
        ];

        let state_signer = [seeds.as_ref()];
        transfer_spl_token(
            ctx.accounts.withdrawal_program.to_account_info(),
            ctx.accounts.withdraw_mint.to_account_info(),
            ctx.accounts.withdraw_holder.to_account_info(),
            ctx.accounts.withdrawal_receiver.to_account_info(),
            ctx.accounts.state.to_account_info(),
            1,
            0,
            Some(state_signer.as_slice()),
        )?;

        close_spl_account(
            ctx.accounts.withdrawal_program.to_account_info(),
            ctx.accounts.withdraw_holder.to_account_info(),
            ctx.accounts.state.to_account_info(),
        )?;

        Ok(())
    }

    pub fn bind_nft(ctx: Context<BindNFT>, _params: BindParams) -> Result<()> {
        msg!("check permission for create collection");

        // fetch the nft

        take_from_owner_store(
            &mut ctx.accounts.owner_store,
            ctx.accounts.signer.key(),
            ctx.accounts.bind_mint.key(),
        )?;

        // burn the token for bind

        burn_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_token.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.signer.to_account_info(),
        )?;

        // send the corresponding nft to the signer
        let seeds = [
            b"state",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.state],
        ];

        let state_signer = [seeds.as_ref()];
        transfer_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.bind_mint.to_account_info(),
            ctx.accounts.bind_holder.to_account_info(),
            ctx.accounts.bind_token.to_account_info(),
            ctx.accounts.state.to_account_info(),
            1,
            0,
            Some(state_signer.as_slice()),
        )?;

        Ok(())
    }

    pub fn unbind_nft(ctx: Context<UnbindNFT>, _params: UnbindParams) -> Result<()> {
        msg!("check permission for create collection");

        // add back the nft to owner store

        add_to_owner_store(
            &mut ctx.accounts.owner_store,
            Rent::get()?,
            ctx.accounts.signer.key(),
            ctx.accounts.bind_mint.key(),
        )?;

        // send the corresponding nft to the signer

        transfer_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.bind_mint.to_account_info(),
            ctx.accounts.bind_token.to_account_info(),
            ctx.accounts.bind_receiver.to_account_info(),
            ctx.accounts.state.to_account_info(),
            1,
            0,
            None,
        )?;

        let seeds = [
            b"fungible_mint",
            ctx.accounts.state.to_account_info().key.as_ref(),
            &[ctx.bumps.fungible_mint],
        ];

        let mint_signer = [seeds.as_ref()];
        // burn the token for bind

        mint_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_token.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.signer.to_account_info(),
            mint_signer.as_slice(),
        )?;
        Ok(())
    }

    pub fn rebalance(ctx: Context<Rebalance>, params: RebalanceParams) -> Result<()> {
        msg!("check permission for create collection");

        // permission check
        require!(
            ctx.accounts.hooker.key() == ctx.accounts.state.fungible_hook,
            SolX404Error::OnlyCallByHooker
        );

        // update

        let to_remove = (ctx.accounts.sender_account.amount / ctx.accounts.state.fungible_supply
            - (ctx.accounts.sender_account.amount - params.amount)
                / ctx.accounts.state.fungible_supply) as usize;

        let to_add = ((ctx.accounts.sender_account.amount + params.amount)
            / ctx.accounts.state.fungible_supply
            - ctx.accounts.sender_account.amount / ctx.accounts.state.fungible_supply)
            as usize;

        // if to_add = to_remove, the amount in second call of `transfer_from_owner_store` will be 0
        // which will be skipped in the function, so no need to check here
        if to_add > to_remove {
            transfer_from_owner_store(
                &mut ctx.accounts.owner_store,
                ctx.accounts.state.to_account_info().key(),
                params.receiver,
                to_add - to_remove,
            )?;
        } else {
            transfer_from_owner_store(
                &mut ctx.accounts.owner_store,
                params.sender,
                ctx.accounts.state.to_account_info().key(),
                to_remove - to_add,
            )?;
        }

        transfer_from_owner_store(
            &mut ctx.accounts.owner_store,
            params.sender,
            params.receiver,
            min(to_add, to_remove),
        )?;
        Ok(())
    }
}
