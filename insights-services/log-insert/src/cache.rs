use async_trait::async_trait;
use importer::PlayerCache;
use std::collections::HashMap;

pub struct LocalCache {
    given_players: HashMap<String, i32>,
    all_players: HashMap<String, Option<i32>>,
}

impl LocalCache {
    pub fn new() -> Self {
        let player_lut_str = std::env::var("PLAYER_TEAMID_LUT").expect("PLAYER_TEAMID_LUT not set");
        let players: HashMap<String, i32> = serde_json::from_str(&player_lut_str).unwrap();
        Self {
            given_players: players,
            all_players: HashMap::new(),
        }
    }

    fn get_team_id_from_list(&self, player_id: &str) -> Option<i32> {
        self.given_players.get(player_id).copied()
    }
}

#[async_trait]
impl PlayerCache for LocalCache {
    async fn fetch_team_id_for_player(&mut self, id: &str) -> Option<i32> {
        if self.all_players.contains_key(id) {
            return *self.all_players.get(id).unwrap();
        }

        let id_opt = self.get_team_id_from_list(id).and_then(|team_id| {
            println!("Found team_id from LUT");
            Some(team_id)
        });

        let id_opt = match id_opt {
            Some(x) => Some(x),
            None => {
                println!("Couldn't find team_id from LUT, querying RGL for {}", id);
                self.fallback(id).await
            }
        };
        self.all_players.insert(id.to_string(), id_opt);

        id_opt
    }

    async fn fetch_team_id_for_past_player(&self, id: &str) -> Option<i32> {
        self.get_team_id_from_list(id)
    }
}
