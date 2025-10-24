use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};


#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contribute {
    amount: [u8; 8],
}

impl Contribute {
    pub const SEED: &str = "contribute";
    pub const LEN: usize = 8;

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }
}