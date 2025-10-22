use anchor_lang::prelude::*;

use crate::state::Player;

#[derive(Accounts)]
#[instruction(dna: u8)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + Player::INIT_SPACE,
        seeds = [b"user", user.key().as_ref(), dna.to_le_bytes().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitUser<'info> {
    pub fn initialize(&mut self, dna: u8, bumps: &InitUserBumps) -> Result<()> {
        self.player.set_inner(Player {
            user: *self.user.key,
            class: 0,
            attack: 0,
            defense: 0,
            stamina: 0,
            dna,
            bump: bumps.player,
        });

        Ok(())
    }
}
