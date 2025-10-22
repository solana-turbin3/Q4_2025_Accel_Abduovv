use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::{self, log}, ProgramResult};

use crate::state::Escrow;

pub fn process_take_instruction(
    accounts: &[AccountInfo],
) -> ProgramResult {

    msg!("Processing Take instruction");

    let [
        taker,
        maker,
        mint_a,
        mint_b,
        escrow_account,
        taker_ata_b,
        taker_ata_a,
        maker_ata_b,
        system_program,
        token_program,
        _associated_token_program,
        _rent_sysvar @ ..
    ] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let taker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_b)?;
    let taker_ata_a_state = pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_a)?;
    let maker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_b)?;

    if taker_ata_b_state.owner() != taker.key() && taker_ata_a_state.owner() != taker.key() && maker_ata_b_state.owner() != maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }
    if taker_ata_b_state.mint() != mint_b.key() && taker_ata_a_state.mint() != mint_a.key() && maker_ata_b_state.mint() != mint_b.key() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }

    let (escrow_account_pda, _bump) = pubkey::find_program_address(
        &[b"escrow".as_ref(), maker.key().as_slice()],
        &crate::ID
    );

    log(&escrow_account_pda);
    log(&escrow_account.key());
    
    if escrow_account_pda != *escrow_account.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    let escrow_account_state = Escrow::from_account_info(&escrow_account)?;
    if escrow_account_state.maker() != *maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    if !taker_ata_a_state.is_initialized() {
        pinocchio_associated_token_account::instructions::Create {
        funding_account: taker,
        account: taker_ata_a,
        wallet: taker,
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }.invoke()?;
    }
    
    if !maker_ata_b_state.is_initialized() {
         pinocchio_associated_token_account::instructions::Create {
        funding_account: taker,
        account: maker_ata_b,
        wallet: maker,
        mint: mint_b,
        token_program: token_program,
        system_program: system_program,
    }.invoke()?;
    }

    Ok(())
}