use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("7JawXA6bWsbYvdp98qMhp1Noo5TxUUCmmHjMWfccfRy4");

const PROJECT_TREASURY: &str = "GtrjYbtvJ9T5oP1P64gY2yBLXcDtKERgNp5o1k6ty7Mj";
#[program]
pub mod potrero_olimpcs {

    use std::str::FromStr;

    use anchor_lang::solana_program;

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        resolultion_time: i64,
        open_until_time: i64,
    ) -> Result<()> {
        ctx.accounts.oracle_event.set_inner(OracleEvent {
            resolultion_time,
            open_until_time,
            resolver: *ctx.accounts.resolver.to_account_info().key,
            bump: ctx.bumps.oracle_event,
            name,
        });
        Ok(())
    }

    pub fn make_prediction(
        ctx: Context<MakePrediction>,
        _name: String,
        _seed: u64,
        gold: u8,
        silver: u8,
        bronze: u8,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.project_treasury.key(),
            Pubkey::from_str(PROJECT_TREASURY).unwrap()
        );
        let clock = Clock::get()?;
        ctx.accounts.prediction.set_inner(PodiumPrediction {
            gold,
            silver,
            bronze,
            event: *ctx.accounts.oracle_event.to_account_info().key,
            timestamp: clock.unix_timestamp,
            owner: ctx.accounts.player.key(),
            bump: ctx.bumps.prediction,
        });

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.player.to_account_info().clone(),
                to: ctx.accounts.project_treasury.clone(),
            },
        );
        system_program::transfer(cpi_context, 100_000)?;
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
        space = 100,
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
        space =  16 + PodiumPrediction::INIT_SPACE,
    )]
    pub prediction: Account<'info, PodiumPrediction>,
    /// CHECK: This should match a constan Pubkey in the program
    #[account(mut)]
    pub project_treasury: AccountInfo<'info>,
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
    pub open_until_time: i64,
    pub resolultion_time: i64,
    pub resolver: Pubkey,
    pub bump: u8,
    pub name: String,
}
