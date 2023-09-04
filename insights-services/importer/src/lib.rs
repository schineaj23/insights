use async_trait::async_trait;

use insights::{
    db,
    log::{LogSerialized, PlayerStats},
    rgl,
};
use sqlx::PgPool;
use std::{collections::HashMap, error::Error, time::Instant};
use steamid_ng::SteamID;
use tracing::{info, warn};

#[async_trait]
pub trait PlayerCache {
    async fn fetch_team_id_for_player(&mut self, id: &str) -> Option<i32>;
    async fn fetch_team_id_for_past_player(&self, id: &str) -> Option<i32>;

    // If all else fails, just query the API our cache completely missed.
    async fn fallback(&self, id: &str) -> Option<i32> {
        match rgl::search_player(id).await {
            Ok(resp) => match resp.current_teams.get("sixes") {
                Some(team) => match team {
                    Some(t) => Some(t.id),
                    None => None,
                },
                None => None,
            },
            Err(_) => None,
        }
    }
}

pub struct Importer<'a, C>
where
    C: PlayerCache,
{
    pool: &'a PgPool,
    cache: C,
    past_season: bool,
}

// TODO: make the db a trait so i can add tests for adding, things are getting weird now.
impl<'a, C: PlayerCache> Importer<'a, C> {
    pub fn new(pool: &'a PgPool, cache: C, is_past: bool) -> Self {
        Self {
            pool: pool,
            cache: cache,
            past_season: is_past,
        }
    }

    async fn get_player_team_id(&mut self, id: &str) -> Option<i32> {
        match self.past_season {
            true => match self.cache.fetch_team_id_for_past_player(&id).await {
                Some(id) => Some(id),
                None => {
                    warn!("Couldn't find team id for player {}", id);
                    None
                }
            },
            false => match self.cache.fetch_team_id_for_player(&id).await {
                Some(id) => Some(id),
                None => {
                    warn!("Couldn't find team id for player {} (PAST)", id);
                    None
                }
            },
        }
    }

    pub async fn import_log(
        &mut self,
        log_id: i32,
        log: &LogSerialized,
    ) -> Result<(), Box<dyn Error>> {
        info!("Log {}: Start", log_id);
        let (red_team_id, blu_team_id) = self.determine_team_ids_for_match(&log.players).await?;

        db::insert_log(self.pool, &log_id, &(red_team_id, blu_team_id), &log).await?;
        info!("Log {}: Added to DB", log_id);

        for (player_id, stats) in log.players.iter() {
            let start_time = Instant::now();

            let player_id = u64::from(SteamID::from_steam3(player_id)?);

            // Skip ringers!
            // TODO: Add ringers with with id 99999 or something.
            let team_id = match self.get_player_team_id(&player_id.to_string()).await {
                Some(id) => id,
                None => {
                    info!("Log {}: Skipping ringer {}", log_id, player_id);
                    continue;
                }
            };

            if db::insert_player(self.pool, &(player_id as i64), &team_id).await? == 0 {
                info!(
                    "Log {}: Player {} was skipped for insert",
                    log_id, player_id
                );
            }

            db::insert_player_stats(self.pool, &log_id, &(player_id as i64), stats).await?;

            info!(
                "Log {}: Took {} seconds to insert player stats",
                log_id,
                start_time.elapsed().as_secs_f32()
            );
        }
        Ok(())
    }

    async fn determine_team_ids_for_match(
        &mut self,
        player_map: &HashMap<String, PlayerStats>,
    ) -> Result<(i32, i32), Box<dyn std::error::Error>> {
        // Separate teams
        let mut last_red_id = 0;
        let mut last_blu_id = 0;
        let mut visited = 0;
        for (id, player) in player_map.iter() {
            let id64 = u64::from(SteamID::from_steam3(&id).unwrap()).to_string();

            let team_id = match self.get_player_team_id(&id64).await {
                Some(id) => id,
                None => continue,
            };

            if last_red_id == team_id || last_blu_id == team_id {
                visited += 1;
            }

            match player.team.as_str() {
                "Red" => {
                    last_red_id = team_id;
                }
                "Blue" => {
                    last_blu_id = team_id;
                }
                _ => {}
            };

            if visited >= 4 {
                break;
            }
        }

        info!("Found Red: {} Blue: {}", last_red_id, last_blu_id);

        Ok((last_red_id, last_blu_id))
    }
}
