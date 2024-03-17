use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum SolX404Error {
    /// 0 - Sanity length check
    #[error("Invalid Length")]
    InvalidLength,
    /// 1 - Factory permission error
    #[error("Can only call by Factory")]
    OnlyCallByFactory,
    /// 2 - Not supported NFT
    #[error("Target NFT is not blue chip NFT")]
    NotBlueChipNFT,
    /// 3 - Token type error
    #[error("Corresponding X404 not created yet")]
    X404NotCreate,
    /// 4 - Address cannot be zero
    #[error("Corresponding X404 not created yet")]
    CantBeZeroAddress,

    ///#[error("Swap Factory Mismatch")]
    /// X404SwapV3FactoryMismatch,

    /// 5 - Not correct NFT owner
    #[error("Shoule be sined by NFT address")]
    InvalidNFTAddress,
    /// 6 - Redeem dead line is invalid
    #[error("Invalid Redeem DeadLine")]
    InvalidDeadLine,
    /// 7 - Redeem time error
    #[error("NFT cannot be redeemed yet")]
    NFTCannotRedeem,
    /// 8 - Failed to remove token ids
    #[error("Failed to redeem token")]
    RemoveFailed,
    /// 9 - Emergency Close
    #[error("Current state is emergency closed")]
    EmergencyClose,
    /// 10 - max redeem deadline error
    #[error("Invalid max redeem deadline")]
    InvaildRedeemMaxDeadline,
    /// 11 - Insufficient fee for redeem
    #[error("Insufficient fee for redeem")]
    MsgValueNotEnough,
    /// 12 - Failed to send Sol token
    #[error("Failed to send Sol token")]
    SendSolFailed,
    /// 13 - Reedem fee is too high
    #[error("Redeem fee is too high")]
    RedeemFeeTooHigh,
}

impl PrintProgramError for SolX404Error {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<SolX404Error> for ProgramError {
    fn from(e: SolX404Error) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SolX404Error {
    fn type_of() -> &'static str {
        "Sol X404 Error"
    }
}
