use cached::proc_macro::cached;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use dotenv::dotenv;
use insights::collect;
use insights::collect::Collector;
use insights::db;
use insights::log::LogSerialized;
use insights::log::PlayerStats;
use insights::rgl;
use insights::steam_id;
use pico_args::Arguments;
use reqwest::Response;
use std::{collections::HashMap, fs::File, io::BufReader};
use tokio::time::Instant;

#[cached(size = 200)]
async fn fetch_team_id_for_player(player_id: String) -> Option<i32> {
    match get_team_id_from_player_lut(&player_id) {
        Some(team_id) => {
            println!("Found team_id from LUT");
            Some(team_id)
        }
        None => {
            println!(
                "Could not find team_id from LUT, asking RGL API for player {}",
                player_id
            );
            let res: Response =
                reqwest::get(format!("https://api.rgl.gg/v0/profile/{}", player_id))
                    .await
                    .ok()?;
            let player = res.json::<rgl::Player>().await.ok()?;
            let team = player.current_teams.get("sixes")?;

            match team {
                Some(team) => Some(team.id),
                _ => None,
            }
        }
    }
}

fn get_team_id_from_player_lut(player_id: &str) -> Option<i32> {
    // TODO: cache this entire file in memory instead of reopening
    let file = File::open("/home/cat/src/insighttf/player_teamid_lut.json").ok()?;
    let reader: BufReader<File> = BufReader::new(file);
    let id_map: HashMap<String, i32> = serde_json::from_reader(reader).ok()?;

    id_map.get(player_id).map(|x| *x)
}

async fn determine_team_ids_for_match(
    player_map: &HashMap<String, PlayerStats>,
) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    println!("determine_team_ids_for_match called");
    // Separate teams

    let mut last_red_id = 0;
    let mut last_blu_id = 0;
    let mut visited = 0;
    for (id, player) in player_map.iter() {
        let id64 = steam_id::from_steamid3(id).unwrap().to_string();
        let team_id = match fetch_team_id_for_player(id64).await {
            Some(id) => id,
            None => {
                println!(
                    "Could not find team id for player {} (are they not rostered?)",
                    id
                );
                continue;
            }
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

    println!("Found Red: {} Blue: {}", last_red_id, last_blu_id);

    Ok((last_red_id, last_blu_id))
}

struct Args {
    fetch_new_logs: bool,
    dump_log_cache: bool,
    insert_into_db: bool,
    team_list_path: String,
    read_logs_path: Option<String>,
    offset: i32,
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let help: &str =
        "USAGE: importer -t | --team-list FILE [-fdi] [-o | --offset NUMBER] [-r | --read FILE]
        -t --team-list  Read teams and players from JSON file
        [-f --fetch]    Fetch new logs from logs.tf
        [-d --dump]     Dump fetched logs to file
        [-i --insert]   Insert collected logs into database
        [-o --offset]   Minimum date of log in Unix timestamp, if none supplied use last run
        [-r --read]     Read log ids from file separated by newlines
    ";
    let mut args: Vec<_> = std::env::args_os().collect();
    args.remove(0);

    if args.len() == 0 {
        println!("{}", help);
        std::process::exit(-1);
    }

    let mut env_args = Arguments::from_vec(args);

    if env_args.contains(["-h", "--help"]) {
        println!("{}", help);
        std::process::exit(-1);
    }

    let offset_args: Result<String, pico_args::Error> = env_args.value_from_str(["-o", "--offset"]);

    let offset: Option<i32> = match offset_args {
        Ok(offset_str) => offset_str.parse::<i32>().ok(),
        Err(e) => match e {
            pico_args::Error::MissingOption(_) => {
                match std::fs::read_to_string("~/.config/insights/importer/last_run") {
                    Ok(o) => o.parse::<i32>().ok(),
                    Err(err) => {
                        println!("Error reading from last_run: {}", err);
                        None
                    }
                }
            }
            err => {
                println!("Found other error {:?}", err);
                None
            }
        },
    };

    let args = Args {
        fetch_new_logs: env_args.contains(["-f", "--fetch"]),
        dump_log_cache: env_args.contains(["-d", "--dump"]),
        insert_into_db: env_args.contains(["-i", "--insert"]),
        team_list_path: env_args
            .value_from_str(["-t", "--team-list"])
            .unwrap_or_else(|_| std::env::var("TEAM_LIST_PATH").expect("TEAM_LIST_PATH not set")),
        offset: offset.unwrap_or_else(|| {
            println!("Error reading offset, using S12 registration deadline as fallback");
            // TODO: automate team/seasons
            // Mon May 15 2023 03:59:00 GMT+0000 (S12 Team Registration Deadline) = 1684123140
            1684123140
        }),
        read_logs_path: env_args.opt_value_from_str(["-r", "--read"])?,
    };

    Ok(args)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args = parse_args()?;
    let file = File::open(args.team_list_path)?;
    let reader = BufReader::new(file);

    // Create cache directory if it doesn't exist already
    std::fs::create_dir_all("~/.config/insights/importer/")?;

    let team_map: HashMap<String, collect::Team> = serde_json::from_reader(reader)?;
    println!("{:#?}", team_map.keys());

    // Inserting teams
    // for (team_name, team_data) in team_map.iter() {
    //     db::insert_team(&pool, &team_name, &team_data.id).await?;
    // }

    // TODO: make the collecting and the adding on separate services so they don't block each other
    let log_collection_start = Instant::now();

    let date_time = NaiveDateTime::from_timestamp_opt(args.offset as i64, 0).unwrap();
    println!(
        "Starting log collection from timestamp {}",
        date_time.format("%d/%m/%Y %H:%M")
    );

    let mut collector = Collector::new(args.offset);

    if let Some(cache_path) = args.read_logs_path {
        println!(
            "Collected {} logs from file",
            collector.import_from_file(&cache_path).await?
        );
    }

    if args.fetch_new_logs {
        collector.collect_all_team_logs(team_map).await?;
    }

    let logs_cache = collector.get_logs();

    if args.dump_log_cache {
        collector.dump_cache_to_file("dumped-logs.txt").await?;
    }

    let log_collection_time = log_collection_start.elapsed();

    // Save current timestamp as next offset
    let utc_now: DateTime<Utc> = Utc::now();
    tokio::fs::write(
        "~/.config/insights/importer/last_run",
        utc_now.timestamp().to_string(),
    )
    .await?;

    let log_insert_start = Instant::now();

    let pool = db::connect().await?;

    if !args.insert_into_db {
        return Ok(());
    }

    for log_id in logs_cache.iter() {
        println!("Making request to logs.tf");

        let start_time = Instant::now();

        let log: LogSerialized = reqwest::get(format!("http://logs.tf/json/{}", log_id))
            .await?
            .json::<LogSerialized>()
            .await?;

        // Why do we bother processing if it is past our offset anyways
        if log.info.date < collector.offset {
            println!("Log {} too old. Skipping...", log_id);
            continue;
        }

        println!(
            "Request recieved from logs.tf in {} seconds",
            start_time.elapsed().as_secs_f32()
        );

        let (red_team_id, blu_team_id) = determine_team_ids_for_match(&log.players).await?;

        db::insert_log(&pool, &log_id, &(red_team_id, blu_team_id), &log).await?;
        println!("Added log {} to db", log_id);

        for (player_id, stats) in log.players.iter() {
            let start_time = Instant::now();

            let player_id = steam_id::from_steamid3(player_id).unwrap();

            // If it is a ringer, just ignore we don't care. It would probably throw an error in DB anyways
            let team_id = match fetch_team_id_for_player(player_id.to_string()).await {
                Some(id) => id,
                None => {
                    println!("Skipping ringer {}", player_id);
                    continue;
                }
            };

            db::insert_player(&pool, &player_id, &team_id).await?;
            db::insert_player_stats(&pool, log_id, &player_id, stats).await?;

            println!(
                "Took {} seconds to insert player stats",
                start_time.elapsed().as_secs_f32()
            );
        }
    }

    println!("Likely scrim log count: {}", logs_cache.len());
    println!(
        "Log collection time: {} seconds",
        log_collection_time.as_secs_f32()
    );
    println!(
        "Log insert time: {} seconds",
        log_insert_start.elapsed().as_secs_f32()
    );
    println!(
        "Total time elapsed: {} seconds",
        log_collection_start.elapsed().as_secs_f32()
    );

    Ok(())
}
