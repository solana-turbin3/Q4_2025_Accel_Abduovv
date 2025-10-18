use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub user: Pubkey,
    pub random_number: u64,
    pub bump: u8,
}

