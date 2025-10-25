use pinocchio::program_error::ProgramError;

pub mod create;
pub mod contribute;
pub mod refund;
pub mod checker;

pub use create::*;
pub use contribute::*;
pub use refund::*;
pub use checker::*;

#[repr(u8)]
pub enum ProgramInstruction {
    Create,
    Contribute,
    Checker,
    Refund,
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::Create),
            1 => Ok(ProgramInstruction::Contribute),
            2 => Ok(ProgramInstruction::Checker),
            3 => Ok(ProgramInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}