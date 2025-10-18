use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_gpt_oracle::{ContextAccount, Counter, Identity};
use crate::Agent;
use crate::CallbackFromAi;
use crate::ID;

use crate::states::*;

#[derive(Accounts)]
#[instruction(id: u8)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [b"proposal", creator.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        space = 8 + ProposalAccount::INIT_SPACE,
    )]
    pub proposal_account: Account<'info, ProposalAccount>,
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

impl<'info> CreateProposal<'info> {
    pub fn create_process(&mut self, title: String, description: String, id: u8, bump: CreateProposalBumps) -> Result<()> {

      

        let cpi_program = self.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: self.creator.to_account_info(),
            interaction: self.interaction.to_account_info(),
            context_account: self.context_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let disc: [u8; 8] = CallbackFromAi::DISCRIMINATOR.try_into().expect("Discriminator must be 8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, text, ID, disc, None)?;

    

    Ok(())
}

}