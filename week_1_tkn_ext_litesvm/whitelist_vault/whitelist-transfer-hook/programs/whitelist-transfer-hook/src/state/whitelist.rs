use anchor_lang::prelude::*;

#[account]
pub struct UserWhitelist {
    pub vault: Pubkey,
    pub is_whitelisted: bool,
    pub bump: u8,
}