use anchor_lang::prelude::*;

use crate::state::whitelist::Whitelist;

#[derive(Accounts)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"whitelist", user.key().as_ref()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    /// CHECK:
    pub user: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistOperations<'info> {
    pub fn switch_whitelist(&mut self) -> Result<()> {
        if self.whitelist.is_whitelisted {
            self.whitelist.is_whitelisted = false;
        } else {
            self.whitelist.is_whitelisted = true;
        }
        Ok(())
    }

    

    
}