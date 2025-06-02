use anchor_lang::prelude::*;


declare_id!("HDhkebca19sS5qcas1DXkCQJoxN6upiEvc8wYZvFp4y7");

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

        // Guard: check signer matches depositor.owner pubkey if already initialized
        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);
        } else {
            // First-time init: set owner and mark initialized
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
        }

        // Transfer SOL from user to vault PDA
        **ctx.accounts.user.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.vault_pda.to_account_info().try_borrow_mut_lamports()? += amount;

        depositor.amount = depositor.amount.checked_add(amount).unwrap();
        depositor.deposit_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;

        // Guard: ensure only owner can withdraw
        require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);

        let amount = depositor.amount;

        **ctx.accounts.vault_pda.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

        depositor.amount = 0;
        Ok(())
    }

    pub fn owner_withdraw(ctx: Context<OwnerWithdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;

        // Only the vault owner can withdraw
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
        space = 8 + 32 + 1 + 8 + 8, // discriminator + owner pubkey + is_initialized (bool) + amount + deposit_time
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,

    #[account(
        mut,
        seeds = [b"vault_pda"],
        bump
    )]
    /// CHECK: PDA used to store lamports
    pub vault_pda: AccountInfo<'info>,

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

    #[account(
        mut,
        seeds = [b"vault_pda"],
        bump
    )]
    /// CHECK: PDA used to store lamports
    pub vault_pda: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct OwnerWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, VaultAccount>,

    #[account(
        mut,
        seeds = [b"vault_pda"],
        bump
    )]
    /// CHECK: PDA used to store lamports
    pub vault_pda: AccountInfo<'info>,
}

#[account]
pub struct VaultAccount {
    pub owner: Pubkey,
}

#[account]
pub struct Depositor {
    pub owner: Pubkey,       // user public key
    pub is_initialized: bool,
    pub amount: u64,
    pub deposit_time: i64,
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized action")]
    Unauthorized,
}
