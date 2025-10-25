use anchor_lang::prelude::*;

use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_and_undelegate_accounts};

use crate::state::Player;

#[commit]
#[derive(Accounts)]
pub struct Undelegate<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref(), player.dna.to_le_bytes().as_ref(), player.dna.to_le_bytes().as_ref()],
        bump = player.bump,
    )]
    pub player: Account<'info, Player>,
}

impl<'info> Undelegate<'info> {
    
    pub fn undelegate(&mut self) -> Result<()> {

        self.player.exit(&crate::ID)?;

        commit_and_undelegate_accounts(
            &self.user.to_account_info(), 
            vec![&self.player.to_account_info()], 
            &self.magic_context, 
            &self.magic_program
        )?;

        Ok(())
    }
}