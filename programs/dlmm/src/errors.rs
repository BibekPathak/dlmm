use anchor_lang::prelude::*;

#[error_code]
pub enum DlmmError {
    #[msg("Invalid bin range: lower must be <= upper")]
    InvalidBinRange,
    #[msg("Bin ID out of bounds")]
    BinIdOutOfBounds,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Not implemented")]
    NotImplemented,
    #[msg("Slippage limit exceeded")]
    SlippageExceeded,
    #[msg("Insufficient liquidity in bin")]
    InsufficientLiquidity,
    #[msg("Bin array not found for this bin ID")]
    BinArrayNotFound,
    #[msg("Bin array already initialized")]
    BinArrayAlreadyInitialized,
    #[msg("Position not owned by signer")]
    InvalidPositionOwner,
    #[msg("Minimum output not met")]
    MinOutputNotMet,
    #[msg("Maximum input exceeded")]
    MaxInputExceeded,
    #[msg("Price limit reached")]
    PriceLimitReached,
    #[msg("Duplicate bin ID in deposits")]
    DuplicateBinId,
    #[msg("Deposit amounts must not be zero")]
    ZeroDeposit,
    #[msg("Invalid fee configuration")]
    InvalidFeeConfig,
    #[msg("Bin array start ID must be aligned to BINS_PER_ARRAY boundary")]
    InvalidBinArrayStart,
}
