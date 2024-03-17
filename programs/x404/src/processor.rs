use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::error::SolX404Error;
use crate::instruction::{Deposit, Initiate, Reedem, SolX404Instruction};
use crate::state::*;

pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: SolX404Instruction = SolX404Instruction::try_from_slice(instruction_data)?;
    match instruction {
        SolX404Instruction::Init(args) => {
            msg!("Instruction: Create");
            initiate(accounts, args)
        }
        SolX404Instruction::Deposit(args) => {
            msg!("Instruction: Deposit");
            Deposit(accounts, args)
        }
        SolX404Instruction::Reedem(args) => {
            msg!("Instruction: Reedem");
            Redeem(accounts, args)
        }
    }
}

fn initiate<'a>(accounts: &'a [AccountInfo<'a>], args: Initiate) -> ProgramResult {
    //!todo()
    Ok(())
}
fn Deposit<'a>(accounts: &'a [AccountInfo<'a>], args: Deposit) -> ProgramResult {
    //!todo()
    Ok(())
}
fn Redeem<'a>(accounts: &'a [AccountInfo<'a>], args: Reedem) -> ProgramResult {
    //!todo()
    Ok(())
}
