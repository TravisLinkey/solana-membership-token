use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

declare_id!("Gqsr3fjeaVBWNrr4NDetcZaNZYNWxdiN3EQZy5wtzpH9");

#[program]
pub mod liftskit_membership_v4 {
    use super::*;

    pub fn create(ctx: Context<Create>, member_address: Pubkey) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let address = base_account.to_account_info().key;
        base_account.members.push(*address);
        Ok(())
    }

    pub fn add_member(ctx: Context<AddMember>, member_address: Pubkey) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let address = base_account.to_account_info().key;
        base_account.members.push(*address);
        Ok(())
    }

    /**
     * @todo: only pay user on the membership list
     */
    pub fn pay_user(ctx: Context<PayUser>, interaction_fee: u64) -> ProgramResult {
        // create cross-program-invocation to run the token txns
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };

        // TODO :
        // 1. check if they are a member
        // 2. check they werent paid today

        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, interaction_fee)?;
       
        Ok(())
    }
}

// Transaction instructions
#[derive(Accounts)]
pub struct Create<'info> {
    // membership account
    #[account(init, payer = user, space = 64 + 64)]
    pub base_account: Account<'info, BaseAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program <'info, System>,
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
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
}


// An account that goes inside a transaction instruction
#[account]
pub struct BaseAccount {
    pub members: Vec<Pubkey>,
}

#[error]
pub enum ErrorCode {
    #[msg("The derived interaction signer does not match that which was given.")]
    InvalidInteractionSigner,
}