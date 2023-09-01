use std::{collections::HashMap, error::Error};

use analyzer::{AnalyzerResult, BombAttempt, BombAttemptAnalyzer, BombState};
use serde::Serialize;
use sqlx::PgPool;
use steamid_ng::SteamID;
use tf_demo_parser::{demo::Buffer, DemoParser, Stream};

pub mod analyzer;

#[derive(Serialize)]
pub struct PlayerSummary {
    pub name: String,
    pub steamid: u64,
    pub attempts: i32,
    pub damage_per_attempt: f32,
}

pub fn analyze(bytes: Vec<u8>) -> Result<AnalyzerResult, Box<dyn Error>> {
    let stream = Stream::new(Buffer::from(bytes));
    let (_, (attempts, users)) =
        DemoParser::new_with_analyser(stream, BombAttemptAnalyzer::new()).parse()?;
    Ok((attempts, users))
}

pub fn package_summary(results: AnalyzerResult) -> Vec<PlayerSummary> {
    let mut bomb_map: HashMap<u16, (i32, i32)> = HashMap::new();

    for attempt in results.0 {
        bomb_map
            .entry(attempt.user)
            .and_modify(|u| {
                u.0 += 1;
                u.1 += attempt.damage as i32;
            })
            .or_insert((1, attempt.damage as i32));
    }

    let mut players: Vec<PlayerSummary> = Vec::new();

    for (uid, (cnt, dmg)) in bomb_map {
        let user = results.1.get(&uid.into()).unwrap();
        let id = u64::from(SteamID::from_steam3(&user.steam_id).unwrap());
        players.push(PlayerSummary {
            name: user.name.clone(),
            steamid: id,
            attempts: cnt,
            damage_per_attempt: dmg as f32 / cnt as f32,
        })
    }

    players
}

// FIXME: move this to insights without making a circular dependency
pub async fn insert_bomb_attempt(
    pool: &PgPool,
    attempt: &BombAttempt,
    player_id: i64,
    log_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let died = attempt.state == BombState::Died;
    let start_tick = u32::from(attempt.start_tick) as i32;
    let end_tick = u32::from(attempt.land_tick.unwrap_or_default()) as i32;

    sqlx::query::<sqlx::Postgres>("insert into bomb_attempt (player_id, log_id, damage, damage_taken, start_tick, end_tick, died) values ($1, $2, $3, $4, $5, $6, $7)")
    .bind(&player_id)
    .bind(&log_id)
    .bind(attempt.damage as i32)
    .bind(attempt.damage_taken as i32)
    .bind(&start_tick)
    .bind(&end_tick)
    .bind(&died)
    .execute(pool).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
