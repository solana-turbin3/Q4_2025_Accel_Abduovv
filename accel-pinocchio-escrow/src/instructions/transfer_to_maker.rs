use pinocchio::{
    account_info::AccountInfo,
    pubkey,
    ProgramResult,
};

use crate::state::Escrow;

pub fn transfer_to_maker(accounts: &[AccountInfo]) -> ProgramResult {

     let [
        taker,
        maker,
        escrow_account,
        taker_ata_b,
        maker_ata_b,
        _associated_token_program,
        _rent_sysvar @ ..
    ] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    let (_escrow_account_pda, _bump) = pubkey::find_program_address(
        &[b"escrow".as_ref(), maker.key().as_slice()],
        &crate::ID
    );

    let escrow_account_state = Escrow::from_account_info(&escrow_account)?;
    let maker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_b)?;

    if escrow_account_state.maker() != *maker.key() && maker_ata_b_state.owner() != maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    if !escrow_account_state.amount_to_receive() == 0 {
        return Err(pinocchio::program_error::ProgramError::InsufficientFunds);
    }



    Ok(())
}
