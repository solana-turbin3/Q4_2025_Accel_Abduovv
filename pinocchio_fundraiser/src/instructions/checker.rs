use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::{self, log},
    ProgramResult,
};

use crate::states::*;


pub fn process_checker(accounts: &[AccountInfo]) -> ProgramResult {

    let [ creator, creator_ata, fundraiser_account, vault, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let vault_state = pinocchio_token::state::TokenAccount::from_account_info(vault)?;

    let seed = [Fundraiser::SEED.as_ref(), creator.key().as_ref()];
    let (fundraiser_account_pda, bump) = pubkey::find_program_address(&seed, &crate::ID);
    log(&fundraiser_account_pda);
    log(&fundraiser_account.key());
    assert_eq!(fundraiser_account_pda,*fundraiser_account.key());

    let fundraiser_state = Fundraiser::from_account_info(&fundraiser_account)?;

    if vault_state.amount() >= fundraiser_state.amount_to_raise() {
        return Err(pinocchio::program_error::ProgramError::InsufficientFunds);
    }

    let initial_bump = bump.to_le();
    let bump = [initial_bump];
    let seed = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(creator.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);
    let close_seed = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault,
        to: creator_ata,
        authority: fundraiser_account,
        amount: vault_state.amount(),
    }
    .invoke_signed(&[seeds])?;

   
    pinocchio_token::instructions::CloseAccount {
        account: vault,
        authority: fundraiser_account,
        destination: creator,
    }
    .invoke_signed(&[close_seed])?;

    let lamports_at_escrow = fundraiser_account.lamports();

    let mut creator_lamports = creator.try_borrow_mut_lamports()?;
    *creator_lamports += lamports_at_escrow;

    {
        let mut fundraiser_acc_lamports = fundraiser_account.try_borrow_mut_lamports()?;
        *fundraiser_acc_lamports -= lamports_at_escrow;
    }

    fundraiser_account.close()?;
    Ok(())
}

