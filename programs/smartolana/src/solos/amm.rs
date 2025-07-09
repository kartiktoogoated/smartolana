pub fn swap(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;

    // Read tokens reservers directly from vault accounts
    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;

    // Uniswap vs style constant product with 0.3% fee
    let amount_in_with_fee = amount_in * 997;
    let numerator = amount_in_with_fee * reserve_b;
    let denominator = (reserve_a * 1000) + amount_in_with_fee;
    let amount_out = numerator / denominator;

    // Protect against front running or unexpected slippage
    require!(amount_out >= min_amount_out, CustomError::SlippageExceeded);

    // Transfer user token_in -> vault_a
    let cpi_ctx_in = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_in_ata.to_account_info(),
            to: ctx.accounts.vault_a.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_in, amount_in)?;

    // Transfer token_out from vault_b -> user_out_ata using PDA
    let bump = ctx.bumps.vault_authority;
    let pool_key = pool.key();
    let token_out_key = ctx.accounts.token_out.key();
    let signer_seeds: &[&[&[u8]]] =
        &[&[b"vault", pool_key.as_ref(), token_out_key.as_ref(), &[bump]]];

    let cpi_ctx_out = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_b.to_account_info(),
            to: ctx.accounts.user_out_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_ctx_out, amount_out)?;

    Ok(())
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_in_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_out_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"vault", pool.key().as_ref(), token_out.key().as_ref()],
        bump
    )]
    /// CHECK: PDA auth used to sign vault to user transfer verified via :-
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_out: Account<'info, Mint>, // Used only for PDA seed

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Pool {
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub total_lp_supply: u64,
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 1;
}

