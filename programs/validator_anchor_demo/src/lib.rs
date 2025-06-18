use anchor_lang::prelude::*;

declare_id!("BH2vhWg3AJqKn5VXKf6nepTPQUigJEhPEApUo9XXekjz");

#[program]
pub mod validator_anchor_demo {
    use super::*;

    // Create a new validator as a PDA owned by the program
    pub fn init_validator(ctx: Context<InitValidator>, id: u64, name: String) -> Result<()> {
        let validator = &mut ctx.accounts.validator;
        validator.id = id;
        validator.name = name;
        validator.is_active = true;
        validator.authority = ctx.accounts.authority.key();
        validator.bump = ctx.bumps.validator;
        Ok(())
    }

    // Update PDA validator if caller is authority
    pub fn update_validator(ctx: Context<UpdateValidator>, new_name: String, is_active: bool) -> Result<()> {
        let validator = &mut ctx.accounts.validator;
        validator.name = new_name;
        validator.is_active = is_active;
        Ok(())
    }

    // Close PDA validator and refund rent to the authority
    pub fn close_validator(ctx: Context<CloseValidator>) -> Result<()> {
        Ok(())
    }

    // Create a PDA based profile for the user
    pub fn init_profile(ctx: Context<InitProfile>, name: String) -> Result<()> {
        let profile = &mut ctx.accounts.profile;
        profile.authority = ctx.accounts.authority.key();
        profile.name = name;
        profile.bump = ctx.bumps.profile;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(id: u64, name: String)]
pub struct InitValidator<'info> {
    #[account(
        init,
        seeds = [b"validator", authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + 8 + 4 + 32 + 1 + 32 + 1 // discriminator + u64 + string + bool + pubkey + dump
    )]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateValidator<'info> {
    #[account(
        mut,
        seeds = [b"validator", authority.key().as_ref()],
        bump = validator.bump,
        has_one = authority,
    )]
    pub validator: Account<'info, ValidatorInfo>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseValidator<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"validator",authority.key().as_ref()],
        bump = validator.bump,
        has_one = authority,
    )]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct ValidatorInfo {
    pub id: u64,
    pub name: String,
    pub is_active: bool,
    pub authority: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitProfile<'info> {
    #[account(
        init,
        seeds = [b"profile",authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + 32 + 4 + 32 + 1
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserProfile {
    pub authority: Pubkey,
    pub name: String,
    pub bump: u8,
}