use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer, Mint};
use std::str::FromStr;

declare_id!("Havovdums4jVo6HwPj6iUSMLtfmaEHeBNhPBrDgDrWZy");

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[program]
pub mod factory {
    use super::*;
    
    pub fn initialize_factory(ctx: Context<InitializeFactory>) -> Result<()> {
        let factory = &mut ctx.accounts.factory;
        factory.owner = ctx.accounts.owner.key();
        factory.vault_count = 0;
        factory.vaults = Vec::new();
        factory.vault_managers = Vec::new();
        Ok(())
    }

    pub fn create_vault(ctx: Context<CreateVault>, manager: Pubkey) -> Result<()> {
        let factory = &mut ctx.accounts.factory;
        require_keys_eq!(ctx.accounts.owner.key(), factory.owner, CustomError::Unauthorized);
        require!(!factory.vault_managers.contains(&manager), CustomError::ManagerAlreadyExists);

        let vault = &mut ctx.accounts.vault;
        vault.manager = manager;
        vault.total_deposit = 0;
        vault.index = factory.vault_count;

        factory.vaults.push(vault.key());
        factory.vault_count += 1;
        factory.vault_managers.push(manager);

        Ok(())
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);
        
        let vault = &mut ctx.accounts.vault;
        let depositor = &mut ctx.accounts.depositor;
        let user = &mut ctx.accounts.user;

        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, user.key(), CustomError::Unauthorized);
        } else {
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
            depositor.vault_pda = vault.key();
            depositor.sol_amount = 0;
            depositor.usdc_amount = 0;
        }

        depositor.sol_amount = depositor.sol_amount.checked_add(amount).ok_or(CustomError::MathOverflow)?;
        depositor.deposit_time = Clock::get()?.unix_timestamp;
        
        vault.total_deposit += amount;

        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }

    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);
        
        let depositor = &mut ctx.accounts.depositor;
        let user = &mut ctx.accounts.user;

        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, user.key(), CustomError::Unauthorized);
        } else {
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
            depositor.vault_pda = ctx.accounts.vault_usdc_account.key();
            depositor.sol_amount = 0;
            depositor.usdc_amount = 0;
        }

        require_keys_eq!(ctx.accounts.usdc_mint.key(), Pubkey::from_str(USDC_MINT).unwrap(), CustomError::InvalidMint);
        require_keys_eq!(ctx.accounts.user_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.vault_usdc_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        depositor.usdc_amount = depositor.usdc_amount.checked_add(amount).ok_or(CustomError::MathOverflow)?;
        depositor.deposit_time = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, sol_amount: u64, usdc_amount: u64) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;
        require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);

        // Withdraw SOL
        if sol_amount > 0 {
            require!(depositor.sol_amount >= sol_amount, CustomError::InsufficientBalance);
            
            let bump = ctx.bumps.vault_pda;
            let seeds = &[b"vault_pda".as_ref(), &[bump]];
            let signer = &[&seeds[..]];

            let ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.vault_pda.key(),
                &ctx.accounts.user.key(),
                sol_amount,
            );
            anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[
                    ctx.accounts.vault_pda.to_account_info(),
                    ctx.accounts.user.to_account_info(),
                ],
                signer,
            )?;

            depositor.sol_amount = depositor.sol_amount.checked_sub(sol_amount).ok_or(CustomError::MathOverflow)?;
        }

        // Withdraw USDC
        if usdc_amount > 0 {
            require!(depositor.usdc_amount >= usdc_amount, CustomError::InsufficientBalance);

            require_keys_eq!(ctx.accounts.usdc_mint.key(), Pubkey::from_str(USDC_MINT).unwrap(), CustomError::InvalidMint);
            require_keys_eq!(ctx.accounts.user_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);
            require_keys_eq!(ctx.accounts.vault_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);

            let bump = ctx.bumps.vault_usdc_account;
            let seeds = &[b"vault_usdc_account".as_ref(), &[bump]];
            let signer = &[&seeds[..]];

            let cpi_accounts = Transfer {
                from: ctx.accounts.vault_usdc_account.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: ctx.accounts.vault_usdc_account.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
            token::transfer(cpi_ctx, usdc_amount)?;

            depositor.usdc_amount = depositor.usdc_amount.checked_sub(usdc_amount).ok_or(CustomError::MathOverflow)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeFactory<'info> {
    #[account(
        init, 
        seeds = [b"vault_factory"], 
        bump, 
        payer = owner, 
        space = 8 + 32 + 4 + 4 + (32 * 1000) + 4 + (32 * 1000)
    )]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(manager: Pubkey)]
pub struct CreateVault<'info> {
    #[account(mut, seeds = [b"vault_factory"], bump)]
    pub factory: Account<'info, Factory>,
    #[account(mut)] 
    pub owner: Signer<'info>,
    #[account(init, seeds = [b"vault", manager.as_ref()], bump, payer = owner, space = 8 + 32 + 8 + 4)]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 1 + 8 + 8 + 8 + 32,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,
    
    #[account(mut, seeds = [b"vault_pda"], bump)]
    /// CHECK: PDA for holding SOL
    pub vault_pda: AccountInfo<'info>,

    #[account(mut, seeds = [b"vault", vault.manager.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositUsdc<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 1 + 8 + 8 + 8 + 32,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,

    #[account(
        init_if_needed,
        payer = user,
        token::mint = usdc_mint,
        token::authority = vault_usdc_account,
        seeds = [b"vault_usdc_account"],
        bump
    )]
    pub vault_usdc_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint = usdc_mint)]
    pub user_usdc_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"depositor", user.key().as_ref()], bump)]
    pub depositor: Account<'info, Depositor>,

    #[account(mut, seeds = [b"vault_pda"], bump)]
    /// CHECK: Vault PDA holding SOL
    pub vault_pda: AccountInfo<'info>,

    #[account(mut, token::mint = usdc_mint, seeds = [b"vault_usdc_account"], bump)]
    pub vault_usdc_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint = usdc_mint)]
    pub user_usdc_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Factory {
    pub owner: Pubkey,
    pub vault_count: u32,
    pub vaults: Vec<Pubkey>,
    pub vault_managers: Vec<Pubkey>,
}

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub total_deposit: u64,
    pub index: u32,
    pub vault: Pubkey
}

#[account]
pub struct Depositor {
    pub owner: Pubkey,
    pub is_initialized: bool,
    pub deposit_time: i64,
    pub sol_amount: u64,
    pub usdc_amount: u64,
    pub vault_pda: Pubkey,
}

#[error_code]
pub enum CustomError {
    #[msg("Nothing to withdraw")]
    NothingToWithdraw,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Vault not found")]
    VaultNotFound,
    #[msg("Depositor already initialized")]
    DepositorAlreadyInitialized,
    #[msg("Manager already exists")]
    ManagerAlreadyExists,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Insufficient balance")]
    InsufficientBalance,
}