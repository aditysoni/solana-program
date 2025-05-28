use anchor_lang::prelude::*;

declare_id!("Your_Program_ID_Here");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = *ctx.accounts.owner.key;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;
       
        // Transfer SOL from user to vault
        **ctx.accounts.user.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.vault_account.try_borrow_mut_lamports()? += amount;

        depositor.amount += amount;
        depositor.deposit_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;
        let amount = depositor.amount;

        **ctx.accounts.vault_account.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

        depositor.amount = 0;
        Ok(())
    }
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + 32)]
    pub vault: Account<'info, VaultAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 8 + 8,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,
    #[account(mut)]
    pub vault_account: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,
    #[account(mut)]
    pub vault_account: SystemAccount<'info>,
}


#[account]
pub struct VaultAccount {
    pub owner: Pubkey,
}

#[account]
pub struct Depositor {
    pub amount: u64,
    pub deposit_time: i64,
}
