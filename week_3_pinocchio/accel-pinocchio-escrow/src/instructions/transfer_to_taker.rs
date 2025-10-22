use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey, ProgramResult
};

use crate::state::Escrow;

pub fn transfer_to_taker(accounts: &[AccountInfo]) -> ProgramResult {

     let [
        taker,
        maker,
        escrow_account,
        taker_ata_a,
        escrow_ata,
        _associated_token_program,
        _rent_sysvar @ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if taker_ata_a.owner() != taker.key() {
        return Err(ProgramError::IllegalOwner);
    }

    let (_escrow_account_pda, bump) = pubkey::find_program_address(
        &[b"escrow".as_ref(), maker.key().as_slice()],
        &crate::ID
    );

    let escrow_account_state = Escrow::from_account_info(&escrow_account)?;

    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.key()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: taker_ata_a,
        authority: escrow_account,
        amount: escrow_account_state.amount_to_give(),
    }.invoke_signed(&[seeds])?;

    escrow_account_state.set_amount_to_give(0);

    Ok(())
}
