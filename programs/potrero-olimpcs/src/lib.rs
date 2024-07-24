use anchor_lang::error_code;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use std::cmp::Ordering;
use std::str::FromStr;

declare_id!("7JawXA6bWsbYvdp98qMhp1Noo5TxUUCmmHjMWfccfRy4");
const PROJECT_TREASURY: &str = "GtrjYbtvJ9T5oP1P64gY2yBLXcDtKERgNp5o1k6ty7Mj";

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
        creator_basis_points: u16,
    ) -> Result<()> {
        ctx.accounts.oracle_event.set_inner(OracleEvent {
            resolultion_time,
            open_until_time,
            resolver,
            solved_at: 0,
            bump: ctx.bumps.oracle_event,
            leaderboard: vec![],
            name,
            fee,
            creator_basis_points,
            fee_receiver,
            payed: false,
            plays: 0,
            settled: 0,
            gold: 0,
            silver: 0,
            bronze: 0,
        });
        ctx.accounts.global.events += 1;

        Ok(())
    }

    pub fn make_prediction(
        ctx: Context<MakePrediction>,
        _name: String,
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
                to: ctx.accounts.oracle_event.to_account_info().clone(),
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
        let now = clock.unix_timestamp;
        if now < ctx.accounts.oracle_event.resolultion_time {
            return Err(PotreroError::EventOpen.into());
        }
        ctx.accounts.oracle_event.gold = gold;
        ctx.accounts.oracle_event.silver = silver;
        ctx.accounts.oracle_event.bronze = bronze;
        ctx.accounts.oracle_event.solved_at = now;
        Ok(())
    }

    pub fn process_results(ctx: Context<ProcessResults>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.project_treasury.key(),
            Pubkey::from_str(PROJECT_TREASURY).unwrap()
        );
        require_keys_eq!(
            ctx.accounts.oracle_event.key(),
            ctx.accounts.prediction.event
        );
        if ctx.accounts.oracle_event.solved_at == 0 {
            return Err(PotreroError::EventNotResolved.into());
        }
        let mut points: u128 = 0;
        if ctx.accounts.oracle_event.gold == ctx.accounts.prediction.gold {
            points += 3;
        }
        if ctx.accounts.oracle_event.silver == ctx.accounts.prediction.silver {
            points += 2;
        }
        if ctx.accounts.oracle_event.bronze == ctx.accounts.prediction.bronze {
            points += 1;
        }
        ctx.accounts.player_points.score += points;
        ctx.accounts.player_points.settled += 1;
        ctx.accounts.oracle_event.settled += 1;

        if ctx.accounts.oracle_event.leaderboard.len() < 10
            || points
                >= ctx.accounts.oracle_event.leaderboard
                    [ctx.accounts.oracle_event.leaderboard.len() - 1]
                    .score
        {
            if !ctx.accounts.oracle_event.leaderboard.is_empty()
                && points
                    > ctx.accounts.oracle_event.leaderboard
                        [ctx.accounts.oracle_event.leaderboard.len() - 1]
                        .score
            {
                if ctx.accounts.oracle_event.leaderboard
                    [ctx.accounts.oracle_event.leaderboard.len() - 1]
                    .score
                    < points
                    || ctx.accounts.oracle_event.leaderboard
                        [ctx.accounts.oracle_event.leaderboard.len() - 1]
                        .timestamp
                        > ctx.accounts.prediction.timestamp
                {
                    ctx.accounts.oracle_event.leaderboard.pop();
                }
            }
            let user_obj = Player {
                pubkey: ctx.accounts.prediction.owner,
                score: points,
                timestamp: ctx.accounts.prediction.timestamp,
            };

            ctx.accounts.oracle_event.leaderboard.push(user_obj);
            ctx.accounts
                .oracle_event
                .leaderboard
                .sort_by(|a, b| match b.score.cmp(&a.score) {
                    Ordering::Equal => b.timestamp.cmp(&a.timestamp),
                    other => other,
                });


                let position = ctx
                .accounts
                .global
                .leaderboard
                .iter()
                .position(|x| x.pubkey == ctx.accounts.prediction.owner);
    
            if position.is_some() {
                let index = position.unwrap();
                ctx.accounts.global.leaderboard[index].score = ctx.accounts.player_points.score;
            } else if ctx.accounts.global.leaderboard.len() < 10
                || ctx.accounts.player_points.score
                    > ctx.accounts.global.leaderboard[ctx.accounts.global.leaderboard.len() - 1].score
            {
                if !ctx.accounts.global.leaderboard.is_empty()
                    && ctx.accounts.player_points.score
                        > ctx.accounts.global.leaderboard[ctx.accounts.global.leaderboard.len() - 1]
                            .score
                {
                    ctx.accounts.global.leaderboard.pop();
                }
                let user_obj = Player {
                    pubkey: ctx.accounts.prediction.owner,
                    score: ctx.accounts.player_points.score,
                    timestamp: ctx.accounts.player_points.timestamp,
                };
    
                ctx.accounts.global.leaderboard.push(user_obj);
                ctx.accounts
                    .global
                    .leaderboard
                    .sort_by(|a, b| b.score.cmp(&a.score));
            }
            ctx.accounts
                .global
                .leaderboard
                .sort_by(|a, b| b.score.cmp(&a.score));
        }

        Ok(())
    }

    pub fn pay(ctx: Context<Pay>)-> Result<()> {
        
        require_eq!(ctx.accounts.oracle_event.payed, false);
        require_eq!(ctx.accounts.oracle_event.settled, ctx.accounts.oracle_event.plays);
        require_keys_eq!(
            ctx.accounts.fee_receiver.key(),
            ctx.accounts.oracle_event.fee_receiver
        );
        require_keys_eq!(
            ctx.accounts.winner_0.key(),
            ctx.accounts.oracle_event.leaderboard[0].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_1.key(),
            ctx.accounts.oracle_event.leaderboard[1].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_2.key(),
            ctx.accounts.oracle_event.leaderboard[2].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_3.key(),
            ctx.accounts.oracle_event.leaderboard[3].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_4.key(),
            ctx.accounts.oracle_event.leaderboard[4].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_5.key(),
            ctx.accounts.oracle_event.leaderboard[5].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_6.key(),
            ctx.accounts.oracle_event.leaderboard[6].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_7.key(),
            ctx.accounts.oracle_event.leaderboard[7].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_8.key(),
            ctx.accounts.oracle_event.leaderboard[8].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_9.key(),
            ctx.accounts.oracle_event.leaderboard[9].pubkey
        );
        let lmps: u64 = ctx.accounts.oracle_event.to_account_info().lamports();

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.fee_receiver.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 10_000 * ctx.accounts.oracle_event.fee)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.global.to_account_info().clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100)?;

        let remainder = lmps - lmps / 10_000 * ctx.accounts.oracle_event.fee - lmps / 100;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_0.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 35)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_1.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 25)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_2.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 15)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_3.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 8)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_4.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 5)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_5.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 4)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_6.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 3)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_7.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 2)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_8.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 2)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.oracle_event.to_account_info().clone(),
                to: ctx.accounts.winner_9.clone(),
            },
        );
        system_program::transfer(cpi_context, remainder / 100 * 1)?;

        ctx.accounts.oracle_event.payed = true;
        ctx.accounts.global.settled += 1;
    Ok(())
    }
    pub fn pay_global(ctx: Context<PayGlobal>)-> Result<()> {
        require_eq!(ctx.accounts.global.settled, ctx.accounts.global.events);
        require_keys_eq!(
            ctx.accounts.project_treasury.key(),
            Pubkey::from_str(PROJECT_TREASURY).unwrap()
        );
        require_keys_eq!(
            ctx.accounts.winner_0.key(),
            ctx.accounts.global.leaderboard[0].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_1.key(),
            ctx.accounts.global.leaderboard[1].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_2.key(),
            ctx.accounts.global.leaderboard[2].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_3.key(),
            ctx.accounts.global.leaderboard[3].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_4.key(),
            ctx.accounts.global.leaderboard[4].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_5.key(),
            ctx.accounts.global.leaderboard[5].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_6.key(),
            ctx.accounts.global.leaderboard[6].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_7.key(),
            ctx.accounts.global.leaderboard[7].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_8.key(),
            ctx.accounts.global.leaderboard[8].pubkey
        );
        require_keys_eq!(
            ctx.accounts.winner_9.key(),
            ctx.accounts.global.leaderboard[9].pubkey
        );
        let lmps: u64 = ctx.accounts.global.to_account_info().lamports();

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_0.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 35)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_1.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 25)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_2.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 15)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_3.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 8)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_4.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 5)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_5.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 4)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_6.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 3)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_7.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 2)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_8.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 2)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.global.to_account_info().clone(),
                to: ctx.accounts.winner_9.clone(),
            },
        );
        system_program::transfer(cpi_context, lmps / 100 * 1)?;
    Ok(())

    } 

    pub fn close_prediction_pda(ctx: Context<ClosePredictionPda>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.project_treasury.key(),
            Pubkey::from_str(PROJECT_TREASURY).unwrap()
        );
        require!( ctx.accounts.oracle_event.payed, PotreroError::EventClose);
        require_keys_eq!(
            ctx.accounts.prediction.event,
            ctx.accounts.oracle_event.key()
        );
        Ok(())
    }

    pub fn close_event_pdas(ctx: Context<CloseEventPda>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.project_treasury.key(),
            Pubkey::from_str(PROJECT_TREASURY).unwrap()
        );
        require_eq!(
            ctx.accounts.global.events,
            ctx.accounts.global.settled
        );

        Ok(())
    }

    pub fn close_leaderboard_pdas(ctx: Context<CloseLeaderboardPda>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.project_treasury.key(),
            Pubkey::from_str(PROJECT_TREASURY).unwrap()
        );
        require_eq!(
            ctx.accounts.global.events,
            ctx.accounts.global.settled
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
        space = 1000,
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    #[account(
        init_if_needed,
        payer = maker,
        seeds = [b"olimpics".as_ref()],
        bump,
        space = Leaderboard::LEN,
    )]
    pub global: Account<'info, Leaderboard>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_name: String)]
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
pub struct ProcessResults<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"OracleEvent".as_ref(), oracle_event.name.as_ref()],
        bump = oracle_event.bump
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    #[account(mut)]
    pub prediction: Account<'info, PodiumPrediction>,
    #[account(
        mut,
        close = project_treasury,
        seeds = [b"lock".as_ref(),  oracle_event.name.as_ref(), prediction.owner.as_ref(),],
        bump = lock.bump
    )]
    pub lock: Account<'info, Lock>,
    #[account(
        mut,
        seeds = [b"olimpics".as_ref(), prediction.owner.key().as_ref(),],
        bump = player_points.bump
    )]
    pub player_points: Account<'info, PlayerPoints>,
    /// CHECK: This should match a constant Pubkey in the program
    #[account(mut)]
    pub project_treasury: AccountInfo<'info>,
    #[account(mut)]
    pub global: Account<'info, Leaderboard>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_name: String)]
pub struct Pay<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        seeds = [b"OracleEvent".as_ref(), _name.as_ref()],
        bump = oracle_event.bump
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_0: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_1: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_2: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_3: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_4: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_5: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_6: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_7: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_8: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_9: AccountInfo<'info>,
    /// CHECK: This should match the fee_receiver in the OracleEvent
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,
    #[account(mut)]
    pub global: Account<'info, Leaderboard>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_name: String)]
pub struct PayGlobal<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_0: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_1: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_2: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_3: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_4: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_5: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_6: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_7: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_8: AccountInfo<'info>,
    /// CHECK: This should match the leaderboard in the OracleEvent
    #[account(mut)]
    pub winner_9: AccountInfo<'info>,
    /// CHECK: This should match a constant Pubkey in the program
    #[account(mut)]
    pub project_treasury: AccountInfo<'info>,
    #[account(mut)]
    pub global: Account<'info, Leaderboard>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePredictionPda<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    #[account(
        mut,
        close = project_treasury
    )]
    pub prediction: Account<'info, PodiumPrediction>,
    /// CHECK: This should match a constant Pubkey in the program
    #[account(mut)]
    pub project_treasury: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CloseEventPda<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        close = project_treasury
    )]
    pub oracle_event: Account<'info, OracleEvent>,
    #[account(mut)]
    pub global: Account<'info, Leaderboard>,
    #[account(mut)]
    pub project_treasury: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CloseLeaderboardPda<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, close = project_treasury)]
    pub global: Account<'info, Leaderboard>,
    #[account(mut)]
    pub project_treasury: AccountInfo<'info>,
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
    pub creator_basis_points: u16,
    pub fee_receiver: Pubkey,
    pub plays: u128,
    pub settled: u128,
    pub payed: bool,
    pub bump: u8,
    pub gold: u8,
    pub silver: u8,
    pub bronze: u8,
    pub leaderboard: Vec<Player>,
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

#[account]
pub struct Leaderboard {
    pub leaderboard: Vec<Player>,
    pub settled: u128,
    pub events: u128,
}

impl Leaderboard {
    const LEN: usize = 8 + 16 + 16 + ((8 + 16 + 32) * 10);
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
    #[msg("The Event is not resolved.")]
    EventNotResolved,
    #[msg("The Event is not payed.")]
    EventNotPayed,
}
