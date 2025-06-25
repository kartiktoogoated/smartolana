use anchor_spl::token::{self, Burn, TokenAccount, Mint, Token};

#derive(Accounts)
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account
        (mut, 
        constraint = owner_ata.owner == owner.key(),
        constraint = owner_ata.mint == mint.key())]
    pub owner_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn burn_tokens(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.owner_ata.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );

    token::new(cpi_ctx, amount)?;
    Ok(())
}