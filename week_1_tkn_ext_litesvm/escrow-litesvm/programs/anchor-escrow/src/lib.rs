#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod state;
mod instructions;
mod tests;

use instructions::*;

declare_id!("2BLor9eJ7qC9NmEBjKQ7W4mBgMbQChgnp3wqPWQMzvNm");

#[program]
pub mod anchor_escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64, open_in: i64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, open_in, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }
}