use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint, entrypoint::ProgramResult,
    msg, pubkey::Pubkey,
    program_error::ProgramError,
    program::{invoke_signed},
};
use spl_token::instruction::mint_to;

entrypoint!(process_instruction);

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum MyInstruction {
    MintReward { amount: u64 },
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MyInstruction::try_from_slice(instruction_data)?;

    match instruction {
        MyInstruction::MintReward { amount } => mint_reward(accounts, amount, program_id)
    }
}

fn main_reward(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let mint = next_account_info(account_info_iter)?;               // Mint with PDA as authority
    let recipient = next_account_info(account_info_iter)?;          // ATA of user
    let vault_authority = next_account_info(account_info_iter)?;    // PDA signer
    let token_program = next_account_info(account_info_iter)?;      // SPL Token Program

    // Derive the PDA expected
    let (expected_pda, bump) = Pubkey::find_program_address(&[b"vault-authority"], program_id);

    if expected_pda != *vault_authority.key {
        msg!("Invalid PDA");
        return Err(ProgramError::InvalidSeeds);
    }

    // Build the MintTo instruction from SPL
    let ix = mint_to(
        token_program.key,
        mint.key,
        recipient.key,
        vault_authority.key,
        &[],
        amount
    )?;

    // Sign with PDA
    let signer_seeds: &[&[&[u8]]] = &[&[b"vault-authority", &[bump]]];

    invoke_signed(
        &ix,
        &[
            mint.clone(),
            recipient.clone(),
            vault_authority.clone(),
            token_program.clone(),
        ],
        signer_seeds,
    )?;

    msg!("Minted {} tokens to recipient", amount);
    Ok(())
}