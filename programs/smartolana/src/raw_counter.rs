
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint, entrypoint::ProgramResult,
    msg, pubkey::Pubkey,
    program_error::ProgramError,
    sysvar::{rent::Rent, Sysvar},
};

// Entrypoint for Solana to call
entrypoint!(process_instruction);

// Instruction enum(client encodes this)
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MyInstruction {
    Initialize,
    Increment,
}

// Our on-chain state
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}

// Main processor
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MyInstruction::try_from_slice(instruction_data)?;

    match Instruction {
        MyInstruction::Initialize => handle_initialize(accounts, program_id),
        MyInstruction::Increment => handle_increment(accounts, program_id),
    }
}

fn handle_initialize(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_iter)?;
    let payer = next_account_info(account_iter)?;
    let rent_sysvar = next_account_info(account_iter)?;

    if !counter_account_is_writable {
        msg!("Counter account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    if counter_account.owner != program_id {
        msg!("Counter account not owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check rent exemption
    let rent = &Rent::from_account_info(rent_sysvar)?;
    if !rent.is_exempt(counter_account.lamports(), counter_account.data_len()) {
        msg!("Counter account not rent exempt");
        return Err(ProgramError::AccountNotRentExempt);
    }

    // Write 0 into the account
    let counter = CounterAccount { count: 0 };
    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;

    msg!("Counter initialized to 0");
    Ok(())
}

fn handle_increment(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;

    if !authority.is_signer {
        msg!("Missing signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !counter_account.is_writable {
        msg!("Counter account not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    if counter_account.owner != program_id {
        msg!("Counter account not owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut counter = CounterAccount::try_from_slice(&counter_account.data.borrow())?;
    counter.count += 1;
    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;

    msg!("Counter incremented: {}", counter.count);
    Ok(())
}