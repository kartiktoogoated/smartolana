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
    self, set_authority, spl_token, Burn, InitializeMint, Mint, MintTo, SetAuthority, Token,
    TokenAccount, Transfer,
};
use spl_token::instruction::AuthorityType;

declare_id!("BH2vhWg3AJqKn5VXKf6nepTPQUigJEhPEApUo9XXekjz");

#[program]
/**
It signals to Anchor the account is an executable one, i.e. a program, and you may issue to it a cross program invocation.
The one we have been using is the system program, though later we will use our own programs.
 */
pub mod smartolana {
    use anchor_lang::accounts;

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
        require!(
            deadline > clock.unix_timestamp,
            CustomError::InvalidDeadline
        );

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

        msg!(
            "Staked {} tokens at time {} into pool {}",
            amount,
            now,
            pool.id
        );
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
            signer_seeds,
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

    pub fn init_staking_pool(
        ctx: Context<InitStakingPool>,
        id: u64,
        name: String,
        reward_per_second: u64,
        lock_period: u64,
    ) -> Result<()> {
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

    pub fn update_pool_config(
        ctx: Context<UpdatePoolConfig>,
        new_rate: u64,
        new_lock_period: u64,
        pause: bool,
    ) -> Result<()> {
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

    pub fn init_escrow(
        ctx: Context<InitEscrow>,
        amount_offered: u64,
        amount_expected: u64,
        unlock_at: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            unlock_at > clock.unix_timestamp,
            CustomError::InvalidDeadline
        );

        let escrow = &mut ctx.accounts.escrow;

        // Transfer offered tokens into vault PDA
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx
                    .accounts
                    .initializer_deposit_token_account
                    .to_account_info(),
                to: ctx.accounts.vault_amount.to_account_info(),
                authority: ctx.accounts.initializer.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_offered)?;

        escrow.initializer = ctx.accounts.initializer.key();
        escrow.initializer_deposit_token_account =
            ctx.accounts.initializer_deposit_token_account.key();
        escrow.vault_amount = ctx.accounts.vault_amount.key();
        escrow.mint_offered = ctx.accounts.mint_offered.key();
        escrow.mint_expected = ctx.accounts.mint_expected.key();
        escrow.amount_offered = amount_offered;
        escrow.amount_expected = amount_expected;
        escrow.unlock_at = unlock_at;
        escrow.is_fulfilled = false;
        escrow.bump = ctx.bumps.escrow;

        Ok(())
    }

    pub fn fulfill_escrow(ctx: Context<FulfillEscrow>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow = &mut ctx.accounts.escrow;

        require!(!escrow.is_fulfilled, CustomError::AlreadyFulfilled);
        require!(
            clock.unix_timestamp >= escrow.unlock_at,
            CustomError::StakeLocked
        );

        // Transfer expected tokens from taker -> initializer
        let pay_initializer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.taker_payment_ata.to_account_info(),
                to: ctx.accounts.initializer_receive_ata.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        );
        token::transfer(pay_initializer_ctx, escrow.amount_expected)?;

        // Transfer offered tokens from vault -> taker
        let bump = ctx.bumps.vault_authority;
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"vault-authority", escrow.initializer.as_ref(), &[bump]]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_amount.to_account_info(),
                to: ctx.accounts.taker_receive_ata.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx, escrow.amount_offered)?;

        escrow.is_fulfilled = true;

        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(!escrow.is_fulfilled, CustomError::AlreadyFulfilled);
        require!(
            ctx.accounts.initializer.key() == escrow.initializer,
            CustomError::Unauthorized
        );

        // Return locked tokens back to initializer
        let bump = ctx.bumps.vault_authority;
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"vault-authority", escrow.initializer.as_ref(), &[bump]]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_amount.to_account_info(),
                to: ctx.accounts.initializer_receive_ata.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx, escrow.amount_offered)?;

        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        max_amount_in: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;

        let reserve_a = ctx.accounts.input_vault.amount;
        let reserve_b = ctx.accounts.output_vault.amount;

        // Compute Output
        let amount_in_with_fee = amount_in * 997;
        let numerator = amount_in_with_fee * reserve_b;
        let denominator = (reserve_a * 1000) + amount_in_with_fee;
        let amount_out = numerator / denominator;

        // Slippage protection
        require!(amount_out >= min_amount_out, CustomError::SlippageExceeded);
        require!(amount_in <= max_amount_in, CustomError::SlippageExceeded);

        // Transfer Token A from user -> vault
        let cpi_ctx_in = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_input_token_account.to_account_info(),
                to: ctx.accounts.input_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_in, amount_in)?;

        // Transfer Token B from vault -> user
        let bump = ctx.bumps.vault_authority;
        let pool_key = pool.key();
        let token_out_key = ctx.accounts.output_token_mint.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"vault", pool_key.as_ref(), token_out_key.as_ref(), &[bump]]];

        let cpi_ctx_out = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.output_vault.to_account_info(),
                to: ctx.accounts.user_output_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_out, amount_out)?;

        msg!(
            "Swapped {} → {} (min expected {}, max allowed in {})",
            amount_in,
            amount_out,
            min_amount_out,
            max_amount_in
        );

        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let reserve_a = ctx.accounts.token_a_vault.amount;
        let reserve_b = ctx.accounts.token_b_vault.amount;

        let total_lp_supply = pool.total_lp_supply;

        // Step 1 - Transfer user tokens into vaults
        let cpi_ctx_a = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.token_a_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_a, amount_a)?;

        // Step 2 - Transfer token B from user to vault
        let cpi_ctx_b = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.token_b_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_b, amount_b)?;

        // Step 3 - Calculate LP tokens to mint
        let lp_to_mint = if total_lp_supply == 0 {
            // Frist LP - gets sqrt(amount_a * amount_b)
            integer_sqrt(amount_a.checked_mul(amount_b).unwrap())
        } else {
            let lp_from_a = amount_a * total_lp_supply / reserve_a;
            let lp_from_b = amount_b * total_lp_supply / reserve_b;
            lp_from_a.min(lp_from_b)
        };

        // Step 4 - Mint LP tokens to user using vault PDA as mint authority
        let bump = ctx.bumps.vault_authority;
        let pool_key = pool.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault-authority", pool_key.as_ref(), &[bump]]];

        let cpi_ctx_mint = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(cpi_ctx_mint, lp_to_mint)?;

        // Step 5 - Update LP supply in pool
        pool.total_lp_supply = pool.total_lp_supply.checked_add(lp_to_mint).unwrap();

        Ok(())
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let reserve_a = ctx.accounts.token_a_vault.amount;
        let reserve_b = ctx.accounts.token_b_vault.amount;

        let total_lp_supply = pool.total_lp_supply;
        require!(total_lp_supply > 0, CustomError::NoLiquidity);

        // Calculate the share of tokens to return
        let amount_a = lp_amount
            .checked_mul(reserve_a)
            .unwrap()
            .checked_div(total_lp_supply)
            .unwrap();

        let amount_b = lp_amount
            .checked_mul(reserve_b)
            .unwrap()
            .checked_div(total_lp_supply)
            .unwrap();

        // Step 1 - Burn LP tokens from user
        let cpi_ctx_burn = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::burn(cpi_ctx_burn, lp_amount)?;

        // Step 2 - Transfer token A to user
        let bump = ctx.bumps.vault_authority;
        let pool_key = pool.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault-authority", pool_key.as_ref(), &[bump]]];

        let cpi_ctx_a = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_a_vault.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_a, amount_a)?;

        // Step 3 - Transfer token B to user
        let cpi_ctx_b = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_b_vault.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_b, amount_b)?;

        // Step 4 - Update LP supply
        pool.total_lp_supply = pool.total_lp_supply.checked_sub(lp_amount).unwrap();

        Ok(())
    }

    pub fn create_multisig(
        ctx: Context<CreateMultisig>,
        owners: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        require!(
            owners.len() <= Multisig::MAX_OWNERS,
            CustomError::TooManyOwners
        );
        require!(
            threshold > 0 && threshold <= owners.len() as u8,
            CustomError::InvalidThreshold
        );

        let multisig = &mut ctx.accounts.multisig;

        multisig.owners = owners;
        multisig.threshold = threshold;
        multisig.bump = ctx.bumps.multisig;
        multisig.owner_set_seqno = 0;

        Ok(())
    }

    pub fn propose_transaction(
        ctx: Context<ProposeTransaction>,
        program_id: Pubkey,
        accounts: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        let proposer = &ctx.accounts.proposer;

        // Validator proposer is an owner
        require!(
            multisig.owners.contains(&proposer.key()),
            CustomError::NotAnOwner
        );

        // Populate transaction
        let tx = &mut ctx.accounts.tx;
        tx.multisig = multisig.key();
        tx.program_id = program_id;
        tx.accounts = accounts;
        tx.data = data;
        tx.did_execute = false;
        tx.signers = [false; 10];

        // Mark the propose as approved
        if let Some(index) = multisig.owners.iter().position(|k| k == &proposer.key()) {
            tx.signers[index] = true;
        } else {
            return Err(CustomError::NotAnOwner.into());
        }
        tx.owner_set_seqno = multisig.owner_set_seqno;

        Ok(())
    }

    pub fn approve_transaction(ctx: Context<ApproveTransaction>) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        let signer = &ctx.accounts.signer;
        let tx = &mut ctx.accounts.tx;

        // Check signer is one of the owners
        if let Some(index) = multisig.owners.iter().position(|k| k == &signer.key()) {
            // Ensure signer hasnt already signed
            require!(!tx.signers[index], CustomError::AlreadySigned);

            tx.signers[index] = true;
            Ok(())
        } else {
            Err(CustomError::NotAnOwner.into())
        }
    }

    pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
        let tx = &mut ctx.accounts.tx;
        let multisig = &ctx.accounts.multisig;

        // Count how many owners signed
        let sig_count = tx
            .signers
            .iter()
            .zip(multisig.owners.iter())
            .filter(|(signed, _)| **signed)
            .count();

        // Ensure enough approvals
        require!(
            sig_count >= multisig.threshold as usize,
            CustomError::InsufficientSignatures
        );

        // Mark tx as executed
        tx.did_execute = true;

        // CPI to the intended program
        let account_infos: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();

        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: tx.program_id,
            accounts: tx
                .accounts
                .iter()
                .map(
                    |acc| anchor_lang::solana_program::instruction::AccountMeta {
                        pubkey: acc.pubkey,
                        is_signer: acc.is_signer,
                        is_writable: acc.is_writable,
                    },
                )
                .collect(),
            data: tx.data.clone(),
        };

        anchor_lang::solana_program::program::invoke(&ix, &account_infos)?;

        Ok(())
    }

    pub fn init_pool_clmm(
        ctx: Context<InitPoolClmm>,
        sqrt_price_x64: u128,
        current_tick: i32,
        tick_spacing: u16,
        fee_rate: u16,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool_clmm;

        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.vault_a = ctx.accounts.vault_a.key();
        pool.vault_b = ctx.accounts.vault_b.key();

        pool.sqrt_price_x64 = sqrt_price_x64;
        pool.current_tick = current_tick;
        pool.tick_spacing = tick_spacing;
        pool.liquidity = 0;
        pool.fee_rate = fee_rate;
        pool.fee_growth_global_a = 0;
        pool.fee_growth_global_b = 0;

        pool.bump = ctx.bumps.pool_clmm;
        pool.signer_bump = ctx.bumps.pool_signer;

        Ok(())
    }

    pub fn init_tick(ctx: Context<InitTick>, tick_index: i32) -> Result<()> {
        let tick = &mut ctx.accounts.tick;
        tick.tick_index = tick_index;
        tick.liquidity_net = 0;
        tick.fee_growth_outside_a = 0;
        tick.fee_growth_outside_b = 0;
        Ok(())
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        liquidity: u128,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool_clmm;
        let tick_lower_account = &mut ctx.accounts.tick_lower;
        let tick_upper_account = &mut ctx.accounts.tick_upper;

        // Validate
        require!(tick_lower < tick_upper, CustomError::InvalidTickRange);
        require!(liquidity > 0, CustomError::ZeroLiquidity);

        // Update ticks
        tick_lower_account.liquidity_net = tick_lower_account
            .liquidity_net
            .checked_add(liquidity as i128)
            .unwrap();
        tick_upper_account.liquidity_net = tick_upper_account
            .liquidity_net
            .checked_sub(liquidity as i128)
            .unwrap();

        // Inline liquidity-to-token calculation (placeholder for now)
        let amount_a = (liquidity / 1000) as u64;
        let amount_b = (liquidity / 1000) as u64;

        // Cpi - Transfer token A
        let cpi_ctx_a = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_a, amount_a)?;

        // Cpi - Transfer token B
        let cpi_ctx_b = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_b, amount_b)?;

        // Store user_position
        let user_position = &mut ctx.accounts.user_position;
        user_position.owner = ctx.accounts.owner.key();
        user_position.pool_clmm = pool.key();
        user_position.tick_lower = tick_lower;
        user_position.tick_upper = tick_upper;
        user_position.liquidity = liquidity;
        user_position.fee_growth_checkpoint_a = 0;
        user_position.fee_growth_checkpoint_b = 0;

        // If tick is active update pool liquidity
        if pool.current_tick >= tick_lower && pool.current_tick < tick_upper {
            pool.liquidity = pool.liquidity.checked_add(liquidity).unwrap();
        }

        msg!(
            "Position opened: {} liquidity between ticks {} - {}",
            liquidity,
            tick_lower,
            tick_upper
        );

        Ok(())
    }

    pub fn decrease_liquidity(
        ctx: Context<DecreaseLiquidity>,
        liquidity_to_remove: u128,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool_clmm;
        let tick_lower = &mut ctx.accounts.tick_lower;
        let tick_upper = &mut ctx.accounts.tick_upper;
        let user_position = &mut ctx.accounts.user_position;

        // Safety checks
        require!(liquidity_to_remove > 0, CustomError::ZeroLiquidity);
        require!(
            liquidity_to_remove <= user_position.liquidity,
            CustomError::InvalidLiquidityAmount
        );

        // Update tick liquidity_net
        tick_lower.liquidity_net = tick_lower
            .liquidity_net
            .checked_sub(liquidity_to_remove as i128)
            .unwrap();
        tick_upper.liquidity_net = tick_upper
            .liquidity_net
            .checked_add(liquidity_to_remove as i128)
            .unwrap();

        // Update pool liquidity if position is currently active
        if pool.current_tick >= tick_lower.tick_index && pool.current_tick < tick_upper.tick_index {
            pool.liquidity = pool.liquidity.checked_sub(liquidity_to_remove).unwrap()
        }

        // Placeholder right now (Dummy token A & B to return)
        let amount_a = (liquidity_to_remove / 1000) as u64;
        let amount_b = (liquidity_to_remove / 1000) as u64;

        // CPI transfer token A back to the user
        let pool_key = pool.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"signer", pool_key.as_ref(), &[pool.signer_bump]]];

        let cpi_ctx_a = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_a, amount_a)?;

        // Cpi Transfer token B back to user
        let cpi_ctx_b = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_b, amount_b)?;

        // Update users position
        user_position.liquidity = user_position
            .liquidity
            .checked_sub(liquidity_to_remove)
            .unwrap();

        msg!(
            "Liquidity removed: {} → A: {}, B: {} | Remaining: {}",
            liquidity_to_remove,
            amount_a,
            amount_b,
            user_position.liquidity
        );

        Ok(())
    }

    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        let pool = &ctx.accounts.pool_clmm;
        let user_position = &mut ctx.accounts.user_position;

        // Assuming 10 tokens of A and B
        let earned_a: u64 = 10;
        let earned_b: u64 = 10;

        // PDA Signer setup
        let pool_key = pool.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"signer", pool_key.as_ref(), &[pool.signer_bump]]];

        // Transfer A
        let cpi_ctx_a = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_a, earned_a)?;

        // Transfer B
        let cpi_ctx_b = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_b, earned_b)?;

        // Update fee checkpoints (placeholder — real values will come from pool)
        user_position.fee_growth_checkpoint_a = pool.fee_growth_global_a;
        user_position.fee_growth_checkpoint_b = pool.fee_growth_global_b;

        msg!("Fees collected: {} A and {} B", earned_a, earned_b);

        Ok(())
    }

    pub fn swap_clmm(
        ctx: Context<SwapClmm>,
        amount_in: u64,
        min_amount_out: u64,
        a_to_b: bool,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool_clmm;

        // Fee handlding
        let fee_rate = pool.fee_rate as u64;
        let fee_amount = amount_in
            .checked_mul(fee_rate)
            .unwrap()
            .checked_div(10_000)
            .unwrap();

        let amount_in_after_fee = amount_in.checked_sub(fee_amount).unwrap();
        let amount_out = amount_in_after_fee; // 1:1 for now

        // Fee growth update
        if pool.liquidity > 0 {
            let fee_growth = ((fee_amount as u128) << 64) / pool.liquidity;
            if a_to_b {
                pool.fee_growth_global_a =
                    pool.fee_growth_global_a.checked_add(fee_growth).unwrap();
            } else {
                pool.fee_growth_global_b =
                    pool.fee_growth_global_b.checked_add(fee_growth).unwrap();
            }
        }
        // --- Token Transfer In: From User → Vault ---
        let cpi_ctx_in = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_source_token.to_account_info(),
                to: ctx.accounts.vault_source.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_in, amount_in)?;

        // --- Token Transfer Out: From Vault → User (Signer PDA) ---
        let pool_key = pool.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"signer", pool_key.as_ref(), &[ctx.bumps.pool_signer]]];

        let cpi_ctx_out = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_destination.to_account_info(),
                to: ctx.accounts.user_destination_token.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_out, amount_out)?;

        // --- Validation ---
        require!(amount_out >= min_amount_out, CustomError::SlippageExceeded);

        msg!(
            "Swapped {} (A→B: {}) → {}, fee: {}",
            amount_in,
            a_to_b,
            amount_out,
            fee_amount
        );

        Ok(())
    }

    pub fn close_position(
        ctx: Context<ClosePosition>,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        let user_position = &ctx.accounts.user_position;

        // Only allow closing if all liquidity is removed
        require!(
            user_position.liquidity == 0,
            CustomError::PositionStillHasLiquidity
        );

        msg!(
            "Position closed for user: {} | tick range: [{} - {}]",
            ctx.accounts.owner.key(),
            tick_lower,
            tick_upper
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

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdatePoolConfig<'info> {
    #[account(mut, has_one = authority)]
    pub staking_pool: Account<'info, StakingPool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitEscrow<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(
        mut,
        constraint = initializer_deposit_token_account.owner == initializer.key(),
        constraint = initializer_deposit_token_account.mint == mint_offered.key()
    )]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint_offered: Account<'info, Mint>,

    #[account(mut)]
    pub mint_expected: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"escrow", initializer.key().as_ref()],
        bump,
        space = Escrow::LEN
    )]
    pub escrow: Account<'info, Escrow>,

    // This vault account is PDA ATA that holds initializers offered tokens
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_offered,
        associated_token::authority = vault_authority
    )]
    pub vault_amount: Account<'info, TokenAccount>,

    /// CHECK: PDA used to own vault_account
    #[account(
        seeds = [b"vault-authority", initializer.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FulfillEscrow<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut, constraint = taker_payment_ata.owner == taker.key())]
    pub taker_payment_ata: Account<'info, TokenAccount>,

    #[account(mut, constraint = initializer_receive_ata.owner == escrow.initializer)]
    pub initializer_receive_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_amount: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"vault-authority", escrow.initializer.as_ref()],
        bump
    )]
    /// CHECK: Vault authority PDA for signing transfer
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub taker_receive_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"escrow", escrow.initializer.as_ref()],
        bump,
        has_one = vault_amount @ CustomError::Unauthorized,
        constraint = !escrow.is_fulfilled @ CustomError::AlreadyFulfilled
    )]
    pub escrow: Account<'info, Escrow>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut)]
    pub initializer_receive_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_amount: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"vault-authority", initializer.key().as_ref()],
        bump
    )]
    /// CHECK: PDA signer
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"escrow", initializer.key().as_ref()],
        bump = escrow.bump,
        constraint = !escrow.is_fulfilled @ CustomError::AlreadyFulfilled,
        has_one = initializer @ CustomError::Unauthorized
    )]
    pub escrow: Account<'info, Escrow>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_input_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_output_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub input_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub output_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"vault", pool.key().as_ref(), output_token_mint.key().as_ref()],
        bump
    )]
    /// CHECK: PDA signer for output_token_mint auth
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub output_token_mint: Account<'info, Mint>, // Used only as seed for PDA

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [b"vault-authority", pool.key().as_ref()],
        bump
    )]
    /// CHECK: PDA for mint authority
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [b"vault-authority", pool.key().as_ref()],
        bump
    )]
    /// CHECK: PDA used as vault authority
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(owners: Vec<Pubkey>)]
pub struct CreateMultisig<'info> {
    #[account(
        init,
        seeds = [b"multisig", payer.key().as_ref()],
        bump,
        payer = payer,
        space = Multisig::LEN
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(tx_nonce: u8)]
pub struct ProposeTransaction<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"multisig", proposer.key().as_ref()],
        bump
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(
        init,
        payer = proposer,
        space = Transaction::LEN,
        seeds = [b"tx", multisig.key().as_ref(), &[tx_nonce]],
        bump
    )]
    pub tx: Account<'info, Transaction>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveTransaction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [b"multisig", signer.key().as_ref()],
        bump
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(
        mut,
        seeds = [b"tx", multisig.key().as_ref()],
        bump,
        constraint = !tx.did_execute @ CustomError::AlreadyExecuted
    )]
    pub tx: Account<'info, Transaction>,
}

#[derive(Accounts)]
#[instruction(tx_nonce: u8)]
pub struct ExecuteTransaction<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(
        seeds = [b"multisig", executor.key().as_ref()],
        bump
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(
        mut,
        seeds = [b"tx", multisig.key().as_ref(), &[tx_nonce]],
        bump,
        constraint = !tx.did_execute @ CustomError::AlreadyExecuted,
        constraint = tx.multisig == multisig.key() @ CustomError::Unauthorized,
    )]
    pub tx: Account<'info, Transaction>,
}

#[derive(Accounts)]
#[instruction(tick_spacing: u16)]
pub struct InitPoolClmm<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Pool state account (PDA)
    #[account(
        init,
        payer = payer,
        seeds = [b"pool-clmm", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        space = PoolClmm::LEN
    )]
    pub pool_clmm: Account<'info, PoolClmm>,

    /// Token A and B mints
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    /// Vaults to hold A/B liquidity
    #[account(
        init,
        payer = payer,
        token::mint = token_a_mint,
        token::authority = pool_signer
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = token_b_mint,
        token::authority = pool_signer
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// PDA that signs for the vaults
    #[account(
        seeds = [b"signer", pool_clmm.key().as_ref()],
        bump
    )]
    /// CHECK: Used only as a PDA signer
    pub pool_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(tick_index: i32)]
pub struct InitTick<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The CLMM pool this tick belongs to
    pub pool_clmm: Account<'info, PoolClmm>,

    #[account(
        init,
        payer = payer,
        seeds = [b"tick", pool_clmm.key().as_ref(), &tick_index.to_le_bytes()],
        bump,
        space = Tick::LEN
    )]
    pub tick: Account<'info, Tick>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_clmm: Account<'info, PoolClmm>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"signer", pool_clmm.key().as_ref()],
        bump = pool_clmm.signer_bump
    )]
    /// CHECK: Only used for CPI signing
    pub pool_signer: UncheckedAccount<'info>,

    #[account(mut)]
    tick_lower: Account<'info, Tick>,
    #[account(mut)]
    tick_upper: Account<'info, Tick>,

    #[account(
        init,
        payer = owner,
        seeds = [
            b"position",
            owner.key().as_ref(),
            pool_clmm.key().as_ref(),
            &tick_lower.tick_index.to_le_bytes(),
            &tick_upper.tick_index.to_le_bytes(),
        ],
        bump,
        space = Position::LEN
    )]
    pub user_position: Account<'info, Position>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DecreaseLiquidity<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool for which the position was opened
    #[account(
        mut,
        has_one = vault_a,
        has_one = vault_b
    )]
    pub pool_clmm: Account<'info, PoolClmm>,

    /// Position owned by the user
    #[account(
        mut,
        seeds = [
            b"position",
            owner.key().as_ref(),
            pool_clmm.key().as_ref(),
            &tick_lower.tick_index.to_le_bytes(),
            &tick_upper.tick_index.to_le_bytes(),
        ],
        bump,
        has_one = owner,
        has_one = pool_clmm
    )]
    pub user_position: Account<'info, Position>,

    /// Lower tick account
    #[account(
        mut,
        seeds = [b"tick", pool_clmm.key().as_ref(), &tick_lower.tick_index.to_le_bytes()],
        bump
    )]
    pub tick_lower: Account<'info, Tick>,

    /// Upper tick account
    #[account(
        mut,
        seeds = [b"tick", pool_clmm.key().as_ref(), &tick_upper.tick_index.to_le_bytes()],
        bump
    )]
    pub tick_upper: Account<'info, Tick>,

    /// Vault A (Token A liquidity vault)
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    /// Vault B (Token B liquidity vault)
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    /// Vault signer PDA (pool auth)
    #[account(
        seeds = [b"signer", pool_clmm.key().as_ref()],
        bump
    )]
    /// CHECK: PDA auth only
    pub pool_signer: UncheckedAccount<'info>,

    /// Users receiving token accounts
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool for which fees are collected
    #[account(
        mut,
        has_one = vault_a,
        has_one = vault_b
    )]
    pub pool_clmm: Account<'info, PoolClmm>,

    /// Users position account
    #[account(
        mut,
        seeds = [
            b"position",
            owner.key().as_ref(),
            pool_clmm.key().as_ref(),
            &tick_lower.tick_index.to_le_bytes(),
            &tick_upper.tick_index.to_le_bytes(),
        ],
        bump,
        has_one = owner,
        has_one = pool_clmm
    )]
    pub user_position: Account<'info, Position>,

    /// Lower and upper tick accounts (used for future fee tracking logic)
    #[account(
        seeds = [b"tick", pool_clmm.key().as_ref(), &tick_lower.tick_index.to_le_bytes()],
        bump
    )]
    pub tick_lower: Account<'info, Tick>,

    #[account(
        seeds = [b"tick", pool_clmm.key().as_ref(), &tick_upper.tick_index.to_le_bytes()],
        bump
    )]
    pub tick_upper: Account<'info, Tick>,

    /// Vault that hold fees
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    /// PDA signer that controls the vaults
    #[account(
        seeds = [b"signer", pool_clmm.key().as_ref()],
        bump
    )]
    /// CHECK: PDA signer only
    pub pool_signer: UncheckedAccount<'info>,

    /// User receiving accounts
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapClmm<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_clmm: Account<'info, PoolClmm>,

    #[account(mut)]
    pub user_source_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_destination_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_destination: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"signer", pool_clmm.key().as_ref()],
        bump
    )]
    /// CHECK: PDA auth only
    pub pool_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(tick_lower: i32, tick_upper: i32)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool the position belongs to
    #[account(mut)]
    pub pool_clmm: Account<'info, PoolClmm>,

    /// Users position to close
    #[account(
        mut,
        close = owner,
        seeds = [
            b"position",
            owner.key().as_ref(),
            pool_clmm.key().as_ref(),
            &tick_lower.to_le_bytes(),
            &tick_upper.to_le_bytes()
        ],
        bump,
        has_one = owner,
        has_one = pool_clmm
    )]
    pub user_position: Account<'info, Position>,
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
    pub id: u64,             // unique per profile
    pub profile: Pubkey,     // creator profile pda
    pub title: String,       // 4 + N
    pub description: String, // 4 + M
    pub created_at: i64,     // UNIX timestamp
    pub deadline: i64,       // voting end
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
    pub owner: Pubkey,   // 32 bytes
    pub profile: Pubkey, // 32 bytes
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,           // 8 bytes
    pub reward_collected: u64, // 8 bytes
    pub start_stake_time: i64, // 8 bytes
    pub bump: u8,              // 1 byte
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

#[account]
pub struct Escrow {
    pub initializer: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub vault_amount: Pubkey,
    pub mint_offered: Pubkey,
    pub mint_expected: Pubkey,
    pub amount_offered: u64,
    pub amount_expected: u64,
    pub unlock_at: i64,
    pub is_fulfilled: bool,
    pub bump: u8,
}

impl Escrow {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 1 + 1;
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

#[account]
pub struct Multisig {
    pub owners: Vec<Pubkey>,  // List of valid signers
    pub threshold: u8,        // Min approvals required
    pub bump: u8,             // PDA bump for vault authority
    pub owner_set_seqno: u32, // Tracks changes to owner set
}

impl Multisig {
    pub const MAX_OWNERS: usize = 10;

    pub const LEN: usize = 8 + 4 + (32 * Self::MAX_OWNERS) + 1 + 1 + 4;
}

#[account]
pub struct Transaction {
    pub multisig: Pubkey,
    pub program_id: Pubkey,
    pub accounts: Vec<TransactionAccount>,
    pub data: Vec<u8>,
    pub did_execute: bool,
    pub signers: [bool; 10],
    pub owner_set_repo: u64,
    pub owner_set_seqno: u32,
}

impl Transaction {
    pub const MAX_ACCOUNTS: usize = 10;
    pub const MAX_DATA: usize = 512;

    pub const LEN: usize = 8
        + 32
        + 32
        + 4
        + TransactionAccount::LEN * Self::MAX_ACCOUNTS
        + 4
        + Self::MAX_DATA
        + 1
        + 10
        + 4;
}

// Anchor doesn't allow serializing raw AccountMeta
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl TransactionAccount {
    pub const LEN: usize = 32 + 1 + 1;
}

#[account]
pub struct PoolClmm {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,

    pub sqrt_price_x64: u128, // sqrt(p), stored in fixed-point
    pub current_tick: i32,
    pub tick_spacing: u16,

    pub liquidity: u128, // active liquidity at current tick
    pub fee_rate: u16,   // eg 30 = 0.3%

    pub fee_growth_global_a: u128,
    pub fee_growth_global_b: u128,

    pub bump: u8,
    pub signer_bump: u8,
}

impl PoolClmm {
    pub const LEN: usize = 8 +     // Anchor discriminator
        32 + // token_a_mint
        32 + // token_b_mint
        32 + // vault_a
        32 + // vault_b
        16 + // sqrt_price_x64
        4  + // current_tick
        2  + // tick_spacing
        16 + // liquidity
        2  + // fee_rate
        16 + // fee_growth_global_a
        16 + // fee_growth_global_b
        1  + // bump
        1; // signer_bump
}

#[account]
pub struct Tick {
    pub tick_index: i32,     // This ticks the index (-200, 0 , 100)
    pub liquidity_net: i128, // +liquidity if lower_tick, -liquidity if upper_tick
    pub fee_growth_outside_a: u128,
    pub fee_growth_outside_b: u128,
}

impl Tick {
    pub const LEN: usize = 8 +     // Anchor discriminator
        4 +  // tick_index (i32)
        16 + // liquidity_net (i128)
        16 + // fee_growth_outside_a (u128)
        16; // fee_growth_outside_b (u128)
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub pool_clmm: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub fee_growth_checkpoint_a: u128,
    pub fee_growth_checkpoint_b: u128,
}

impl Position {
    pub const LEN: usize = 8 +     // Anchor discriminator
        32 + // owner
        32 + // pool_clmm
        4 +  // tick_lower
        4 +  // tick_upper
        16 + // liquidity
        16 + // fee_growth_checkpoint_a
        16; // fee_growth_checkpoint_b
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

    #[msg("The escrow transaction has already been fulfilled")]
    AlreadyFulfilled,

    #[msg("The tx cant happen due to slippage limits")]
    SlippageExceeded,

    #[msg("No liquidity is present")]
    NoLiquidity,

    #[msg("Too many owners. The maximum number of owners allowed in a multisig is 10.")]
    TooManyOwners,

    #[msg("Invalid threshold. Must be greater than 0 and less than or equal to number of owners.")]
    InvalidThreshold,

    #[msg("Only a multisig owner can propose a transaction")]
    NotAnOwner,

    #[msg("Transaction already signed by this owner")]
    AlreadySigned,

    #[msg("Transaction already executed")]
    AlreadyExecuted,

    #[msg("Not enough signatures to execute this transaction")]
    InsufficientSignatures,

    #[msg("Invalid tick range.")]
    InvalidTickRange,

    #[msg("Liquidity must be greater than zero.")]
    ZeroLiquidity,

    #[msg("Insufficient liquidity to remove")]
    InvalidLiquidityAmount,

    #[msg("Liquidity is still present in the pool")]
    PositionStillHasLiquidity,
}

// Utitility fns
fn integer_sqrt(value: u64) -> u64 {
    (value as f64).sqrt() as u64
}

fn update_fee_growth_global(pool: &mut Account<PoolClmm>, fee_amount_a: u64, fee_amount_b: u64) {
    if pool.liquidity > 0 {
        let fee_growth_a = (fee_amount_a as u128) << 64 / pool.liquidity;
        let fee_growth_b = (fee_amount_b as u128) << 64 / pool.liquidity;

        pool.fee_growth_global_a = pool.fee_growth_global_a.checked_add(fee_growth_a).unwrap();

        pool.fee_growth_global_b = pool.fee_growth_global_b.checked_add(fee_growth_b).unwrap();
    }
}
