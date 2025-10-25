use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::{self, log},
    ProgramResult,
};
use solana_program::sysvar::{Sysvar as SysvarClock, clock::Clock};

use crate::states::*;

pub fn process_refund(accounts: &[AccountInfo]) -> ProgramResult {

    let [contributer, creator, contribute_account,contribute_ata, mint_to_raise, fundraiser_account, vault, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    if !contributer.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    let contribute_seed = [Contribute::SEED.as_ref(), contributer.key().as_ref(), fundraiser_account.key().as_ref()];
    let (contribute_account_pda, bump_contribute) = pubkey::find_program_address(&contribute_seed, &crate::ID);
    assert_eq!(contribute_account_pda,*contribute_account.key());

    let seed = [Fundraiser::SEED.as_ref(), creator.key().as_ref()];
    let (fundraiser_account_pda, bump) = pubkey::find_program_address(&seed, &crate::ID);
    log(&fundraiser_account_pda);
    log(&fundraiser_account.key());
    assert_eq!(fundraiser_account_pda,*fundraiser_account.key());

    let fundraiser_state = Fundraiser::from_account_info(&fundraiser_account)?;
    let vault_state = pinocchio_token::state::TokenAccount::from_account_info(vault)?;
    let contribute_state = Contribute::from_account_info(&contribute_account)?;

    if fundraiser_state.mint_to_raise() != *mint_to_raise.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    let current_time = Clock::get().unwrap().unix_timestamp;
 
    assert!(fundraiser_state.duration() >= ((current_time - fundraiser_state.time_started()) / Fundraiser::SECONDS_TO_DAYS) as u8);
    assert!(vault_state.amount() < fundraiser_state.amount_to_raise());

    let initial_bump = bump_contribute.to_le();
    let bump: [u8; 1] = [initial_bump];
    let seed = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(creator.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault,
        to: contribute_ata,
        authority: contributer,
        amount: contribute_state.amount(),
    }
    .invoke_signed(&[seeds])?;

    fundraiser_state.set_amount_to_raise(fundraiser_state.amount_to_raise() - contribute_state.amount());

    let lamports_at_contribute_account = contribute_account.lamports();

    let mut contributer_lamports = contributer.try_borrow_mut_lamports()?;
    *contributer_lamports += lamports_at_contribute_account;

    {
        let mut contribute_acc_lamports = contribute_account.try_borrow_mut_lamports()?;
        *contribute_acc_lamports -= lamports_at_contribute_account;
    }

    fundraiser_account.close()?;
    Ok(())
}

