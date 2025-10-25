use anchor_lang::prelude::*;


#[error_code]
pub enum CustomError {
    #[msg("Owner is not whitelisted")]
    NotWhitelisted,
    #[msg("Not called during transfer")]
    NotTransferring
}

