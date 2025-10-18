use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProposalAccount {
  pub creator: Pubkey,
  #[max_len(100)]
  pub title: String,
  #[max_len(100)]
  pub description: String,
  #[max_len(100)]
  pub ai_summary: String,
  pub clarity_score: u8,
  pub votes_yes: u64,
  pub votes_no: u64,
  pub bump: u8
}

#[account]
#[derive(InitSpace)]
pub struct VoteAccount {
  pub voter: Pubkey,
  proposal: Pubkey,
  pub vote_choice: bool,
  #[max_len(100)]
  pub reason: String,
  #[max_len(100)]
  pub ai_comment: String,
  pub reason_score: u8,
}


