#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod instructions;
mod state;
mod errors;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::{
    instruction::{ExecuteInstruction, InitializeExtraAccountMetaListInstruction},
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("8LUi3qDiD2KLstBYyjJcPQuS8rpD8DGjTwDyAAUGpUwB");

#[program]
pub mod whitelist_transfer_hook {
    use super::*;

    /// Admin initializes whitelist for a user
    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        ctx.accounts.initialize_whitelist(ctx.bumps)
    }

    pub fn mint_token(ctx: Context<TokenFactory>) -> Result<()> {
        ctx.accounts.init_mint()
    }

    /// Admin toggles whitelist (true <-> false)
    pub fn switch_whitelist(ctx: Context<WhitelistOperations>) -> Result<()> {
        ctx.accounts.switch_whitelist()
    }

    /// Initialize the Transfer Hook (ExtraAccountMetaList)
    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;
        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());

        // Initialize ExtraAccountMetaList account with the extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    /// Called automatically when transfer hook executes
    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }
}
