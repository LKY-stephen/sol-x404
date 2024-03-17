use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum SolX404Instruction {
    /// Create My Account.
    /// A detailed description of the instruction.
    #[account(0, writable, name="State", desc = "The global state of x404 hub")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, name="system_program", desc = "The system program")]
    Init(Initiate),
    #[account(0, writable, name="State", desc = "The global state of x404 hub")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, name="system_program", desc = "The system program")]
    Deposit(Deposit),
    #[account(0, writable, name="State", desc = "The global state of x404 hub")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, name="system_program", desc = "The system program")]
    Reedem(Reedem),
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Initiate {
    /// The address of the new x404 token.
    pub owner: Pubkey,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Deposit {
    /// The address of the new x404 token.
    pub token: Pubkey,
    pub redeem_deadline: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Reedem {
    /// The address of the new x404 token.
    pub token: Pubkey,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Update {
    /// The address of the new x404 token.
    pub token: Pubkey,
}
