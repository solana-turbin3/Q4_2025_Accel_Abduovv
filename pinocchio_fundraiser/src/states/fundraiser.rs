use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};




#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fundraiser {
    creator: [u8; 32],
    mint_to_raise: [u8; 32],
    amount_to_raise: [u8; 8],
    current_amount_raised: [u8; 8],
    time_started: [u8; 8],
    duration: [u8; 1],
    bump: [u8; 1],
}

impl Fundraiser {
    pub const LEN: usize = core::mem::size_of::<Self>();
    pub const SEED: &str = "fundraiser";
    pub const MIN_AMOUNT_TO_RAISE: u64 = 3;
    pub const SECONDS_TO_DAYS: i64 = 86400;
    pub const MAX_CONTRIBUTION_PERCENTAGE: u64 = 10;
    pub const PERCENTAGE_SCALER: u64 = 100;


     pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut_data()?;
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn creator(&self) -> Pubkey {
        Pubkey::from(self.creator)
    }

    pub fn mint_to_raise(&self) -> Pubkey {
        self.mint_to_raise
    }

    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn current_amount_raised(&self) -> u64 {
        u64::from_le_bytes(self.current_amount_raised)
    }

    pub fn time_started(&self) -> i64 {
        i64::from_le_bytes(self.time_started) 
    }

    pub fn duration(&self) -> u8 {
        u8::from_le_bytes(self.duration)
    }

    pub fn bump(&self) -> u8 {
        u8::from_le_bytes(self.bump)
    }

    pub fn set_creator(&mut self, creator: Pubkey) {
        self.creator.copy_from_slice(creator.as_ref());;
    }

    pub fn set_mint_to_raise(&mut self, mint_to_raise: Pubkey) {
        self.mint_to_raise.copy_from_slice(mint_to_raise.as_ref());
    }

    pub fn set_amount_to_raise(&mut self, amount_to_raise: u64) {
        self.amount_to_raise = amount_to_raise.to_le_bytes();
    }

    pub fn set_current_amount_raised(&mut self, current_amount_raised: u64) {
        self.current_amount_raised = current_amount_raised.to_le_bytes();
    }

    pub fn set_time_started(&mut self, time_started: i64) {
        self.time_started = time_started.to_le_bytes();
    }

    pub fn set_duration(&mut self, duration: u8) {
        self.duration = duration.to_le_bytes();
    }

    pub fn set_bump(&mut self, bump: [u8; 1]) {
        self.bump = bump;
    }
}