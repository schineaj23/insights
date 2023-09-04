use std::error::Error;

use dotenv::dotenv;
use insights::collect::Collector;
use itertools::Itertools;
use steamid_ng::SteamID;

// Creates teams in the database if they do not exist already.
// Give this a file with the teams and players, and a season id
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // let args = parse_args();

    let mut collector = Collector::new(0, None);
    let file_path = "/home/cat/src/insights/insights-services/importer/season_8_logs.csv";
    collector.import_from_file(file_path).await?;
    let logs = collector.get_logs();

    for log_id in logs {
        let log = insights::log::fetch_log(log_id).await?;

        let players_arg = log
            .players
            .keys()
            .map(|x| {
                let id = SteamID::from_steam3(&x).unwrap();
                let id64 = u64::from(id).to_string();
                id64
            })
            .join("&players[]=");

        let url = format!(
            "https://api.demos.tf/demos?map={}&type=6v6&players[]={}",
            log.info.map, players_arg
        );

        println!("Log: https://logs.tf/{}", log_id);
        println!("Demo: {}", url);

        let found = insights::demos::search_demo(&log.info.map, &players_arg).await?;

        println!("Found demos: {:?}\n", found);
    }

    Ok(())
}
