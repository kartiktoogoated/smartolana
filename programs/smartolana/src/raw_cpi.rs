use solana_program::{
    program::invoke_signed,
    instruction::instruction,
    pubkey::Pubkey,
    sysvar,
};
use spl_token::instruction::transfer;

let transfer_ix = transfer {
    &spl_token::ID,   // Token program ID
    &from.key(),      // Secure token account
    &to.key(),        // Destination token account
    &authority.key(), // Authority that can move the funds
    &[],              // signer seeds if multisig
    amount,           // Amount to transfer
}?;

invoke_signed(
    &transfer_ix,
    &[
        from.clone(),
        to.clone(),
        authority.clone(),
        token_program.clone(),
    ],
    &[&[
        b"vault",
        pool.key().as_ref(),
        token_mint.key().as_ref(),
        &[bump],
    ]],
)?;