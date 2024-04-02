use std::collections::HashMap;

use anchor_lang::{context::CpiContext, prelude::*};
use anchor_spl::{
    token::{
        close_account, transfer_checked as spl_transfer_checked, CloseAccount,
        TransferChecked as SPLTransferChecked,
    },
    token_2022::{
        burn, mint_to, set_authority, spl_token_2022::instruction::AuthorityType, transfer_checked,
        Burn, MintTo, SetAuthority, TransferChecked,
    },
};

use crate::{error::SolX404Error, OwnerStore};

pub(crate) fn mint_nft<'info>(
    token_program: AccountInfo<'info>,
    nft_mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    current_mint_authority: AccountInfo<'info>,
    nft_signer: &[&[&[u8]]],
) -> Result<()> {
    require!(
        mint_token(
            token_program.clone(),
            nft_mint.clone(),
            to.clone(),
            1,
            current_mint_authority.clone(),
            nft_signer,
        )
        .is_ok(),
        SolX404Error::FailedToMintNFT
    );

    msg!("Close NFT Account.");
    let close_context = CpiContext::new_with_signer(
        token_program,
        SetAuthority {
            account_or_mint: nft_mint.clone(),
            current_authority: current_mint_authority,
        },
        nft_signer,
    );

    require!(
        set_authority(close_context, AuthorityType::MintTokens, None).is_ok(),
        SolX404Error::FailedToCloseMint
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
        token_program.clone(),
        MintTo {
            mint: mint.clone(),
            to,
            authority: mint_authority.clone(),
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
        token_program.clone(),
        Burn {
            mint: mint.clone(),
            from,
            authority: signer.clone(),
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
    signer: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_context = CpiContext::new_with_signer(
        token_program.clone(),
        SPLTransferChecked {
            mint,
            from,
            to,
            authority,
        },
        signer.unwrap_or(&[]),
    );

    spl_transfer_checked(cpi_context, amount, decimal)
}

pub(crate) fn close_spl_account<'info>(
    program: AccountInfo<'info>,
    target: AccountInfo<'info>,
    authority: AccountInfo<'info>,
) -> Result<()> {
    let cpi_context = CpiContext::new(
        program,
        CloseAccount {
            account: target,
            authority: authority.clone(),
            destination: authority,
        },
    );

    close_account(cpi_context)
}

pub(crate) fn transfer_token<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    decimal: u8,
    signer: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_context = CpiContext::new_with_signer(
        token_program.clone(),
        TransferChecked {
            mint,
            from,
            to,
            authority,
        },
        signer.unwrap_or(&[]),
    );

    transfer_checked(cpi_context, amount, decimal)
}

pub(crate) fn add_to_owner_store(
    account: &mut Account<'_, OwnerStore>,
    rent: Rent,
    target: Pubkey,
    owner: Pubkey,
) -> Result<()> {
    let mut map = HashMap::<Pubkey, Vec<Pubkey>>::try_from_slice(account.store.as_slice())?;
    map.entry(owner).or_insert_with(Vec::new).push(target);

    let new_store = map.try_to_vec()?;
    // add rent
    account.add_lamports(rent.minimum_balance(new_store.len() - account.store.len()))?;
    account.store = new_store;

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

    let mut map = HashMap::<Pubkey, Vec<Pubkey>>::try_from_slice(account.store.as_slice())?;
    let record = map.entry(owner).or_insert_with(Vec::new);
    require!(record.len() >= amount, SolX404Error::InsufficientNFT);

    let mut tail = record.split_off(amount);
    let target = map.entry(to).or_insert(vec![]);
    target.append(&mut tail);

    account.store = map.try_to_vec()?;

    Ok(())
}

pub(crate) fn take_from_owner_store(
    account: &mut Account<'_, OwnerStore>,
    owner: Pubkey,
    target: Pubkey,
) -> Result<()> {
    let mut map = HashMap::<Pubkey, Vec<Pubkey>>::try_from_slice(account.store.as_slice())?;
    let record = map.entry(owner).or_insert_with(Vec::new);

    require!(record.contains(&target), SolX404Error::InsufficientNFT);

    record.retain(|x| x != &target);

    account.store = map.try_to_vec()?;

    Ok(())
}
