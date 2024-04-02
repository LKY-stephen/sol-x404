use anchor_lang::{
    prelude::*,
    system_program::{self, create_account, CreateAccount}, InstructionData,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount},
};
use solana_program::instruction::Instruction;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::{ExecuteInstruction, TransferHookInstruction};
use x404::state::{OwnerStore, X404State};

// transfer-hook program that charges a SOL fee on token transfer
// use a delegate and wrapped SOL because signers from initial transfer are not accessible

declare_id!("6uCDZftnA5YaTmqv3PnGSQSomFpnrtvkrk2xQSHrvNgh");

#[program]
#[cfg(not(feature = "no-entrypoint"))]
pub mod transfer_hook {

    use solana_program::program::invoke_signed;
    use x404::instructions::rebalance;

    use super::*;

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        // index 0-3 are the accounts required for token transfer (source, mint, destination, owner=mint)
        // index 4 is address of ExtraAccountMetaList account
        let account_metas = vec![
            // index 5, 404 state
            ExtraAccountMeta::new_with_pubkey(&ctx.accounts.state.key(), false, false)?,
            // index 6, 404 owner store
            ExtraAccountMeta::new_with_pubkey(&ctx.accounts.owner_store.key(), false, false)?,
            // index 7, associated token program
            ExtraAccountMeta::new_with_pubkey(
                &ctx.accounts.associated_token_program.key(),
                false,
                false,
            )?,
        ];

        // calculate account size
        let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64;
        // calculate minimum required lamports
        let lamports = Rent::get()?.minimum_balance(account_size as usize);

        let mint = ctx.accounts.mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"extra-account-metas",
            &mint.as_ref(),
            &[ctx.bumps.extra_account_meta_list],
        ]];

        // create ExtraAccountMetaList account
        create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.extra_account_meta_list.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            lamports,
            account_size,
            ctx.program_id,
        )?;

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &account_metas,
        )?;

        Ok(())
    }

    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
       let mint_key = ctx.accounts.mint.key().clone();
       let signer_seeds: &[&[&[u8]]] = &[&[b"extra-account-metas",mint_key.as_ref(), &[ctx.bumps.extra_account_meta_list]]];
       msg!("Rebalance the state");

        // transfer WSOL from sender to delegate token account using delegate PDA
        
        let instruction = rebalance(
            ctx.accounts.state.key(),
            ctx.accounts.owner_store.key(),
            ctx.accounts.source_token.owner.key(),
            ctx.accounts.destination_token.owner.key(),
            amount,
            mint_key,
            ctx.accounts.source_token.key(),
            ctx.accounts.destination_token.key(),
            ctx.accounts.extra_account_meta_list.key());

         invoke_signed(&instruction, 
            &[
            ctx.accounts.state.to_account_info(),
            ctx.accounts.owner_store.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.source_token.to_account_info(),
            ctx.accounts.destination_token.to_account_info(),
            ctx.accounts.extra_account_meta_list.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(), ],
            signer_seeds)?;

        Ok(())
    }

    // fallback instruction handler as workaround to anchor instruction discriminator check
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;

        // match instruction discriminator to transfer hook interface execute instruction  
        // token2022 program CPIs this instruction on token transfer
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount_bytes = amount.to_le_bytes();

                // invoke custom transfer hook instruction on our program
                __private::__global::transfer_hook(program_id, accounts, &amount_bytes)
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
    }
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub state: Account<'info, X404State>,
    pub owner_store: Account<'info, OwnerStore>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Order of accounts matters for this struct.
// The first 4 accounts are the accounts required for token transfer (source, mint, destination, owner)
// Remaining accounts are the extra accounts required from the ExtraAccountMetaList account
// These accounts are provided via CPI to this program from the token2022 program
#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint, 
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    pub owner: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    pub state: Account<'info, X404State>,
    #[account(mut)]
    pub owner_store: Account<'info, OwnerStore>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn initialize_extra_account(
    extra_account: Pubkey,
    fungible_mint: Pubkey,
    owner: Pubkey,
    x404_state: Pubkey,
    owner_store: Pubkey,
)-> Instruction{
    let data = instruction::InitializeExtraAccountMetaList {
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new(extra_account, false),
            AccountMeta::new_readonly(fungible_mint, false),
            AccountMeta::new_readonly(x404_state, false),
            AccountMeta::new_readonly(owner_store, false),
            AccountMeta::new_readonly(AssociatedToken::id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}