use anchor_lang::{ 
    prelude::*, 
};
use anchor_spl::token_interface::{
    Mint, 
    TokenInterface,
};

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    // address: 
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 9,
        mint::authority = admin,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self) -> Result<()> {
        Ok(())
    }
}