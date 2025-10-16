use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_accounts};
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;
use crate::CALLBACK_RAND_UPDATE_DISCRIMINATOR;
use crate::state::UserAccount;

#[vrf]
#[commit]
#[derive(Accounts)]
pub struct UpdateCommit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: Oracle queue
    #[account(
        mut, 
        address = ephemeral_vrf_sdk::consts::DEFAULT_QUEUE
    )]
    pub oracle_queue: AccountInfo<'info>,
}

impl<'info>UpdateCommit<'info> {
    pub fn update_commit(&self) -> Result<()> {

        msg!("Requesting randomness...");
        let slot = Clock::get()?.slot;
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&slot.to_le_bytes());
        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            callback_discriminator: CALLBACK_RAND_UPDATE_DISCRIMINATOR.to_vec(),
            caller_seed: seed_bytes,
            // Specify any account that is required by the callback
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });
        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;

        commit_accounts(
            &self.user.to_account_info(), 
            vec![&self.user_account.to_account_info()], 
            &self.magic_context, 
            &self.magic_program
        )?;
        
        Ok(())
    }
}