use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Player {
    pub user: Pubkey,
    pub class: u32,
    pub attack: u8,
    pub defense: u8,
    pub stamina: u8,
    pub dna: u8,
    pub bump: u8,
}

