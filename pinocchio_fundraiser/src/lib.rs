use pinocchio::{ProgramResult, account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey};

use crate::instructions::ProgramInstruction;

pub mod errors;
pub mod instructions;
pub mod states;

entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("AYQEqZMiyxTfz9m9fcoQuPu3SA6wtbD2XCjEMMgfiXJH");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    assert_eq!(program_id, &ID);

    let (disc, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

    let _ = match ProgramInstruction::try_from(disc)? {
        ProgramInstruction::Create => instructions::process_create(accounts, data),
        ProgramInstruction::Contribute => instructions::process_contribute(accounts, data),
        ProgramInstruction::Checker => instructions::process_checker(accounts),
        ProgramInstruction::Refund => instructions::process_refund(accounts),
    };
    Ok(())
}