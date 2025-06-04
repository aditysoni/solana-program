use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("2vo1Sdq39gUPV1GoivRXz8t7tqsCcaa8WiQ3AeZhHynE");

#[program]
pub mod vault_version2 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = *ctx.accounts.owner.key;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;

        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);
        } else {
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
        }

        **ctx.accounts.user.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.vault_pda.to_account_info().try_borrow_mut_lamports()? += amount;

        depositor.amount = depositor.amount.checked_add(amount).unwrap();
        depositor.deposit_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    // New deposit_usdc function
    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;

        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);
        } else {
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
        }

        // Transfer USDC tokens from user's token account to vault's token account
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.vault_usdc_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        depositor.amount = depositor.amount.checked_add(amount).unwrap();
        depositor.deposit_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;

        require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);

        let amount = depositor.amount;

        **ctx.accounts.vault_pda.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

        depositor.amount = 0;
        Ok(())
    }

    pub fn owner_withdraw(ctx: Context<OwnerWithdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;

        require_keys_eq!(vault.owner, ctx.accounts.owner.key(), CustomError::Unauthorized);

        **ctx.accounts.vault_pda.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.owner.try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + 32, seeds = [b"vault"], bump)]
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
        space = 8 + 32 + 1 + 8 + 8,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,

    #[account(mut, seeds = [b"vault_pda"], bump)]
    /// CHECK: PDA holding lamports
    pub vault_pda: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositUsdc<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 1 + 8 + 8,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,

    /// CHECK: Vault's USDC token account (PDA)
    #[account(mut,
        seeds = [b"vault_usdc_account"],
        bump
    )]
    pub vault_usdc_account: AccountInfo<'info>,

    /// CHECK: User's USDC token account (SPL Token account)
    #[account(mut,
        constraint = user_usdc_account.owner == user.key(),
        constraint = user_usdc_account.mint == mint.key(),
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,  // USDC Mint address

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"depositor", user.key().as_ref()], bump)]
    pub depositor: Account<'info, Depositor>,

    #[account(mut, seeds = [b"vault_pda"], bump)]
    /// CHECK: PDA holding lamports
    pub vault_pda: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct OwnerWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(seeds = [b"vault"], bump)]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut, seeds = [b"vault_pda"], bump)]
    /// CHECK: PDA holding lamports
    pub vault_pda: AccountInfo<'info>,
}

#[account]
pub struct VaultAccount {
    pub owner: Pubkey,
}

#[account]
pub struct Depositor {
    pub owner: Pubkey,
    pub is_initialized: bool,
    pub amount: u64,
    pub deposit_time: i64,
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized action")]
    Unauthorized,
}
