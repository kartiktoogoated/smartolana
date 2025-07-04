use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum MyInstruction {
    Initialize { data: u64 },
    Increment,
    Reset,
}

/*
Initialize { data } → sets a counter to a value

Increment → adds 1

Reset → sets it to 0
*/

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::instruction::MyInstruction;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Raw program entrypoint hit");

    match instruction {
        MyInstruction::Initialize { data } => {
            msg!("Instruction: Initialize with {}", data);
            // handle_initialize(accounts, data)?;
        }
        MyInstruction::Increment => {
            msg!("Instruction: Increment");
            // handle_increment(accounts)?;
        }
        MyInstruction::Reset => {
            msg!("Instruction: Reset");
            // handle_reset(accounts)?;
        }
    }

    Ok(())
}
