use anchor_lang::prelude::*;

#[error_code]
pub enum DlmmError {
    #[msg("Invalid bin range: lower must be <= upper")] 
    InvalidBinRange,
}


