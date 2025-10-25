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
use solana_program::sysvar::{clock::Clock, Sysvar as SysvarClock};

use crate::states::*;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct CreateData {
    pub amount_to_raise: u64,
    pub duration: u8,
    pub _padding: [u8; 7],
}

pub trait DataLen {
    const LEN: usize;
}

impl DataLen for CreateData {
    const LEN: usize = core::mem::size_of::<CreateData>();
}

impl CreateData {
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

pub fn process_create(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [creator, mint_to_raise, fundraiser_account, vault, system_program, token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    if data.len() != CreateData::LEN {
        return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
    }

    let ix_data = bytemuck::try_pod_read_unaligned::<CreateData>(data)
        .map_err(|_| pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    if ix_data.amount_to_raise > Fundraiser::MIN_AMOUNT_TO_RAISE.pow(ix_data.duration as u32) {
        return Err(pinocchio::program_error::ProgramError::InvalidInstructionData)?;
    }

    let seed = [Fundraiser::SEED.as_ref(), creator.key().as_ref()];
    let (fundraiser_account_pda, bump) = pubkey::find_program_address(&seed, &crate::ID);

    log(&fundraiser_account_pda);
    log(&fundraiser_account.key());
    assert_eq!(fundraiser_account_pda, *fundraiser_account.key());

    let amount_to_raise = ix_data.amount_to_raise;
    let duration = ix_data.duration;

    let initial_bump = [bump.to_le()];
    let seed = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(creator.key()),
        Seed::from(&initial_bump),
    ];
    let seeds = Signer::from(&seed);

    if unsafe { fundraiser_account.owner() } != &crate::ID {
        CreateAccount {
            from: creator,
            to: fundraiser_account,
            lamports: Rent::get()?.minimum_balance(Fundraiser::LEN),
            space: Fundraiser::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[seeds.clone()])?;

        let fundraiser_state = Fundraiser::from_account_info(&fundraiser_account)?;
        let now = Clock::get();

        fundraiser_state.set_creator(*creator.key());
        fundraiser_state.set_mint_to_raise(*mint_to_raise.key());
        fundraiser_state.set_amount_to_raise(amount_to_raise);
        fundraiser_state.set_current_amount_raised(0);
        fundraiser_state.set_time_started(now.unwrap().unix_timestamp);
        fundraiser_state.set_duration(duration);
        fundraiser_state.set_bump(initial_bump);
    } else {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    pinocchio_associated_token_account::instructions::Create {
        funding_account: creator,
        account: vault,
        wallet: fundraiser_account,
        mint: mint_to_raise,
        token_program,
        system_program,
    }
    .invoke()?;
    Ok(())
}
