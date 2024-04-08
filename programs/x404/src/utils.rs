use std::cmp::min;

use anchor_lang::{
    context::CpiContext,
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::{
    token::{
        close_account as spl_close_account, transfer_checked as spl_transfer_checked,
        CloseAccount as CloseSPLAccount, TransferChecked as SPLTransferChecked,
    },
    token_2022::{
        burn, close_account, initialize_mint2, mint_to, Burn, CloseAccount, InitializeMint2, MintTo,
    },
    token_interface::Mint,
};

use crate::{error::SolX404Error, OwnerStore};

pub(crate) fn mint_nft<'info>(
    token_program: AccountInfo<'info>,
    nft_mint: &mut InterfaceAccount<'info, Mint>,
    to: AccountInfo<'info>,
    current_mint_authority: AccountInfo<'info>,
    nft_signer: &[&[&[u8]]],
) -> Result<()> {
    require_eq!(nft_mint.supply, 0, SolX404Error::NFTAlreadyMinted);
    require!(
        mint_token(
            token_program,
            nft_mint.to_account_info(),
            to,
            1,
            current_mint_authority,
            nft_signer,
        )
        .is_ok(),
        SolX404Error::FailedToMintNFT
    );
    Ok(())
}

pub(crate) fn mint_token<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
    mint_authority: AccountInfo<'info>,
    authority_seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_context = CpiContext::new_with_signer(
        token_program,
        MintTo {
            mint: mint,
            to,
            authority: mint_authority,
        },
        authority_seeds,
    );

    mint_to(cpi_context, amount)
}

pub(crate) fn burn_token<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    amount: u64,
    signer: AccountInfo<'info>,
) -> Result<()> {
    let cpi_context = CpiContext::new(
        token_program,
        Burn {
            mint: mint,
            from,
            authority: signer,
        },
    );

    burn(cpi_context, amount)
}

pub(crate) fn transfer_spl_token<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    decimal: u8,
    seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_context = CpiContext::new_with_signer(
        token_program.clone(),
        SPLTransferChecked {
            mint,
            from,
            to,
            authority,
        },
        seeds,
    );

    spl_transfer_checked(cpi_context, amount, decimal)
}

pub(crate) fn close_spl_account<'info>(
    program: AccountInfo<'info>,
    target: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_context = CpiContext::new_with_signer(
        program,
        CloseSPLAccount {
            account: target,
            authority: authority.clone(),
            destination: authority,
        },
        seeds,
    );

    spl_close_account(cpi_context)
}

pub(crate) fn close_token_account<'info>(
    program: AccountInfo<'info>,
    target: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_context = CpiContext::new_with_signer(
        program,
        CloseAccount {
            account: target,
            authority: authority.clone(),
            destination: authority,
        },
        seeds,
    );

    close_account(cpi_context)
}

pub(crate) fn add_to_owner_store(
    account: &mut Account<'_, OwnerStore>,
    target: Pubkey,
    owner: Pubkey,
) -> Result<()> {
    let mut map = account.get_map();
    map.entry(owner).or_insert_with(Vec::new).push(target);
    // add rent,
    let new_len = map.try_to_vec()?.len() + 4 + 8;
    account.to_account_info().realloc(new_len, false)?;
    account.update_map(&map);

    Ok(())
}

pub(crate) fn transfer_from_owner_store(
    account: &mut Account<'_, OwnerStore>,
    owner: Pubkey,
    to: Pubkey,
    amount: usize,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let mut map = account.get_map();
    let record = map.entry(owner).or_insert_with(Vec::new);
    require!(record.len() >= amount, SolX404Error::InsufficientNFT);

    let mut tail = record.split_off(record.len() - amount);
    let target = map.entry(to).or_insert_with(Vec::new);
    target.append(&mut tail);

    let new_len = map.try_to_vec()?.len() + 4 + 8;
    account.to_account_info().realloc(new_len, false)?;
    account.update_map(&map);

    Ok(())
}

pub(crate) fn take_from_owner_store(
    account: &mut Account<'_, OwnerStore>,
    owner: Pubkey,
    target: Pubkey,
) -> Result<()> {
    let mut map = account.get_map();
    let record = map.entry(owner).or_insert_with(Vec::new);

    require!(record.contains(&target), SolX404Error::InsufficientNFT);

    record.retain(|x| x != &target);

    account.update_map(&map);

    Ok(())
}

pub(crate) fn create_new_account<'info>(
    seeds: &[&[u8]],
    rent: Rent,
    system_program: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    account: AccountInfo<'info>,
    space: u64,
    owner: &Pubkey,
) -> Result<()> {
    let signer = [seeds];

    let init_ctx = CpiContext::new(
        system_program,
        CreateAccount {
            from: payer,
            to: account,
        },
    )
    .with_signer(signer.as_slice());

    create_account(init_ctx, rent.minimum_balance(space as usize), space, owner)?;

    Ok(())
}

pub(crate) fn initiate_mint_account<'info>(
    token_program: AccountInfo<'info>,
    mint_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    decimal: u8,
) -> Result<()> {
    let init_mint_ctx = CpiContext::new(
        token_program,
        InitializeMint2 {
            mint: mint_account.clone(),
        },
    );

    initialize_mint2(init_mint_ctx, decimal, &authority.key, None)?;
    Ok(())
}

pub(crate) fn do_rebalance<'info>(
    owner_store: &mut Account<'info, OwnerStore>,
    sender: Pubkey,
    receiver: Pubkey,
    state: Pubkey,
    to_add: usize,
    to_remove: usize,
) -> Result<()> {
    if to_add > to_remove {
        transfer_from_owner_store(owner_store, state, receiver, to_add - to_remove)?;
    } else {
        transfer_from_owner_store(owner_store, sender, state, to_remove - to_add)?;
    }
    transfer_from_owner_store(owner_store, sender, receiver, min(to_add, to_remove))?;

    msg!("rebalance accomplished");
    Ok(())
}
