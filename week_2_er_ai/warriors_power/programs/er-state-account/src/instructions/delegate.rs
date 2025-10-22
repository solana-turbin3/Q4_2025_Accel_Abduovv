use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

use crate::state::Player;

#[delegate]
#[derive(Accounts)]
pub struct Delegate<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        del,
        seeds = [b"user", user.key().as_ref(), player.dna.to_le_bytes().as_ref()],
        bump = player.bump,
    )]
    pub player: Account<'info, Player>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub validator: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Delegate<'info> {
    
    pub fn delegate(&mut self) -> Result<()> {

        let binding = self.player.dna.to_le_bytes();
        let pda_seeds: &[&[u8]] = &[
            b"user",
            self.user.key.as_ref(),
            binding.as_ref(),
            //&[self.player.bump],
        ];

        self.delegate_player(
            &self.user, 
            pda_seeds, 
            DelegateConfig {
                validator: Some(self.validator.key()),
                ..DelegateConfig::default()
            }
        )?;

        Ok(())
    }
}