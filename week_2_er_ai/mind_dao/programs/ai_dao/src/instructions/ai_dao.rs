use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_gpt_oracle::{ContextAccount, Counter, Identity};
use crate::states::ProposalAccount;
use crate::ID;

//
// ---------------------------
// AI Agent Account Structures
// ---------------------------
//

/// AI Agent Initialization Context
#[derive(Accounts)]
pub struct InitializeAiAgent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32,
        seeds = [b"ai-agent"],
        bump
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,

    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,

    /// CHECK: Checked oracle ID
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
}

/// Interaction with AI Agent
#[derive(Accounts)]
pub struct InteractWithAi<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    #[account(seeds = [b"ai-agent"], bump)]
    pub agent: Account<'info, Agent>,

    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: Checked oracle ID
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

/// Callback from Oracle → Smart Contract
#[derive(Accounts)]
pub struct CallbackFromAi<'info> {
    pub identity: Account<'info, Identity>,
}

// Implement the discriminator manually (to fix your compiler error)


#[account]
pub struct Agent {
    pub context: Pubkey,
}

impl<'info> InitializeAiAgent<'info> {
    pub fn handle(ctx: Context<Self>) -> Result<()> {
        const AGENT_DESC: &str = "You are the governance AI for MindDAO.
        Your job is to:
        - Summarize proposals clearly.
        - Evaluate if they are logical and applicable (0–10 score).
        - Judge voters’ reasoning quality when they explain their yes/no vote (0–10 score).";

        ctx.accounts.agent.context = ctx.accounts.llm_context.key();

        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::CreateLlmContext {
            payer: ctx.accounts.payer.to_account_info(),
            context_account: ctx.accounts.llm_context.to_account_info(),
            counter: ctx.accounts.counter.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solana_gpt_oracle::cpi::create_llm_context(cpi_ctx, AGENT_DESC.to_string())?;

        Ok(())
    }
}

impl<'info> InteractWithAi<'info> {
    /// Ask AI to summarize a proposal
    pub fn summarize_proposal(ctx: Context<Self>, title: String, description: String) -> Result<()> {
        let text = format!(
            "Summarize the following proposal and rate its applicability (0–10).
            Return JSON: {{\"summary\": <string>, \"applicability\": <int>}}.
            Title: {}. Description: {}.",
            title, description
        );
        Self::send_interaction(ctx, text)
    }

    /// Ask AI to evaluate a voter's reasoning
    pub fn evaluate_vote_reason(ctx: Context<Self>, decision: bool, reason: String) -> Result<()> {
        let text = format!(
            "A voter voted '{}' on a proposal because: '{}'.
            Evaluate if this is a strong, logical reason.
            Return JSON: {{\"score\": <int>, \"feedback\": <string>}}.",
            if decision { "YES" } else { "NO" },
            reason
        );
        Self::send_interaction(ctx, text)
    }

    /// Core CPI interaction logic
    fn send_interaction(ctx: Context<Self>, text: String) -> Result<()> {
        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: ctx.accounts.payer.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.context_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let disc: [u8; 8] = CallbackFromAi::DISCRIMINATOR.try_into().expect("Discriminator must be 8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, text, ID, disc, None)?;

        Ok(())
    }
}



#[error_code]
pub enum ErrorCode {
    #[msg("Invalid JSON")]
    InvalidJson,
    #[msg("Invalid creator")]
    InvalidCreator,
    #[msg("AI rejected proposal")]
    AiRejectedProposal,
}