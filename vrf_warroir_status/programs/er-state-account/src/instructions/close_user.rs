use crate::state::Player;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        close = user,
        seeds = [b"user", user.key().as_ref(), player.dna.to_le_bytes().as_ref()],
        bump = player.bump,
    )]
    pub player: Box<Account<'info, Player>>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseUser<'info> {
    pub fn close(&mut self) -> Result<()> {
        // All the closing logic is handled by the `close` constraint in the Accounts struct
        Ok(())
    }
}
