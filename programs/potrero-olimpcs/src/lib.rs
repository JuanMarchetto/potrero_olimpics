use anchor_lang::error_code;
use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("7JawXA6bWsbYvdp98qMhp1Noo5TxUUCmmHjMWfccfRy4");

#[program]
pub mod potrero_olimpcs {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        resolultion_time: i64,
        open_until_time: i64,
        resolver: Pubkey,
        fee: u64,
        fee_receiver: Pubkey,
    ) -> Result<()> {
        ctx.accounts.oracle_event.set_inner(OracleEvent {
            resolultion_time,
            open_until_time,
            resolver,
            solved_at: 0,
            bump: ctx.bumps.oracle_event,
            leadeboard: vec![],
            name,
            fee,
            fee_receiver,
            plays: 0,
            settled: 0,
            gold: 0,
            silver: 0,
            bronze: 0,
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
            ctx.accounts.fee_receiver.key(),
            ctx.accounts.oracle_event.fee_receiver
        );
        let clock = Clock::get()?;
        if clock.unix_timestamp > ctx.accounts.oracle_event.open_until_time
            || ctx.accounts.oracle_event.solved_at != 0
        {
            return Err(PotreroError::EventClose.into());
        }
        ctx.accounts.prediction.set_inner(PodiumPrediction {
            gold,
            silver,
            bronze,
            event: *ctx.accounts.oracle_event.to_account_info().key,
            timestamp: clock.unix_timestamp,
            owner: ctx.accounts.player.key(),
            bump: ctx.bumps.prediction,
        });
        ctx.accounts.oracle_event.plays += 1;
        ctx.accounts.lock.set_inner(Lock {
            bump: ctx.bumps.lock,
        });
        ctx.accounts.player_points.set_inner(PlayerPoints {
            pubkey: ctx.accounts.player.key(),
            score: ctx.accounts.player_points.score,
            timestamp: clock.unix_timestamp,
            events: ctx.accounts.player_points.events + 1,
            settled: ctx.accounts.player_points.settled,
            bump: ctx.bumps.player_points,
        });
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.player.to_account_info().clone(),
                to: ctx.accounts.fee_receiver.clone(),
            },
        );
        system_program::transfer(cpi_context, ctx.accounts.oracle_event.fee)?;
        Ok(())
    }

    pub fn resolve(ctx: Context<OracleResolve>, gold: u8, silver: u8, bronze: u8) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.resolver.key(),
            ctx.accounts.oracle_event.resolver
        );
        let clock = Clock::get()?;
        if clock.unix_timestamp < ctx.accounts.oracle_event.resolultion_time {
            return Err(PotreroError::EventOpen.into());
        }
        ctx.accounts.oracle_event.gold = gold;
        ctx.accounts.oracle_event.silver = silver;
        ctx.accounts.oracle_event.bronze = bronze;
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
        space = 1000,
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_name: String, seed: u64)]
pub struct MakePrediction<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        seeds = [b"OracleEvent".as_ref(), _name.as_ref()],
        bump = oracle_event.bump
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    #[account(
        init,
        payer = player,
        seeds = [b"PodiumPrediction".as_ref(), _name.as_ref(), oracle_event.plays.to_le_bytes().as_ref()],
        bump,
        space =  8 + PodiumPrediction::INIT_SPACE,
    )]
    pub prediction: Account<'info, PodiumPrediction>,
    #[account(
        init,
        payer = player,
        seeds = [b"lock".as_ref(),  _name.as_ref(), player.key().as_ref(),],
        bump,
        space =  8 + 1,
    )]
    pub lock: Account<'info, Lock>,
    #[account(
        init_if_needed,
        payer = player,
        seeds = [b"olimpics".as_ref(), player.key().as_ref(),],
        bump,
        space =  8 + PlayerPoints::INIT_SPACE,
    )]
    pub player_points: Account<'info, PlayerPoints>,
    /// CHECK: This should match the fee_receiver in the OracleEvent
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct OracleResolve<'info> {
    #[account(mut)]
    pub resolver: Signer<'info>,
    #[account(mut)]
    pub oracle_event: Account<'info, OracleEvent>,
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
    pub solved_at: i64,
    pub resolver: Pubkey,
    pub fee: u64,
    pub fee_receiver: Pubkey,
    pub plays: u128,
    pub settled: u128,
    pub bump: u8,
    pub gold: u8,
    pub silver: u8,
    pub bronze: u8,
    pub leadeboard: Vec<Player>,
    pub name: String,
}

#[account]
pub struct Lock {
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct PlayerPoints {
    pub pubkey: Pubkey,
    pub score: u128,
    pub timestamp: i64,
    pub events: u16,
    pub settled: u16,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Player {
    pub pubkey: Pubkey,
    pub score: u128,
    pub timestamp: i64,
}

#[error_code]
pub enum PotreroError {
    #[msg("The Event is already closed.")]
    EventClose,
    #[msg("The Event is still open.")]
    EventOpen,
}
