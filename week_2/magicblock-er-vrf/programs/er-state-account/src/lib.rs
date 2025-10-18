#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;


mod state;
mod instructions;

use instructions::*;

declare_id!("7CW4YRL9j7fms4oNLV7Zd6uYjpF4p54oDJXRWFPsNHjY");
pub const CALLBACK_RAND_UPDATE_DISCRIMINATOR: [u8; 7] = *b"clbrand"; 

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    pub fn update(ctx: Context<UpdateUser>) -> Result<()> {
        ctx.accounts.update()?;
        Ok(())
    }

    pub fn update_commit(ctx: Context<UpdateCommit>) -> Result<()> {
        ctx.accounts.update_commit()?;
        
        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;
        
        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;
        
        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;
        
        Ok(())
    }

    #[instruction(discriminator = &CALLBACK_RAND_UPDATE_DISCRIMINATOR)]
    pub fn callback_rand_update(ctx: Context<CallbackRandUpdate>, randomness: [u8; 32]) -> Result<()> {
        ctx.accounts.callback(randomness)
    }
}

