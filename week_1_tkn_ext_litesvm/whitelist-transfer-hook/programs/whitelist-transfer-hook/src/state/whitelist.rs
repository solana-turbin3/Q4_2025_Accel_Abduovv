use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub is_whitelisted: bool,
    pub bump: u8,
}