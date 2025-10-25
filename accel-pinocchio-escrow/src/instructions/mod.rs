pub mod make;
pub mod take;
pub mod cancel;
pub mod transfer_to_escrow;
pub mod transfer_to_maker;
pub mod transfer_to_taker;

pub use make::*;
pub use take::*;
pub use cancel::*;
pub use transfer_to_escrow::*;
pub use transfer_to_maker::*;
pub use transfer_to_taker::*;

pub enum EscrowInstrctions {
    Make = 0,
    Take = 1,
    Cancel = 2,
}

impl TryFrom<&u8> for EscrowInstrctions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstrctions::Make),
            1 => Ok(EscrowInstrctions::Take),
            2 => Ok(EscrowInstrctions::Cancel),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}