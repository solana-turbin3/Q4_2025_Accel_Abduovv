#[allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_gpt_oracle::{ContextAccount, Counter, Identity};
use serde_json::Value;

declare_id!("2csGAx9iytPe4EzgecgHB6tuDMAkCwU2GnmRFjarhnxr");

#[program]
pub mod mind_dao {
    use super::*;

    const AGENT_DESC: &str = "You are a governance AI for MindDAO.
- Summarize and rate proposals (0–10).
- Evaluate vote reasoning (0–10).";

    // Initialize LLM context
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
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

    /// Create proposal: store a draft and send text to the oracle for evaluation
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        id: u8,
        title: String,
        description: String,
    ) -> Result<()> {
        // 1) initialize draft proposal (placeholder)
        ctx.accounts.proposal_account.set_inner(ProposalAccount {
            creator: ctx.accounts.creator.key(),
            id,
            title: title.clone(),
            description: description.clone(),
            ai_summary: "Pending AI review...".to_string(),
            clarity_score: 0,
            votes_yes: 0,
            votes_no: 0,
            status: Status::Pending,
            bump: ctx.bumps.proposal_account,
        });

        let text = format!(
            r#"
You are an AI assistant evaluating a DAO governance proposal.

TASKS:
1) Provide a concise summary (1-2 sentences).
2) Rate applicability from 0–10 (consider clarity, feasibility, impact, and cost).
3) Provide a short reason if low or high quality (one sentence).

Return a JSON object exactly in this format:
{{
  "summary": "<string>",
  "applicability": <int>,
  "risk_reasoning": "<string>"
}}

Proposal:
Title: {}
Description: {}
"#,
            title, description
        );

        // 3) Prepare CPI to oracle
        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: ctx.accounts.creator.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.context_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // 4) Use proposal-specific callback discriminator
        let disc: [u8; 8] = CALLBACK_PROPOSAL_DISCRIM.try_into().expect("8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, text, crate::ID, disc, None)?;
        Ok(())
    }

    /// Vote: create vote account and send reason to AI for scoring
    pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, vote_choice: bool, reason: String) -> Result<()> {
        // 1) initialize vote account (store reason; score pending)
        ctx.accounts.vote.set_inner(VoteAccount {
            voter: ctx.accounts.voter.key(),
            proposal: ctx.accounts.proposal_account.key(),
            vote_choice,
            reason: reason.clone(),
            ai_comment: "Pending AI review...".to_string(),
            reason_score: 0,
            bump: ctx.bumps.vote,
        });

        // 2) Build vote-evaluation prompt — more focused and constrained
        let text = format!(
            r#"
You are an AI evaluating the quality of a voter's reasoning in a DAO governance vote.

TASKS:
1) Evaluate how logically connected the voter's reason is to the proposal (0–10).
2) Return a short feedback sentence explaining the assessment.

Return JSON exactly:
{{
  "score": <int>,
  "comment": "<string>"
}}

Voter Reason: {}
Vote Choice: {}
Proposal Title: {}
Proposal Description: {}
"#,
            reason,
            if vote_choice { "YES" } else { "NO" },
            ctx.accounts.proposal_account.title,
            ctx.accounts.proposal_account.description
        );

        // 3) Prepare CPI
        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: ctx.accounts.voter.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.context_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // 4) Use vote-specific callback discriminator
        let disc: [u8; 8] = CALLBACK_VOTE_DISCRIM.try_into().expect("8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, text, crate::ID, disc, None)?;
        Ok(())
    }

    // ============================
    // Callbacks
    // ============================
    /// Callback for proposal analysis: update proposal if AI approves
    pub fn callback_for_proposal(ctx: Context<CallbackForProposal>, response: String) -> Result<()> {
        // ensure the oracle program signed this callback
        require!(
            ctx.accounts.identity.to_account_info().is_signer,
            CustomError::UnauthorizedCallback
        );

        msg!("AI callback (proposal): {}", response);

        // parse JSON
        let parsed: Value = serde_json::from_str(&response).map_err(|_| error!(CustomError::InvalidJson))?;

        let summary = parsed["summary"].as_str().unwrap_or("No summary").to_string();
        let applicability = parsed["applicability"].as_i64().unwrap_or(0) as i64;

        let prop = &mut ctx.accounts.proposal_account;

        // if AI rejects
        if applicability < 5 {
            msg!("AI rejected proposal (score {})", applicability);
            return Err(error!(CustomError::AiRejectedProposal));
        }

        // update proposal
        prop.ai_summary = summary;
        prop.clarity_score = applicability as u8;

        msg!("Proposal accepted: score {}", prop.clarity_score);
        Ok(())
    }

    /// Callback for vote analysis: update vote account and increment proposal counters
    pub fn callback_for_vote(ctx: Context<CallbackForVote>, response: String) -> Result<()> {
        require!(
            ctx.accounts.identity.to_account_info().is_signer,
            CustomError::UnauthorizedCallback
        );

        msg!("AI callback (vote): {}", response);

        let parsed: Value = serde_json::from_str(&response).map_err(|_| error!(CustomError::InvalidJson))?;
        let score = parsed["score"].as_i64().unwrap_or(0) as u8;
        let comment = parsed["comment"].as_str().unwrap_or("").to_string();

        // update vote account
        let vote_acc = &mut ctx.accounts.vote;
        vote_acc.reason_score = score;
        vote_acc.ai_comment = comment.clone();

        // update proposal counters
        let prop = &mut ctx.accounts.proposal_account;
        if vote_acc.vote_choice {
            prop.votes_yes = prop.votes_yes.checked_add(1).unwrap_or(prop.votes_yes);
        } else {
            prop.votes_no = prop.votes_no.checked_add(1).unwrap_or(prop.votes_no);
        }

        msg!("Vote scored {}. Proposal now yes:{} no:{}", score, prop.votes_yes, prop.votes_no);
        Ok(())
    }
}

// =====================================
// Accounts - Initialization & PDAs
// =====================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init, payer = payer, space = 8 + 32, seeds = [b"ai-agent"], bump)]
    pub agent: Account<'info, Agent>,

    /// CHECK - LLM context account (oracle)
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,

    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,

    /// CHECK - oracle program verified by address
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(id: u8)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(init, payer = creator, space = 8 + ProposalAccount::INIT_SPACE,
        seeds = [b"proposal", creator.key().as_ref(), id.to_le_bytes().as_ref()], bump)]
    pub proposal_account: Account<'info, ProposalAccount>,

    /// CHECK: oracle interaction account
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    #[account(seeds = [b"ai-agent"], bump)]
    pub agent: Account<'info, Agent>,

    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: oracle program
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(mut)]
    pub proposal_account: Account<'info, ProposalAccount>,

    #[account(init, payer = voter, space = 8 + VoteAccount::INIT_SPACE,
        seeds = [b"vote", voter.key().as_ref(), proposal_account.key().as_ref()], bump)]
    pub vote: Account<'info, VoteAccount>,

    /// CHECK: oracle interaction account
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    #[account(seeds = [b"ai-agent"], bump)]
    pub agent: Account<'info, Agent>,

    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: oracle program
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

// Callback accounts for proposal
#[derive(Accounts)]
pub struct CallbackForProposal<'info> {
    /// CHECK: oracle identity account
    pub identity: Account<'info, Identity>,

    /// proposal to update
    #[account(mut)]
    pub proposal_account: Account<'info, ProposalAccount>,
}

// Callback accounts for vote
#[derive(Accounts)]
pub struct CallbackForVote<'info> {
    /// CHECK: oracle identity account
    pub identity: Account<'info, Identity>,

    /// vote account to update
    #[account(mut)]
    pub vote: Account<'info, VoteAccount>,

    /// proposal to update counts
    #[account(mut)]
    pub proposal_account: Account<'info, ProposalAccount>,
}



// =====================================
// Data accounts
// =====================================

#[account]
pub struct Agent {
    pub context: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct ProposalAccount {
    pub creator: Pubkey,
    pub id: u8,
    #[max_len(100)]
    pub title: String,
    #[max_len(300)]
    pub description: String,
    #[max_len(512)]
    pub ai_summary: String,
    pub clarity_score: u8,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub status: Status,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct VoteAccount {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_choice: bool,
    #[max_len(200)]
    pub reason: String,
    #[max_len(200)]
    pub ai_comment: String,
    pub reason_score: u8,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum Status {
    Pending,
    Accepted,
    Rejected,
}



// =====================================
// Discriminators and constants
// =====================================

// We use two different discriminator byte strings for proposals and votes.
// These must be 8 bytes long.
const CALLBACK_PROPOSAL_DISCRIM: &[u8] = b"AI_P_CB1";
const CALLBACK_VOTE_DISCRIM: &[u8] = b"AI_V_CB1";

// Implement Discriminator trait for Anchor derive compatibility
impl Discriminator for CallbackForProposal<'_> {
    const DISCRIMINATOR: &'static [u8] = CALLBACK_PROPOSAL_DISCRIM;
}
impl Discriminator for CallbackForVote<'_> {
    const DISCRIMINATOR: &'static [u8] = CALLBACK_VOTE_DISCRIM;
}

// =====================================
// Errors
// =====================================

#[error_code]
pub enum CustomError {
    #[msg("Invalid JSON structure from AI")]
    InvalidJson,
    #[msg("Unauthorized oracle callback")]
    UnauthorizedCallback,
    #[msg("Proposal rejected by AI (low clarity score)")]
    AiRejectedProposal,
}
