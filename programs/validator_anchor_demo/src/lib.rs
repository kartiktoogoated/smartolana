/**
 * THE OVERALL FLOW
 * program logic,
 * context structs,
 * account structs
 */
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::spl_token::state::Mint as SplMint;
use anchor_spl::token::{
    self, set_authority, spl_token, Burn, InitializeMint, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer
};
use spl_token::instruction::AuthorityType;

declare_id!("BH2vhWg3AJqKn5VXKf6nepTPQUigJEhPEApUo9XXekjz");

#[program]
/**
It signals to Anchor the account is an executable one, i.e. a program, and you may issue to it a cross program invocation.
The one we have been using is the system program, though later we will use our own programs.
 */
pub mod validator_anchor_demo {
    use super::*;

    pub fn init_profile(ctx: Context<InitProfile>, name: String) -> Result<()> {
        let profile = &mut ctx.accounts.profile;
        profile.authority = ctx.accounts.authority.key();
        profile.name = name;
        profile.bump = ctx.bumps.profile;
        Ok(())
    }

    pub fn create_mint(ctx: Context<CreateMint>) -> Result<()> {
        let mint_info = ctx.accounts.mint.to_account_info();
        let mint_data = mint_info.try_borrow_data()?;
        let is_initialized = SplMint::unpack(&mint_data).is_ok();

        if !is_initialized {
            let bump = ctx.bumps.mint_authority;
            let signer_seeds: &[&[&[u8]]] = &[&[b"mint-authority", &[bump]]];

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                InitializeMint {
                    mint: mint_info.clone(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                signer_seeds,
            );

            token::initialize_mint(
                cpi_ctx,
                9,
                ctx.accounts.mint_authority.key,
                Some(ctx.accounts.mint_authority.key),
            )?;
        }

        Ok(())
    }

    pub fn init_validator(ctx: Context<InitValidator>, id: u64, name: String) -> Result<()> {
        let validator = &mut ctx.accounts.validator;
        validator.id = id;
        validator.name = name;
        validator.is_active = true;
        validator.authority = ctx.accounts.authority.key();
        validator.profile = ctx.accounts.profile.key();
        validator.bump = ctx.bumps.validator;

        let bump = ctx.bumps.mint_authority;
        let signer_seeds: &[&[&[u8]]] = &[&[b"mint-authority", &[bump]]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.validator_ata.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        );

        token::mint_to(cpi_ctx, 100_000_000_000)?; // 100 tokens with 9 decimals
        Ok(())
    }

    pub fn update_validator(
        ctx: Context<UpdateValidator>,
        new_name: String,
        is_active: bool,
    ) -> Result<()> {
        let validator = &mut ctx.accounts.validator;
        validator.name = new_name;
        validator.is_active = is_active;
        Ok(())
    }

    pub fn close_validator(_ctx: Context<CloseValidator>) -> Result<()> {
        Ok(())
    }

    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        // Manual verification that the sender owns the 'from' ATA
        require!(
            ctx.accounts.from.owner == ctx.accounts.sender.key(),
            CustomError::Unauthorized
        );

        token::transfer(ctx.accounts.into_transfer_context(), amount)?;
        Ok(())
    }

    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.owner_ata.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );

        token::burn(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn reassign_mint_authority(
        ctx: Context<ReassignMintAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
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

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_id: u64,
        title: String,
        description: String,
        deadline: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        require!(deadline > clock.unix_timestamp, CustomError::InvalidDeadline);

        let proposal = &mut ctx.accounts.proposal;
        proposal.id = proposal_id;
        proposal.profile = ctx.accounts.profile.key();
        proposal.title = title;
        proposal.description = description;
        proposal.created_at = clock.unix_timestamp;
        proposal.deadline = deadline;
        proposal.yes_votes = 0;
        proposal.no_votes = 0;
        proposal.bump = ctx.bumps.proposal;

        Ok(())
    }

    pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, vote: bool) -> Result<()> {
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

}

// ----------------- CONTEXT STRUCTS ---------------------

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitProfile<'info> {
    #[account(
        init,
        seeds = [b"profile", authority.key().as_ref()],
        bump,
        payer = authority,
        space = UserProfile::LEN
    )]
    pub profile: Account<'info, UserProfile>, // The Account type will check that the owner of the account being loaded is actually owned by the program

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        mint::decimals = 9,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
        seeds = [b"global-mint"],
        bump
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA mint authority, validated via seed constraints
    #[account(seeds = [b"mint-authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>, // UncheckedAccount is an alias for AccountInfo. This does not check for ownership, so care must be taken as it will accept arbitrary accounts.

    #[account(mut)]
    pub payer: Signer<'info>, // This type will check that the Signer account signed the transaction; it checks that the signature matches the public key of the account.

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id: u64, name: String)]
pub struct InitValidator<'info> {
    #[account(
        init,
        seeds = [b"validator", authority.key().as_ref(), &id.to_le_bytes()],
        bump,
        payer = authority,
        space = ValidatorInfo::LEN
    )]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub validator_ata: Account<'info, TokenAccount>,

    /// CHECK: Mint authority PDA
    #[account(seeds = [b"mint-authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut, seeds = [b"global-mint"], bump)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateValidator<'info> {
    #[account(
        mut,
        seeds = [b"validator", authority.key().as_ref(), &validator.id.to_le_bytes()],
        bump = validator.bump,
        has_one = authority,
        has_one = profile
    )]
    pub validator: Account<'info, ValidatorInfo>,

    pub authority: Signer<'info>,

    #[account(
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, UserProfile>,
}

#[derive(Accounts)]
pub struct CloseValidator<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"validator", authority.key().as_ref(), &validator.id.to_le_bytes()],
        bump = validator.bump,
        has_one = authority,
        has_one = profile
    )]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, UserProfile>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(mut)]
    pub from: Account<'info, TokenAccount>, // SENDERS ATA
    #[account(mut)]
    pub to: Account<'info, TokenAccount>, // RECEIVERS ATA

    pub token_program: Program<'info, Token>,
}

impl<'info> TransferTokens<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.sender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = owner_ata.owner == owner.key(),
        constraint = owner_ata.mint == mint.key()
    )]
    pub owner_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ReassignMintAuthority<'info> {
    #[account(
        mut,
        seeds = [b"global-mint"],
        bump
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA mint authority (not enforced here, signer seeds used instead)
    #[account(seeds = [b"mint-authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateProposal<'info> {
    #[account(
        mut,
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        init,
        seeds = [b"proposal", profile.key().as_ref(), &proposal_id.to_le_bytes()],
        bump,
        payer = authority,
        space = Proposal::LEN
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

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
        has_one = authority,
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        mut,
        seeds = [b"proposal", profile.key().as_ref(), &proposal.id.to_le_bytes()],
        bump = proposal.bump,
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

// ----------------- ACCOUNT STRUCTS ---------------------

#[account]
pub struct UserProfile {
    pub authority: Pubkey, // 32
    pub name: String,      // 4 (length prefix) + max characters
    pub bump: u8,          // 1
}

impl UserProfile {
    pub const LEN: usize = 8 + 32 + 4 + 32 + 1; // 8 = discriminator
}

#[account]
pub struct ValidatorInfo {
    pub id: u64,           // 8
    pub name: String,      // 4 + 32
    pub is_active: bool,   // 1
    pub authority: Pubkey, // 32
    pub profile: Pubkey,   // 32
    pub bump: u8,          //1
}

impl ValidatorInfo {
    pub const LEN: usize = 8 + 8 + 4 + 32 + 1 + 32 + 32 + 1;
}

#[account]
pub struct Proposal {
    pub id: u64,  // unique per profile
    pub profile: Pubkey, // creator profile pda
    pub title: String,  // 4 + N
    pub description: String,  // 4 + M
    pub created_at: i64,  // UNIX timestamp
    pub deadline: i64,  // voting end
    pub yes_votes: u64,
    pub no_votes: u64,
    pub bump: u8,
}

impl Proposal {
    pub const LEN: usize = 8 + 8 + 32 + (4 + 64) + (4 + 256) + 8 + 8 + 8 + 1;
}

#[account]
pub struct VoteRecord {
    pub proposal: Pubkey,
    pub validator: Pubkey,
    pub vote: bool,
    pub timestamp: i64,
    pub bump: u8,
}

impl VoteRecord {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 1;
}
// ----------------- ERROR ---------------------

#[error_code]
pub enum CustomError {
    #[msg("Sender is not the owner of the provided token account")]
    Unauthorized,

    #[msg("Deadline must be a future timestamp")]
    InvalidDeadline,

    #[msg("Voting period has ended for this proposal")]
    ProposalExpired,
}
