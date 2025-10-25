use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::{self},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use solana_program::sysvar::{Sysvar as SysvarClock, clock::Clock};

use crate::states::*;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct ContributeData {
    pub amount_contributed: u64,
}

pub trait DataLen {
    const LEN: usize;
}

impl DataLen for ContributeData {
    const LEN: usize = core::mem::size_of::<ContributeData>();
}

impl ContributeData {
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

pub fn process_contribute(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {

    let [
        contributer, 
        creator, 
        contribute_account,
        contribute_ata, 
        mint_to_raise, 
        fundraiser_account, 
        vault, 
        system_program, 
        token_program, 
        _associated_token_program, 
        _rent_sysvar @ ..
        ] =
        accounts
    else {return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);};

    if data.len() != ContributeData::LEN {return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);}

    let ix_data = bytemuck::try_pod_read_unaligned::<ContributeData>(data)
        .map_err(|_| pinocchio::program_error::ProgramError::InvalidInstructionData)?;


    let mint_to_raise_state = pinocchio_token::state::Mint::from_account_info(mint_to_raise)?;

    let fundraiser_seed = [Fundraiser::SEED.as_ref(), creator.key().as_ref()];

    let contribute_seed = [Contribute::SEED.as_ref(), contributer.key().as_ref(), fundraiser_account.key().as_ref()];

    let (fundraiser_account_pda, _bump) = pubkey::find_program_address(&fundraiser_seed, &crate::ID);

    let (contribute_account_pda, bump) = pubkey::find_program_address(&contribute_seed, &crate::ID);

    assert_eq!(fundraiser_account_pda,*fundraiser_account.key());
    
    assert_eq!(contribute_account_pda,*contribute_account.key());

    let fundraiser_state = Fundraiser::from_account_info(&fundraiser_account)?;

    if fundraiser_state.mint_to_raise() != *mint_to_raise.key() {return Err(pinocchio::program_error::ProgramError::IllegalOwner);}

    assert!(ix_data.amount_contributed > 1_u8.pow(mint_to_raise_state.decimals() as u32) as u64);
    assert!(ix_data.amount_contributed <= ( fundraiser_state.amount_to_raise() * Fundraiser::MAX_CONTRIBUTION_PERCENTAGE / Fundraiser::PERCENTAGE_SCALER) as u64);

    let current_time = Clock::get().unwrap().unix_timestamp;
    assert!(current_time <= (((current_time - fundraiser_state.time_started()) / Fundraiser::SECONDS_TO_DAYS) as u8).into());

    let amount_to_raise = ix_data.amount_contributed;

    let initial_bump = bump.to_le();
    let bump = [initial_bump];
    let seed = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(creator.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    if contribute_account.owner() != &crate::ID {
        CreateAccount {
            from: contributer,
            to: contribute_account,
            lamports: Rent::get()?.minimum_balance(Contribute::LEN),
            space: Contribute::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[seeds.clone()])?;  
    } else {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

   

    pinocchio_token::instructions::Transfer {
        from: contribute_ata,
        to: vault,
        authority: contributer,
        amount: ix_data.amount_contributed,
    }
    .invoke()?;


    let contribute_state = Contribute::from_account_info(&contribute_account)?;
    fundraiser_state.set_amount_to_raise(fundraiser_state.amount_to_raise().checked_add(ix_data.amount_contributed).unwrap());
    contribute_state.set_amount(contribute_state.amount().checked_add(ix_data.amount_contributed).unwrap());

    Ok(())
}

