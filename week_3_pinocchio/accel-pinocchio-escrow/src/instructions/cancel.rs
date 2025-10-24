use pinocchio::{
    pubkey, account_info::AccountInfo, instruction::{Seed, Signer}, msg, pubkey::log, ProgramResult
};
use crate::state::Escrow;

pub fn process_cancel_instruction(
    accounts: &[AccountInfo],
) -> ProgramResult {

    msg!("Processing Cancel instruction");

     let [
        maker,
        mint_a,
        escrow_account,
        maker_ata_a,
        escrow_ata,
        _associated_token_program,
    ] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let maker_ata_a_state = pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_a)?;

    if maker_ata_a_state.owner() != maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }
    if maker_ata_a_state.mint() != mint_a.key() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }

    let (escrow_account_pda, bump) = pubkey::find_program_address(
        &[b"escrow".as_ref(), maker.key().as_slice()],
        &crate::ID
    );

    log(&escrow_account_pda);
    log(&escrow_account.key());
    assert_eq!(escrow_account_pda, *escrow_account.key());
 
    let escrow_state = Escrow::from_account_info(&escrow_account)?;

    if escrow_state.maker() != *maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }
    if escrow_state.mint_a() != *mint_a.key() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }

    let escrow_ata_state = pinocchio_token::state::TokenAccount::from_account_info(&escrow_ata)?;

    if escrow_state.amount_to_give() > escrow_ata_state.amount() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }

    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.key()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);
    
    
    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata_a,
        authority: escrow_account,
        amount: escrow_state.amount_to_give(),
    }.invoke_signed(&[seeds])?;

    escrow_state.set_amount_to_give(0);

    Ok(())
}