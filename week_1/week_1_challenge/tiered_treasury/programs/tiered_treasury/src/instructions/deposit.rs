use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::spl_token_2022, token_interface::{Mint, TokenAccount, TokenInterface}
};

use spl_token_2022::onchain::invoke_transfer_checked;
use crate::states::Config;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account( seeds = [b"config"], bump=config.bump)]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint=mint,
        associated_token::authority=user,
        associated_token::token_program=token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config,
        associated_token::token_program=token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetalist Account
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetalist Account
    pub whitelist: UncheckedAccount<'info>,
    /// CHECK: this will be the program created for the whitelist tf hook
    pub transfer_hook_program: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Deposit<'_> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {

        invoke_transfer_checked(
            &self.token_program.key(),
            self.user_ata.to_account_info(),
            self.mint.to_account_info(),
            self.vault.to_account_info(),
            self.user.to_account_info(),
            &[
                self.extra_account_meta_list.to_account_info(),
                self.whitelist.to_account_info(),
                self.transfer_hook_program.to_account_info(),
            ],
            amount,
            self.mint.decimals,
            &[],
        )?;
        Ok(())
    }
}