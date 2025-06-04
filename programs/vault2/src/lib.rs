use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer, Mint};
use std::str::FromStr;

declare_id!("2vo1Sdq39gUPV1GoivRXz8t7tqsCcaa8WiQ3AeZhHynE");

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // Correct mainnet USDC mint

#[program]
pub mod vault_version2{
    use super::*;

    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;

        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);
        } else {
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
            depositor.usdc_mint = Pubkey::from_str(USDC_MINT).unwrap();
        }

        depositor.sol_amount = depositor.sol_amount.checked_add(amount).ok_or(CustomError::MathOverflow)?;
        depositor.deposit_time = Clock::get()?.unix_timestamp;

        // Use system program transfer instead of manual lamport manipulation
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault_pda.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault_pda.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> Result<()> {
        let depositor = &mut ctx.accounts.depositor;

        if depositor.is_initialized {
            require_keys_eq!(depositor.owner, ctx.accounts.user.key(), CustomError::Unauthorized);
        } else {
            depositor.owner = *ctx.accounts.user.key;
            depositor.is_initialized = true;
            depositor.usdc_mint = Pubkey::from_str(USDC_MINT).unwrap();
        }

        // Validate mint against the actual USDC mint account
        require_keys_eq!(ctx.accounts.usdc_mint.key(), Pubkey::from_str(USDC_MINT).unwrap(), CustomError::InvalidMint);
        require_keys_eq!(ctx.accounts.user_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);
        require_keys_eq!(ctx.accounts.vault_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);

        // Transfer USDC
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

            // Validate mint
            require_keys_eq!(ctx.accounts.usdc_mint.key(), Pubkey::from_str(USDC_MINT).unwrap(), CustomError::InvalidMint);
            require_keys_eq!(ctx.accounts.user_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);
            require_keys_eq!(ctx.accounts.vault_usdc_account.mint, ctx.accounts.usdc_mint.key(), CustomError::InvalidMint);

            let bump = ctx.bumps.vault_usdc_account;
            let seeds = &[b"vault_pda".as_ref(), &[bump]];
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
pub struct DepositSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 1 + 8 + 8 + 32 + 8,
        seeds = [b"depositor", user.key().as_ref()],
        bump
    )]
    pub depositor: Account<'info, Depositor>,

    #[account(mut, seeds = [b"vault_pda"], bump)]
    /// CHECK: PDA for holding SOL
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
        space = 8 + 32 + 1 + 8 + 8 + 32 + 8,
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
pub struct Depositor {
    pub owner: Pubkey,           // 32 bytes
    pub is_initialized: bool,    // 1 byte  
    pub sol_amount: u64,         // 8 bytes
    pub usdc_amount: u64,        // 8 bytes
    pub usdc_mint: Pubkey,       // 32 bytes
    pub deposit_time: i64,       // 8 bytes
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Math overflow")]
    MathOverflow,
}