use anchor_lang::prelude::*;

declare_id!("BH2vhWg3AJqKn5VXKf6nepTPQUigJEhPEApUo9XXekjz");

#[program] //#[program] tells Anchor: “This is the entry point for external calls (instructions)”
pub mod validator_anchor_demo {
    use super::*;

    pub fn create_validator(ctx: Context<CreateValidator>, id: u64, name: String) -> Result<()> {
        let validator = &mut ctx.accounts.validator;
        validator.id = id;
        validator.name = name;
        validator.is_active = true;
        Ok(())
    }

    // Update validator name and active status
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

    // Close account and refund rent to the user
    pub fn close_validator(ctx: Context<DeleteValidator>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)] //#[derive(Accounts)] means: "This struct defines the accounts needed to run the instruction"
pub struct CreateValidator<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 8 + 32 + 1 // discriminator(Anchor adds a hidden discriminator (used to identify the struct type)) + u64 + string(32max) + bool
    )]
    pub validator: Account<'info, ValidatorInfo>,
    
    #[account[mut]]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateValidator<'info> {
    #[account[mut]]
    pub validator: Account<'info, ValidatorInfo>,
}

#[derive(Accounts)]
pub struct DeleteValidator<'info> {
    #[account[mut, close = refund_to]]
    pub validator: Account<'info, ValidatorInfo>,

    #[account[mut]]
    pub refund_to: Signer<'info>
}

#[account] //Actual Struct Stored On-Chain
pub struct ValidatorInfo {
    pub id: u64,
    pub name: String,
    pub is_active: bool,
}


use anchor_lang::prelude::*;

declare_id!("BH2vhWg3AJqKn5VXKf6nepTPQUigJEhPEApUo9XXekjz");

#[program] //#[program] tells Anchor: “This is the entry point for external calls (instructions)”
pub mod validator_anchor_demo {
    use super::*;

    // Create new validator account with id and name
    pub fn create_validator(ctx: Context<CreateValidator>, id: u64, name: String) -> Result<()>{
        let validator = &mut ctx.accounts.validator;
        validator.id = id;
        validator.name = name;
        validator.is_active = true;
        Ok(())
    }

    // Update validator name and active status
    pub fn update_validator(ctx: Context<UpdateValidator>, new_name: String, is_active: bool) -> Result<()> {
        let validator = &mut ctx.accounts.validator;
        validator.name = new_name;
        validator.is_active = is_active;
        Ok(())
    }

    // Close account and refund rent to the user
    pub fn close_validator(ctx: Context<CloseValidator>) -> Result<()> {
        Ok(())
    }

    // Creating a PDA-based User Profile
    pub fn init_profile(ctx: Context<InitProfile>, name: String) -> Result<()> {
        let profile = &mut ctx.accounts.profile;

        // Store authority (user who owns the profile)
        profile.authority = ctx.accounts.authority.key();

        // Save user-chosen name
        profile.name = name;

        // Store bump wo we can sign this PDA in future
        profile.bump = ctx.bumps.profile;

        Ok(())
    }
}

#[derive(Accounts)] 
pub struct CreateValidator<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 8 + 32 + 1 // discriminator + id: u64 + name: String + is_active: bool
    )]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateValidator<'info> {
    #[account(mut)]
    pub validator: Account<'info, ValidatorInfo>,
}

#[derive(Accounts)]
pub struct CloseValidator<'info> {
    #[account(mut, close = refund_to)]
    pub validator: Account<'info, ValidatorInfo>,

    #[account(mut)]
    pub refund_to: Signer<'info>,
}

#[account]
pub struct ValidatorInfo {
    pub id: u64,
    pub name: String,
    pub is_active: bool,
}

// New PDA Profile Account

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitProfile<'info> {
    #[account(
        init,
        seeds = [b"profile", authority.key().as_ref()], // seed: ["profile", authority_pubkey]
        bump, // bump auto found by Anchor
        payer = authority, // payer for account creation
        space = 8 + 32 + 4  + 32 + 1 // discriminator + pubkey + name string + bump
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub authority: Signer<'info>, // users wallet signs the tx

    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserProfile {
    pub authority: Pubkey, // owner of the profile
    pub name: String, // name with max 32 chars
    pub bump: u8, // bump used to create the PDA
}