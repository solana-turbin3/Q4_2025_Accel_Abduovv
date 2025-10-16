use anchor_lang::prelude::*;
use crate::state::UserAccount;

#[derive(Accounts)]
pub struct CallbackRandUpdate<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,

    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}


impl<'info> CallbackRandUpdate<'info> {
    pub fn callback_roll_dice(
    &mut self,
    randomness: [u8; 32],
) -> Result<()> {
    let rnd_u64 = ephemeral_vrf_sdk::rnd::random_u64(&randomness);
    msg!("Consuming random number: {:?}", rnd_u64);
    let user_account = &mut self.user_account;
    user_account.data = rnd_u64;
    
    Ok(())
}

}


