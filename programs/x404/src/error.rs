use anchor_lang::prelude::*;

#[error_code]
pub enum SolX404Error {
    /// 0 - Sanity length check
    #[msg("Invalid Length")]
    InvalidLength,
    /// 1 - Factory permission error
    #[msg("Can only call by Factory")]
    OnlyCallByOwner,
    /// 2 - Not supported NFT
    #[msg("Target NFT is not blue chip NFT")]
    NotBlueChipNFT,
    /// 3 - Token type error
    #[msg("Corresponding X404 not created yet")]
    X404NotCreate,
    /// 4 - Address cannot be zero
    #[msg("Corresponding X404 not created yet")]
    CantBeZeroAddress,

    ///#[msg("Swap Factory Mismatch")]
    /// X404SwapV3FactoryMismatch,
    /// 5 - Not correct NFT owner
    #[msg("Shoule be sined by NFT address")]
    InvalidNFTAddress,
    /// 6 - Redeem dead line is invalid
    #[msg("Invalid Redeem DeadLine")]
    InvalidDeadLine,
    /// 7 - Redeem time error
    #[msg("NFT cannot be redeemed yet")]
    NFTCannotRedeem,
    /// 8 - Failed to remove token ids
    #[msg("Failed to redeem token")]
    RemoveFailed,
    /// 9 - Emergency Close
    #[msg("Current state is emergency closed")]
    EmergencyClose,
    /// 10 - redeem deadline error
    #[msg("Invalid redeem deadline")]
    InvaildRedeemDeadline,
    /// 11 - Insufficient fee for redeem
    #[msg("Insufficient fee for redeem")]
    InsufficientFee,
    /// 12 - Failed to send Sol token
    #[msg("Failed to send Sol token")]
    SendSolFailed,
    /// 13 - Reedem fee is too high
    #[msg("Redeem fee is too high")]
    RedeemFeeTooHigh,
    /// 14 - Reedem fee is too high
    #[msg("Collection is not verified yet")]
    NotVerifiedCollection,
    /// 15 - account generate failed
    #[msg("Failed to generate account")]
    FailedToGenerateAccount,
    /// 16 - mint close failed
    #[msg("Failed to close mint")]
    FailedToCloseMint,
    /// 17 - mint nft failed
    #[msg("Failed to mint nft")]
    FailedToMintNFT,
    /// 18 - nft already minted
    #[msg("NFT already minted")]
    NFTAlreadyMinted,
    /// 19 - invalid owner store
    #[msg("Invalid owner store")]
    InvalidOwnerStore,
    /// 20 - insufficient NFT
    #[msg("Insufficient NFT from")]
    InsufficientNFT,
    // 21 - only call by hooker
    #[msg("Only call by hooker")]
    OnlyCallByHooker,
    // 22 - only call by hooker
    #[msg("Invalid NFT Mint")]
    InvalidNFTMint,
    // 23 - Not a valid deposit user
    #[msg("InvalidDepositer")]
    InvalidDepositer,
}
