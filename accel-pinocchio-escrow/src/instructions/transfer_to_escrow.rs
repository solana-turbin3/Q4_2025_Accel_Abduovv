use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, ProgramResult
};

use crate::state::Escrow;

pub fn transfer_to_escrow(accounts: &[AccountInfo]) -> ProgramResult {

    let [
        maker,
        maker_ata_a,
        escrow_account,
        escrow_ata,
        _associated_token_program,
        _rent_sysvar @ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let escrow_state = Escrow::from_account_info(escrow_account)?;

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if escrow_state.maker() != *maker.key() {
        return Err(ProgramError::IllegalOwner);
    }

    let escrow_ata_state = pinocchio_token::state::TokenAccount::from_account_info(escrow_ata)?;

    if escrow_ata_state.amount() >= escrow_state.amount_to_give() {
        return Err(ProgramError::InsufficientFunds);
    }

    // Transfer maker tokens into escrow
    pinocchio_token::instructions::Transfer {
        from: maker_ata_a,
        to: escrow_ata,
        authority: maker,
        amount: escrow_state.amount_to_give(),
    }.invoke()?;

    Ok(())
}
