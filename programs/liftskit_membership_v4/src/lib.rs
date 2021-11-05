use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
// use std::time::{SystemTime, UNIX_EPOCH};
// use std::time::{SystemTime, UNIX_EPOCH};

declare_id!("Gqsr3fjeaVBWNrr4NDetcZaNZYNWxdiN3EQZy5wtzpH9");

#[program]
pub mod liftskit_membership_v4 {
    use super::*;

    pub fn create(ctx: Context<Create>) -> ProgramResult {
        let membership_account = &mut ctx.accounts.membership_account;
        let address = membership_account.to_account_info().key;
        membership_account.members.push(*address);

        // TODO: FIX THIS!!!
        membership_account.last_paid = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }

    pub fn add_member(ctx: Context<AddMember>, member_address: Pubkey) -> ProgramResult {
        let membership_account = &mut ctx.accounts.membership_account;
        membership_account.members.push(member_address);
        Ok(())
    }

    pub fn pay_user(ctx: Context<PayUser>, interaction_fee: u64) -> ProgramResult {
        let secs_in_a_day = 86400;
        let current_time = Clock::get().unwrap().unix_timestamp;

        let membership_account = &mut ctx.accounts.membership_account;

        // update today function
        if membership_account.last_paid < current_time - secs_in_a_day {

            // clear out the paid_today
            membership_account.last_paid = current_time;
            membership_account.paid_today = Vec::new();
        }
        
        // 1. check if they are a member
        if membership_account.members.contains(ctx.accounts.to.to_account_info().key) {
            msg!("User IS A MEMBER!");

            // 2. check they were not already paid today
            if !membership_account.paid_today.contains(ctx.accounts.to.to_account_info().key) {

                // create cross-program-invocation to run the token txns
                let cpi_accounts = Transfer {
                    from: ctx.accounts.from.to_account_info().clone(),
                    to: ctx.accounts.to.to_account_info().clone(),
                    authority: ctx.accounts.owner.clone(),
                };
        
                let cpi_program = ctx.accounts.token_program.clone();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, interaction_fee)?;
    
                // add member to paid_today list
                membership_account.paid_today.push(*ctx.accounts.to.to_account_info().key);
            }
        }
       
        Ok(())
    }
}

// Transaction instructions
#[derive(Accounts)]
pub struct Create<'info> {
    // membership account
    #[account(init, payer = user, space = 64 + 64 + 64)]
    pub membership_account: Account<'info, MembershipAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program <'info, System>,
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(mut)]
    pub membership_account: Account<'info, MembershipAccount>,
}

#[derive(Accounts)]
pub struct PayUser<'info> {
    #[account(mut, has_one = owner)]
    from: Account<'info, TokenAccount>,
    
    #[account(mut, "from.mint == to.mint")]
    to: Account<'info, TokenAccount>,
    
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,

    #[account(mut)]
    membership_account: Account<'info, MembershipAccount>,
}


// An account that goes inside a transaction instruction
#[account]
pub struct MembershipAccount {
    pub members: Vec<Pubkey>,
    pub paid_today: Vec<Pubkey>,

    pub last_paid: i64,
}

#[error]
pub enum ErrorCode {
    #[msg("The derived interaction signer does not match that which was given.")]
    InvalidInteractionSigner,
}