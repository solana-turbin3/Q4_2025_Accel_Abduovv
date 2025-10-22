use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey,
    pubkey::log,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use crate::state::Escrow;

pub fn process_make_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing Make instruction");

    let [
        maker,
        mint_a,
        mint_b,
        escrow_account,
        maker_ata_a,
        escrow_ata,
        system_program,
        token_program,
        _associated_token_program,
        _rent_sysvar @ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let maker_ata_a_state = pinocchio_token::state::TokenAccount::from_account_info(maker_ata_a)?;
    if maker_ata_a_state.owner() != maker.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if maker_ata_a_state.mint() != mint_a.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // PDA check
    let (escrow_pda, bump) = pubkey::find_program_address(&[b"escrow", maker.key().as_ref()], &crate::ID);
    if escrow_pda != *escrow_account.key() {
        return Err(ProgramError::InvalidSeeds);
    }
    log(&escrow_pda);
    log(escrow_account.key());

        // Parse input safely
    if data.len() < 17 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount_to_receive = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
    let amount_to_give = u64::from_le_bytes(data[9..17].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);


        
    // Create escrow account securely
   
    if escrow_account.owner() != &crate::ID || escrow_account.data_len() != Escrow::LEN {
        CreateAccount {
            from: maker,
            to: escrow_account,
            lamports: Rent::get()?.minimum_balance(Escrow::LEN),
            space: Escrow::LEN as u64,
            owner: &crate::ID,
        }.invoke()?;
    } else {
        return Err(ProgramError::IllegalOwner);
    }

    // Write escrow data
        let mut escrow_state = Escrow::from_account_info(escrow_account)?;
        escrow_state.set_maker(maker.key());
        escrow_state.set_mint_a(mint_a.key());
        escrow_state.set_mint_b(mint_b.key());
        escrow_state.set_amount_to_receive(amount_to_receive);
        escrow_state.set_amount_to_give(amount_to_give);
        escrow_state.set_bump(bump);

    // Create escrow ATA
{
    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow_ata,
        wallet: escrow_account,
        mint: mint_a,
        token_program,
        system_program,
    }.invoke()?;
}
    pinocchio_token::instructions::Transfer {
        from: maker_ata_a,
        to: escrow_ata,
        authority: maker,
        amount: escrow_state.amount_to_give(),
    }.invoke()?;

    Ok(())
}
