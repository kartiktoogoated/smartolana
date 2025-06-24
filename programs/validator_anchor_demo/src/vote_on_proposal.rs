#[account]
pub struct VoteRecord {
    pub proposal: Pubkey,
    pub validator: Pubkey,
    pub vote: bool,       // true = yes, false = no
    pub timestamp: i64,   // when vote was cast
    pub bump: u8,
}

impl VoteRecord {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 1;
}

// Vote Account Struct

#[account]
pub struct VoteRecord {
    pub proposal: Pubkey,
    pub validator: Pubkey,
    pub vote: bool,       // true = yes, false = no
    pub timestamp: i64,   // when vote was cast
    pub bump: u8,
}

impl VoteRecord {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 1;
}

// Context Struct: VoteOnProposal

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"validator", authority.key().as_ref(), &validator.id.to_le_bytes()],
        bump = validator.bump,
        has_one = authority,
    )]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        mut,
        seeds = [b"proposal", profile.key().as_ref(), &proposal.id.to_le_bytes()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = authority,
        seeds = [b"vote", proposal.key().as_ref(), validator.key().as_ref()],
        bump,
        space = VoteRecord::LEN
    )]
    pub vote_record: Account<'info, VoteRecord>,

    pub system_program: Program<'info, System>,
}


// Instruction

pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    vote: bool,
) -> Result<()> {
    let clock = Clock::get()?;
    let proposal = &mut ctx.accounts.proposal;

    require!(
        clock.unix_timestamp < proposal.deadline,
        CustomError::ProposalExpired
    );

    if vote {
        proposal.yes_votes += 1;
    } else {
        proposal.no_votes += 1;
    }

    let vote_record = &mut ctx.accounts.vote_record;
    vote_record.proposal = proposal.key();
    vote_record.validator = ctx.accounts.validator.key();
    vote_record.vote = vote;
    vote_record.timestamp = clock.unix_timestamp;
    vote_record.bump = ctx.bumps.vote_record;

    Ok(())
}

#[error_code]
pub enum CustomError {
    
    #[msg("Voting period has ended for this proposal")]
    ProposalExpired,
}

