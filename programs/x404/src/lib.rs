pub mod error;
pub mod instructions;
pub mod state;

mod utils;

use anchor_lang::prelude::*;
use state::*;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod x404 {
    use std::{borrow::BorrowMut, collections::HashMap, ops::Deref};

    use anchor_spl::token_2022::spl_token_2022::{
        extension::{transfer_hook::instruction::initialize as hook_initialize, ExtensionType},
        state::Mint,
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
        require_eq!(
            ctx.accounts.signer.key(),
            ctx.accounts.hub.manager,
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
            ctx.accounts.state.to_account_info(),
            params.decimals,
        )?;

        msg!("fungible mint created successfully");

        let initiate_map = HashMap::<Pubkey, Vec<Pubkey>>::new();
        ctx.accounts.owner_store.store = initiate_map.try_to_vec()?;

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

        require_eq!(
            ctx.accounts.signer.key(),
            ctx.accounts.state.owner,
            SolX404Error::OnlyCallByOwner
        );

        require_eq!(
            ctx.accounts.collection_mint.key(),
            ctx.accounts.state.collection_mint,
            SolX404Error::InvalidNFTAddress
        );

        msg!("start to mint");
        // seeds for state account
        let seeds = [
            b"state",
            ctx.accounts.state.source.as_ref(),
            &[ctx.bumps.state],
        ];

        let state_signer = [seeds.as_ref()];
        mint_nft(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.collection_mint.borrow_mut(),
            ctx.accounts.collection_token.to_account_info(),
            ctx.accounts.state.to_account_info(),
            state_signer.as_slice(),
        )?;

        msg!("NFT minted successfully.");
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositSPLNFT>, params: DepositParams) -> Result<()> {
        msg!("check permission for deposit nft");

        // TODO check deposit mint's metadata against the state source
        require_gt!(
            ctx.accounts.state.redeem_max_deadline,
            params.redeem_deadline,
            SolX404Error::InvaildRedeemDeadline
        );

        // deposit the spl nft to state.
        transfer_spl_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.deposit_mint.to_account_info(),
            ctx.accounts.deposit_holder.to_account_info(),
            ctx.accounts.deposit_receiver.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            1,
            0,
            &[],
        )?;

        // close account to save rent
        close_spl_account(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.deposit_holder.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            &[],
        )?;

        msg!("init bank");

        ctx.accounts.nft_bank.id = ctx.accounts.deposit_mint.to_account_info().key();
        ctx.accounts.nft_bank.owner = ctx.accounts.signer.to_account_info().key();
        ctx.accounts.nft_bank.redeem_deadline = params.redeem_deadline + Clock::get()?.epoch;
        ctx.accounts.nft_bank.issued = false;
        Ok(())
    }

    pub fn issue_token(ctx: Context<IssueTokens>, params: IssueTokenParams) -> Result<()> {
        msg!("check permission for issue new tokens");

        require_eq!(
            ctx.accounts.owner.key(),
            ctx.accounts.state.owner,
            SolX404Error::OnlyCallByOwner
        );

        require_eq!(
            ctx.accounts.user.key(),
            ctx.accounts.nft_bank.owner,
            SolX404Error::InvalidDepositer
        );

        require_eq!(
            ctx.accounts.nft_bank.issued,
            false,
            SolX404Error::NFTAlreadyMinted
        );

        let state_seeds = [b"state", params.source.as_ref(), &[ctx.bumps.state]];

        mint_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_token.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.state.to_account_info(),
            [state_seeds.as_ref()].as_slice(),
        )?;

        msg!("Fungible Token minted successfully.");

        if ctx.accounts.state.nft_supply > ctx.accounts.state.nft_in_use {
            msg!("use existed nft");
            transfer_from_owner_store(
                &mut ctx.accounts.owner_store,
                ctx.accounts.state.key(),
                ctx.accounts.user.key(),
                1,
            )?;
            ctx.accounts.state.nft_in_use += 1;
        } else {
            // if the mint is initiated before, then it never added to the store
            // otherwise, it should be added to the store now

            ctx.accounts.state.nft_supply += 1;

            msg!("update owner store");
            add_to_owner_store(
                &mut ctx.accounts.owner_store,
                ctx.accounts.nft_mint.key(),
                ctx.accounts.user.key(),
            )?;

            msg!("NFT minted successfully.");
        }

        ctx.accounts.state.nft_in_use += 1;
        ctx.accounts.nft_bank.issued = true;
        msg!("NFT recorded");

        Ok(())
    }

    pub fn redeem(ctx: Context<RedeemSPLNFT>, _params: RedeemParams) -> Result<()> {
        msg!("check permission for redeem NFT");

        // redeem check
        if ctx.accounts.signer.key() != ctx.accounts.nft_bank.owner {
            require_gt!(
                Clock::get()?.epoch,
                ctx.accounts.nft_bank.redeem_deadline,
                SolX404Error::NFTCannotRedeem
            );

            require_gte!(
                ctx.accounts.fungible_token.amount,
                ctx.accounts.state.redeem_fee + ctx.accounts.state.fungible_supply,
                SolX404Error::InsufficientFee
            );
        } else {
            require_gte!(
                ctx.accounts.fungible_token.amount,
                ctx.accounts.state.fungible_supply,
                SolX404Error::InsufficientFee
            );
        }

        let seeds = [
            b"state",
            ctx.accounts.state.source.as_ref(),
            &[ctx.bumps.state],
        ];

        let state_signer = [seeds.as_ref()];

        if ctx.accounts.signer.key() != ctx.accounts.nft_bank.owner {
            // charge fee

            let to_remove = (ctx.accounts.fungible_token.amount
                / ctx.accounts.state.fungible_supply) as usize
                - ((ctx.accounts.fungible_token.amount
                    - ctx.accounts.state.redeem_fee
                    - ctx.accounts.state.fungible_supply)
                    / ctx.accounts.state.fungible_supply) as usize;

            let to_add = ((ctx.accounts.original_owner.amount + ctx.accounts.state.redeem_fee)
                / ctx.accounts.state.fungible_supply) as usize
                - (ctx.accounts.original_owner.amount / ctx.accounts.state.fungible_supply)
                    as usize;

            // we do not want to trigger the hook here
            // so we use mint and burn way

            burn_token(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.fungible_mint.to_account_info(),
                ctx.accounts.fungible_token.to_account_info(),
                ctx.accounts.state.redeem_fee + ctx.accounts.state.fungible_supply,
                ctx.accounts.signer.to_account_info(),
            )?;

            msg!(
                "{} lose {}",
                ctx.accounts.fungible_token.deref().owner,
                to_remove
            );

            mint_token(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.fungible_mint.to_account_info(),
                ctx.accounts.original_owner.to_account_info(),
                ctx.accounts.state.redeem_fee,
                ctx.accounts.state.to_account_info(),
                &state_signer,
            )?;

            msg!(
                "{} get {}",
                ctx.accounts.original_owner.deref().owner,
                to_add
            );

            do_rebalance(
                &mut ctx.accounts.owner_store,
                ctx.accounts.fungible_token.deref().owner,
                ctx.accounts.original_owner.deref().owner,
                ctx.accounts.state.to_account_info().key(),
                to_add,
                to_remove,
            )?;

            msg!("redeem fee charged.");
        } else {
            burn_token(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.fungible_mint.to_account_info(),
                ctx.accounts.fungible_token.to_account_info(),
                ctx.accounts.state.fungible_supply,
                ctx.accounts.signer.to_account_info(),
            )?;
            msg!("{} lose {}", ctx.accounts.fungible_token.deref().owner, 1);
            // remove nft due to redeem. Must be 1.
            transfer_from_owner_store(
                &mut ctx.accounts.owner_store,
                ctx.accounts.signer.key(),
                ctx.accounts.state.key(),
                1,
            )?;
        }

        msg!("Fungible Token burned successfully.");

        // Transfer NFT to the signer
        transfer_spl_token(
            ctx.accounts.withdrawal_program.to_account_info(),
            ctx.accounts.withdraw_mint.to_account_info(),
            ctx.accounts.withdraw_holder.to_account_info(),
            ctx.accounts.withdrawal_receiver.to_account_info(),
            ctx.accounts.state.to_account_info(),
            1,
            0,
            &state_signer,
        )?;

        close_spl_account(
            ctx.accounts.withdrawal_program.to_account_info(),
            ctx.accounts.withdraw_holder.to_account_info(),
            ctx.accounts.state.to_account_info(),
            &state_signer,
        )?;

        ctx.accounts.state.nft_in_use -= 1;

        Ok(())
    }

    pub fn bind_nft(ctx: Context<BindNFT>, _params: BindParams) -> Result<()> {
        msg!("check permission for bind nft");

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
            ctx.accounts.state.source.as_ref(),
            &[ctx.bumps.state],
        ];

        let state_signer = [seeds.as_ref()];
        mint_nft(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.bind_mint.borrow_mut(),
            ctx.accounts.bind_token.to_account_info(),
            ctx.accounts.state.to_account_info(),
            state_signer.as_slice(),
        )?;

        Ok(())
    }

    pub fn unbind_nft(ctx: Context<UnbindNFT>, params: UnbindParams) -> Result<()> {
        msg!("check permission for unbind nft");

        // send the corresponding nft to the signer
        burn_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.unbind_mint.to_account_info(),
            ctx.accounts.unbind_holder.to_account_info(),
            1,
            ctx.accounts.signer.to_account_info(),
        )?;

        close_token_account(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.unbind_holder.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            &[],
        )?;

        // add back the nft to owner store
        add_to_owner_store(
            &mut ctx.accounts.owner_store,
            ctx.accounts.unbind_mint.key(),
            ctx.accounts.signer.key(),
        )?;

        let seeds = [b"state", params.source.as_ref(), &[ctx.bumps.state]];

        let state_signer = [seeds.as_ref()];
        // remint token for signer

        mint_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_token.to_account_info(),
            ctx.accounts.state.fungible_supply,
            ctx.accounts.state.to_account_info(),
            state_signer.as_slice(),
        )?;
        Ok(())
    }

    pub fn rebalance(ctx: Context<Rebalance>, params: RebalanceParams) -> Result<()> {
        msg!("check permission for rebalance");

        // permission check
        require_eq!(
            ctx.accounts.hooker.key(),
            ctx.accounts.state.fungible_hook,
            SolX404Error::OnlyCallByHooker
        );
        require_eq!(
            ctx.accounts.hooker.is_signer,
            true,
            SolX404Error::OnlyCallByHooker
        );

        // update
        msg!(
            "{} -> {}: {}",
            params.sender,
            params.receiver,
            params.amount
        );

        // rebalance is triggered after transfer.
        let to_remove = ((ctx.accounts.sender_account.amount + params.amount)
            / ctx.accounts.state.fungible_supply
            - ctx.accounts.sender_account.amount / ctx.accounts.state.fungible_supply)
            as usize;

        let to_add = (ctx.accounts.receiver_account.amount / ctx.accounts.state.fungible_supply
            - (ctx.accounts.receiver_account.amount - params.amount)
                / ctx.accounts.state.fungible_supply) as usize;

        msg!("{} lose {}", params.sender, to_remove);
        msg!("{} get {}", params.receiver, to_add);

        // if to_add = to_remove, the amount in second call of `transfer_from_owner_store` will be 0
        // which will be skipped in the function, so no need to check here
        do_rebalance(
            &mut ctx.accounts.owner_store,
            params.sender,
            params.receiver,
            ctx.accounts.state.to_account_info().key(),
            to_add,
            to_remove,
        )?;
        Ok(())
    }
}
