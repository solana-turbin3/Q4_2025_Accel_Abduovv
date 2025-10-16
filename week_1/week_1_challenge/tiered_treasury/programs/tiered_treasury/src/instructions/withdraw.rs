use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::spl_token_2022, token_interface::{Mint, TokenAccount, TokenInterface}
};

use spl_token_2022::onchain::invoke_transfer_checked;

use crate::states::{Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account( seeds = [b"config"], bump=config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mint::decimals = 9,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint=mint,
        associated_token::authority=user,
        associated_token::token_program=token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = config, associated_token::token_program=token_program)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Withdraw<'_> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {

        let seeds: &[&[u8]] = &[b"config", &[self.config.bump]];
        let signer_seeds: &[&[&[u8]]] = &[seeds];

        invoke_transfer_checked(
            &self.token_program.key(),
            self.vault.to_account_info(),
            self.mint.to_account_info(),
            self.user_ata.to_account_info(),
            self.config.to_account_info(),
            &[
            ],
            amount,
            self.mint.decimals,
            signer_seeds,
        )?;
        Ok(())
    }
}