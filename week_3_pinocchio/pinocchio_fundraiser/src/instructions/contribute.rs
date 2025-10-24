use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    pubkey::{self, log},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use solana_program::sysvar::{Sysvar as SysvarClock, clock::Clock};

use crate::states::Contribute;

pub fn process_contribute(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Contribute");
    Ok(())
}