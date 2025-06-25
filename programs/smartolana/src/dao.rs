use anchor_lang::prelude::*;

#[account]
pub struct Proposal {
    pub id: u64,
    pub profile: Pubkey, // Creater profile pda
    pub title: String,
    pub description: String,
    pub created_at: i64,  // UNIX timestamps
    pub deadline: i64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub bump: u8,   // FOR PDA
}

// Creating Proposal Context
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateProposal<'info> {
    #[account(
        mut,
        has_one = authority // signer who owns the profile
    )]
    pub profile: Account<'info, Profile>,

    #[account(
        init,
        seeds = [b"proposal", profile.key().as_ref(), &proposal_id.to_le_bytes()],
        bump,
        payer = authority,
        space = 8 + 8 + 32 + (4 + 100) + (4 + 500) + 8 + 8 + 8 + 1
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>
}

pub fn create_proposal(
    ctx: Context<CreateProposal>,
    proposal_id: u64,
    title: String,
    description: String,
    deadline: i64,
) -> Result<()> {
    let clock = Clock::get()?;
    let proposal = &mut ctx.accounts.proposal;

    proposal.id = proposal_id;
    proposal.profile = ctx.accounts.profile.key();
    proposal.title = title;
    proposal.description = description;
    proposal.created_at = clock.unix_timestamp;
    proposal.deadline = deadline;
    proposal.yes_votes = 0;
    proposal.no_votes = 0;
    proposal.bump = *ctx.bumps.get("proposal").unwrap();

    Ok(())
}
