use anchor_spl::token::{SetAuthority, set_authority, AuthorityType};

pub fn reassign_mint_authority(ctx: Context<ReassignMintAuthority>,new_authority: Pubkey) -> Result<()> {
    let bump = ctx.bumps.mint_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"mint-authority", &[bump]]];

    set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.mint_authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        AuthorityType::MintTokens,
        Some(new_authority),
    )?;

    Ok(())
}

// Context Setup
#[derive(Accounts)]
pub struct ReassignMintAuthority<'info> {
    #[account(
        mut,
        seeds = [b"global-mint"],
        bump,
    )]
    pub mint: Account<'info, Mint>,

    ///CHECK: PDA mint authority (not enforced here, signer seeds used instead)
    #[account(
        seeds = [b"mint-authority"],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}