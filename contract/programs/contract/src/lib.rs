use anchor_lang::prelude::*;

declare_id!("GBtJHkAivArkwUsA5HBH2Wv16gMqYQPpRCzH3uRRigKC");

#[program]
pub mod contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
