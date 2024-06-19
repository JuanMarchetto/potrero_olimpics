use anchor_lang::prelude::*;

declare_id!("BAewG8BBJnVbxwNfDepJUCmqJMbMwm1TnabBWnbmWK6X");

#[program]
pub mod potrero_olimpcs {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, resolultion_time: i64) -> Result<()> {
        ctx.accounts.oracle_event.set_inner(
            OracleEvent {
                resolultion_time,
                resolver: *ctx.accounts.resolver.to_account_info().key,
                bump: ctx.bumps.oracle_event,
                name
            }
        );
        Ok(())
    }

    pub fn make_prediction(ctx: Context<MakePrediction>, _name: String, gold: u8, silver: u8, bronze: u8)-> Result<()> {
        let clock = Clock::get()?;
        ctx.accounts.prediction.set_inner(
            PodiumPrediction {
                gold,
                silver,
                bronze,
                event: *ctx.accounts.oracle_event.to_account_info().key,
                timestamp:clock.unix_timestamp,
                owner: ctx.accounts.player.key(),
                bump: ctx.bumps.prediction
            }
        );
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        init,
        payer = maker,
        seeds = [b"OracleEvent".as_ref(), name.as_ref()],
        bump,
        space = 40,
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    /// CHECK: we just passing it to store the pubkey
    pub resolver: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,

}


#[derive(Accounts)]
#[instruction(_name: String, seed: u64)]
pub struct MakePrediction<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        seeds = [b"OracleEvent".as_ref(), _name.as_ref()],
        bump = oracle_event.bump
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    #[account(
        init,
        payer = player,
    seeds = [b"PodiumPrediction".as_ref(),  _name.as_ref(), player.key().as_ref(), seed.to_le_bytes().as_ref()],
    bump,
    space = 40,
)]
    pub prediction: Account<'info, PodiumPrediction>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct PodiumPrediction {
    pub gold: u8,
    pub silver: u8,
    pub bronze: u8,
    pub event: Pubkey,
    pub timestamp: i64,
    pub owner: Pubkey,
    pub bump: u8,
}

#[account]
pub struct OracleEvent {
    pub resolultion_time: i64,
    pub resolver: Pubkey,
    pub bump: u8,
    pub name: String
}