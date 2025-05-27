use anchor_lang::prelude::*;  //importing everything from the anchor

declare_id!("74QZ1uTUKCPsao19wAtRRxxQ441PeejhkAZBH7nw9EEN");

#[program]
pub mod counter_ts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter_account =  &mut ctx.accounts.counter_account ;
        counter_account.count = 0 ;
        
        Ok(())
    }

    pub fn incerment(ctx: Context<UpdateCounter>) -> Result<()> {
       let counter_account = &mut ctx.accounts.counter_account ; 
       counter_account.count +=1 ;
       Ok(())
    }
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub counter_account: Account<'info, Counter>,
    #[account(mut)]
    pub user: Signer<'info>,   // using the user to sign and pay the rent 
    pub system_program: Program<'info, System>   //When your program needs to create/init an account (like counter_account), it calls the System Program internally.
}

#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    #[account(mut)]
    pub counter_account : Account<'info, Counter>,
}

#[account] 
pub struct Counter {
    pub count :i64,
}
