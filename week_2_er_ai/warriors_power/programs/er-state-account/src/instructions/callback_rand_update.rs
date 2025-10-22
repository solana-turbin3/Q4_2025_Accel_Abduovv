use anchor_lang::prelude::*;
use crate::state::Player;

#[derive(Accounts)]
pub struct CallbackRandUpdate<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,

    #[account(mut)]
    pub player: Account<'info, Player>,
}

impl<'info> CallbackRandUpdate<'info> {
    pub fn callback(&mut self, randomness: [u8; 32]) -> Result<()> {
        let roll = ephemeral_vrf_sdk::rnd::random_u8_with_range(&randomness, 1, 101);
        msg!("Roll is {}", roll);

        let (class, class_name) = if roll <= 32 {
            (0, "Warrior")
        } else if roll <= 64 {
            (1, "Mage")
        } else if roll <= 96 {
            (2, "Archer")
        } else {
            (3, "Priest")
        };

        let stats_roll = ephemeral_vrf_sdk::rnd::random_u32(&randomness);
        msg!("Stats Roll is {}", stats_roll);

        let attack = ((stats_roll >> 0) & 0x3FF) % 100;
        let defense = ((stats_roll >> 8) & 0x3FF) % 100;
        let stamina = ((stats_roll >> 16) & 0x3FF) % 100;

        msg!(
            "Class: {}, ATK: {}, DEF: {}, STA: {}",
            class_name,
            attack,
            defense,
            stamina
        );

        self.player.class = class;
        self.player.attack = attack as u8;
        self.player.defense = defense as u8;
        self.player.stamina = stamina as u8;

        Ok(())
    }
}
