#![allow(unexpected_cfgs)]
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
pub mod smartolana {
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

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        let stake_vault = &mut ctx.accounts.stake_vault;

        require!(stake_vault.amount == 0, CustomError::AlreadyStaked);
        require!(amount > 0, CustomError::ZeroStake);

        let now = Clock::get()?.unix_timestamp;

        // Transfer tokens from user ATA to vault ATA
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.vault_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        // Record staking data
        stake_vault.owner = ctx.accounts.user.key();
        stake_vault.profile = ctx.accounts.profile.key();
        stake_vault.vault = ctx.accounts.vault_ata.key();
        stake_vault.amount = amount;
        stake_vault.start_stake_time = now;
        stake_vault.reward_collected = 0;
        stake_vault.pool = ctx.accounts.pool.key();
        stake_vault.bump = ctx.bumps.stake_vault;

        // Update pool stats
        let pool = &mut ctx.accounts.pool;
        pool.total_staked = pool.total_staked.checked_add(amount).unwrap();

        msg!("Staked {} tokens at time {} into pool {}", amount, now, pool.id);
        Ok(())
    }

    pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
        let stake_vault = &mut ctx.accounts.stake_vault;
        let now = Clock::get()?.unix_timestamp;
        let pool = &ctx.accounts.pool;
    
        require!(
            now >= stake_vault.start_stake_time + pool.lock_period as i64,
            CustomError::StakeLocked
        );
    
        let amount = stake_vault.amount;
        require!(amount > 0, CustomError::ZeroStake);
    
        // Fix temporary borrow issue
        let user_key = ctx.accounts.user.key();
        let bump = ctx.bumps.stake_vault;
        let signer_seeds: &[&[u8]] = &[b"stake-vault", user_key.as_ref(), &[bump]];
        let signer: &[&[&[u8]]] = &[signer_seeds];
    
        // Use already borrowed `stake_vault` here
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_ata.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: stake_vault.to_account_info(),
            },
            signer,
        );
    
        token::transfer(cpi_ctx, amount)?;
    
        stake_vault.amount = 0;
        stake_vault.start_stake_time = 0;
    
        msg!("Unstaked {} tokens at time {}", amount, now);
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let stake_vault = &mut ctx.accounts.stake_vault;
        let now = Clock::get()?.unix_timestamp;

        let elapsed = now
            .checked_sub(stake_vault.start_stake_time)
            .ok_or(CustomError::TimeCalculationFailed)?;

        let reward_rate = ctx.accounts.pool.reward_per_second;
        let total_reward = elapsed as u64 * reward_rate;
        let pending = total_reward.saturating_sub(stake_vault.reward_collected);

        require!(pending > 0, CustomError::NoRewardAvailable);

        let vault_balance = ctx.accounts.reward_vault.amount;
        require!(
            vault_balance >= pending,
            CustomError::InsufficientRewardVault
        );

        let bump = ctx.bumps.mint_authority;
        let signer_seeds: &[&[&[u8]]] = &[&[b"mint-authority", &[bump]]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.user_reward_ata.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds
        );

        token::mint_to(cpi_ctx, pending)?;
        stake_vault.reward_collected += pending;

        // Deduct from internal reward balance
        ctx.accounts.pool.reward_balance = ctx.accounts.pool.reward_balance.saturating_sub(pending);

        msg!(
            "Minted {} tokens as reward to user {} from pool {}",
            pending,
            ctx.accounts.user.key(),
            ctx.accounts.pool.id
        );

        Ok(())
    }

    pub fn init_staking_pool(ctx: Context<InitStakingPool>, id: u64, name: String, reward_per_second: u64, lock_period: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.id = id;
        pool.name = name;
        pool.authority = ctx.accounts.authority.key();
        pool.stake_mint = ctx.accounts.stake_mint.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.reward_per_second = reward_per_second;
        pool.total_staked = 0;
        pool.lock_period = lock_period;
        pool.bump = ctx.bumps.pool;
        pool.reward_vault = ctx.accounts.reward_vault.key();
        pool.reward_vault_authority_bump = ctx.bumps.reward_vault_authority;

        Ok(())
    }

    pub fn refill_pool(ctx: Context<RefillPool>, amount: u64) -> Result<()> {
        // Transfer tokens from admin_ata -> reward_vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.admin_ata.to_account_info(),
                    to: ctx.accounts.reward_vault.to_account_info(),
                    authority: ctx.accounts.admin.to_account_info(),
                },
            ),
            amount,
        )?;

        // Track internal reward balance
        let pool = &mut ctx.accounts.pool;
        pool.reward_balance = pool.reward_balance.saturating_add(amount);

        msg!(
            "Refilled pool {} with {} tokens into reward vault",
            pool.id,
            amount
        );

        Ok(())
    }

    pub fn update_pool_config(ctx: Context<UpdatePoolConfig>, new_rate: u64, new_lock_period: u64, pause: bool) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;

        pool.reward_per_second = new_rate;
        pool.lock_period = new_lock_period;
        pool.paused = pause;

        msg!(
            "Updated pool config → rate: {}, lock: {}, paused: {}",
            new_rate,
            new_lock_period,
            pause
        );

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

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = profile.authority == user.key()
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        init_if_needed,
        seeds = [b"stake-vault", user.key().as_ref()],
        bump,
        payer = user,
        space = StakeVault::LEN
    )]
    pub stake_vault: Account<'info, StakeVault>,

    #[account(
        mut,
        constraint = user_ata.owner == user.key(),
        constraint = user_ata.mint == stake_mint.key(),
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = stake_mint,
        associated_token::authority = stake_vault,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = pool.stake_mint == stake_mint.key(),
    )]
    pub pool: Account<'info, StakingPool>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = stake_vault.owner == user.key(),
        seeds = [b"stake-vault", user.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, StakeVault>,

    #[account(
        mut,
        constraint = user_ata.owner == user.key(),
        constraint = user_ata.mint == stake_mint.key()
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = stake_vault
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = stake_vault.pool == pool.key()
    )]
    pub pool: Account<'info, StakingPool>,    

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake-vault", user.key().as_ref()],
        bump,
        constraint = stake_vault.owner == user.key()
    )]
    pub stake_vault: Account<'info, StakeVault>,

    #[account(
        mut,
        constraint = user_reward_ata.owner == user.key(),
        constraint = user_reward_ata.mint == reward_mint.key()
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,

    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,

    /// CHECK: mint_authority PDA
    #[account(seeds = [b"mint-authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = stake_vault.pool == pool.key(),
        constraint = pool.reward_mint == reward_mint.key(),
        constraint = pool.reward_vault == reward_vault.key()
    )]
    pub pool: Account<'info, StakingPool>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(id: u64, name: String, reward_per_second: u64, lock_period: u64)]
pub struct InitStakingPool<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"pool", authority.key().as_ref(), &id.to_le_bytes()],
        bump,
        space = StakingPool::LEN
    )]
    pub pool: Account<'info, StakingPool>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub stake_mint: Account<'info, Mint>,
    pub reward_mint: Account<'info, Mint>,

    // Reward vault ATA (owned by PDA)
    #[account(
        init,
        payer = authority,
        associated_token::mint = reward_mint,
        associated_token::authority = reward_vault_authority
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    // PDA authority for the reward vault
    /// CHECK: PDA authority for vault, validated by seeds
    #[account(
        seeds = [b"reward-vault", pool.key().as_ref()],
        bump
    )]
    pub reward_vault_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RefillPool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    // Ensure admin is tranferring from their own ATA (must match reward mint)
    #[account(
        mut,
        constraint = admin_ata.owner == admin.key(),
        constraint = admin_ata.mint == pool.reward_mint
    )]
    pub admin_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        has_one = reward_vault,
        constraint = pool.authority == admin.key()
    )]
    pub pool: Account<'info, StakingPool>,

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct UpdatePoolConfig<'info> {
    #[account(mut, has_one = authority)]
    pub staking_pool: Account<'info, StakingPool>,
    pub authority: Signer<'info>,
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

#[account]
pub struct StakeVault {
    pub owner: Pubkey,  // 32 bytes
    pub profile: Pubkey,  // 32 bytes
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,  // 8 bytes
    pub reward_collected: u64, // 8 bytes
    pub start_stake_time: i64,  // 8 bytes
    pub bump: u8,  // 1 byte
}

impl StakeVault {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct StakingPool {
    pub id: u64,
    pub name: String,
    pub authority: Pubkey, // pool creator
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_per_second: u64,
    pub total_staked: u64,
    pub lock_period: u64,
    pub reward_vault: Pubkey,
    pub reward_vault_authority_bump: u8,
    pub reward_balance: u64, // total tokens available for rewards
    pub paused: bool,
    pub bump: u8,
}

impl StakingPool {
    pub const LEN: usize = 8 + 8 + (4 + 32) + 32 + 32 + 32 + 8 + 8 + 8 + 32 + 1 + 1 + 1;
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

    #[msg("User already has an active stake")]
    AlreadyStaked,

    #[msg("Cannot stake zero amount")]
    ZeroStake,

    #[msg("Stake is still locked. Please wait for the lock period to pass.")]
    StakeLocked,

    #[msg("No rewards available to claim")]
    NoRewardAvailable,

    #[msg("Stake time calculation failed")]
    TimeCalculationFailed,

    #[msg("Insufficient reward vault balance")]
    InsufficientRewardVault,
}
