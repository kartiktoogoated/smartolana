pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let stake_vault = &mut ctx.accounts.stake_vault;

    // Enforce lock period
    let stake_age = now - stake_vault.start_stake_time;
    require!(
        stake_age >= LOCK_PERIOD_SECONDS,
        CustomError::StakeLocked
    );

    // Transfer staked tokens back to user
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_ata.to_account_info(),
            to: ctx.accounts.user_ata.to_account_info(),
            authority: ctx.accounts.stake_vault.to_account_info(),
        },
    );
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"stake_vault",
        ctx.accounts.user.key.as_ref(),
        &[ctx.bumps.stake_vault],
    ]];
    token::transfer(cpi_ctx.with_signer(signer_seeds), stake_vault.amount)?;

    // Optionally reset vault state (preserve history if needed)
    stake_vault.amount = 0;
    stake_vault.start_stake_time = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake_vault", user.key().as_ref()],
        bump = stake_vault.bump,
        has_one = owner,
    )]
    pub stake_vault: Account<'info, StakeVault>,

    #[account(mut, constraint = user_ata.owner == user.key())]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(mut, constraint = vault_ata.key() == stake_vault.vault)]
    pub vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
