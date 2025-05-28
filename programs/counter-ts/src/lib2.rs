use anchor_lang::prelude::*;

declare_id!("74QZ1uTUKCPsao19wAtRRxxQ441PeejhkAZBH7nw9EEN");

#[program]
pub mod counter_ts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter_account = &mut ctx.accounts.counter_account;
        counter_account.count = 0;
        counter_account.owner = ctx.accounts.user.key(); 
        Ok(())
    }

    pub fn increment(ctx: Context<UpdateCounter>) -> Result<()> {
        let counter_account = &mut ctx.accounts.counter_account;

        require_keys_eq!(
            ctx.accounts.user.key(),
            counter_account.owner,
            CustomError::Unauthorized
        );

        counter_account.count += 1;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 32)] 
    pub counter_account: Account<'info, Counter>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    #[account(mut)]
    pub counter_account: Account<'info, Counter>,
    pub user: Signer<'info>, 
}

#[account]
pub struct Counter {
    pub count: i64,
    pub owner: Pubkey,
}

#[error_code]
pub enum CustomError {
    #[msg("Only the owner can increment the counter")]
    Unauthorized,
}
