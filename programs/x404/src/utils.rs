use anchor_lang::{
    context::CpiContext,
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::{
    token::{transfer_checked as spl_transfer_checked, TransferChecked as SPLTransferChecked},
    token_2022::{
        mint_to, set_authority, spl_token_2022::instruction::AuthorityType, MintTo, SetAuthority,
    },
    token_interface::Mint,
};

use crate::{error::SolX404Error, OwnerStore};

pub(crate) fn mint_nft<'info>(
    token_program: AccountInfo<'info>,
    nft_mint: InterfaceAccount<'info, Mint>,
    to: AccountInfo<'info>,
    current_owner: AccountInfo<'info>,
    nft_signer: &[&[&[u8]]],
) -> Result<()> {
    if nft_mint.supply != 0 {
        return err!(SolX404Error::NFTAlreadyMinted);
    }

    if mint_token(
        token_program.clone(),
        nft_mint.to_account_info(),
        to.clone(),
        1,
        current_owner.clone(),
        nft_signer,
    )
    .is_err()
    {
        return err!(SolX404Error::FailedToMintNFT);
    }

    msg!("Close NFT Account.");
    let close_context = CpiContext::new_with_signer(
        token_program,
        SetAuthority {
            account_or_mint: nft_mint.to_account_info(),
            current_authority: current_owner,
        },
        nft_signer,
    );

    if set_authority(close_context, AuthorityType::MintTokens, None).is_err() {
        return err!(SolX404Error::FailedToCloseMint);
    }
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
            to: to,
            authority: mint_authority.clone(),
        },
        authority_seeds,
    );

    mint_to(cpi_context, amount)
}

// pub(crate) fn transfer_token<'info>(
//     token_program: AccountInfo<'info>,
//     mint: AccountInfo<'info>,
//     from: AccountInfo<'info>,
//     to: AccountInfo<'info>,
//     authority: AccountInfo<'info>,
//     amount: u64,
//     decimal: u8,
//     authority_seeds: Option<&[&[&[u8]]]>,
// ) -> Result<()> {
//     let cpi_context = CpiContext::new_with_signer(
//         token_program.clone(),
//         TransferChecked {
//             mint: mint,
//             from: from,
//             to: to,
//             authority: authority,
//         },
//         authority_seeds.unwrap_or(&[]),
//     );

//     transfer_checked(cpi_context, amount, decimal)
// }

pub(crate) fn transfer_spl_token<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    decimal: u8,
) -> Result<()> {
    let cpi_context = CpiContext::new(
        token_program.clone(),
        SPLTransferChecked {
            mint: mint,
            from: from,
            to: to,
            authority: authority,
        },
    );

    spl_transfer_checked(cpi_context, amount, decimal)
}

pub(crate) fn initiate_account<'info>(
    progarm: AccountInfo<'info>,
    account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Result<()> {
    let cpi_context = CpiContext::new(
        progarm,
        CreateAccount {
            from: payer,
            to: account,
        },
    );

    create_account(cpi_context, lamports, space, owner)
}

pub(crate) fn initiate_owner_store<'info>(
    account: UncheckedAccount<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    rent: Rent,
    target: Pubkey,
) -> Result<()> {
    initiate_account(
        system_program,
        account.to_account_info(),
        payer,
        rent.minimum_balance(8 + 4 + 32),
        8 + 4 + 32,
        &owner.key(),
    )?;

    let new_store = OwnerStore {
        store: vec![target],
    };

    let mut buf = &mut account.data.try_borrow_mut().unwrap()[..];

    new_store.serialize(&mut buf)?;

    Ok(())
}

pub(crate) fn add_to_owner_store<'info>(
    account: UncheckedAccount<'info>,
    rent: Rent,
    target: Pubkey,
) -> Result<()> {
    account.add_lamports(rent.minimum_balance(32))?;
    let mut buf = &account.data.try_borrow_mut().unwrap()[..];

    let mut new_store = OwnerStore::try_deserialize(&mut buf)?;

    new_store.store.push(target);

    let mut buf = &mut account.data.try_borrow_mut().unwrap()[..];

    new_store.serialize(&mut buf)?;

    Ok(())
}
