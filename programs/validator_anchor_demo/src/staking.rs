pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
    let stake_vault = &mut ctx.accounts.stake_vault;

    // ❌ Prevent re-staking if already active
    require!(stake_vault.amount == 0, CustomError::AlreadyStaked);

    // 🕐 Get current timestamp
    let now = Clock::get()?.unix_timestamp;

    // 🎯 Transfer tokens via CPI (user -> vault ATA)
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_ata.to_account_info(),
            to: ctx.accounts.vault_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount)?;

    // 💾 Record stake info
    stake_vault.owner = ctx.accounts.user.key();
    stake_vault.vault = ctx.accounts.vault_ata.key();
    stake_vault.amount = amount;
    stake_vault.start_stake_time = now;
    stake_vault.bump = *ctx.bumps.get("stake_vault").unwrap();

    Ok(())
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [b"stake_vault", user.key().as_ref()],
        bump,
        payer = user,
        space = StakeVault::LEN,
    )]
    pub stake_vault: Account<'info, StakeVault>,

    #[account(
        mut,
        constraint = user_ata.owner == user.key()
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct StakeVault {
    pub owner: Pubkey,  // user wallet address
    pub vault: Pubkey,  // token account that holds staked token
    pub start_stake_time: i64,  // timestamp of when the stake began
    pub amount: u64,  // amount of tokens staked
    pub bump: u8, // bump for PDA
}

impl StakeVault {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 1;
}