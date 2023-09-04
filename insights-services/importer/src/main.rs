use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use dotenv::dotenv;
use importer::Importer;
use importer::PlayerCache;
use insights::collect;
use insights::collect::Collector;
use insights::log::LogSerialized;
use pico_args::Arguments;
use std::error::Error;
use std::{collections::HashMap, fs::File, io::BufReader};
use tokio::time::Instant;
use tracing::info;
use tracing::warn;

struct LocalCache {
    given_players: Option<HashMap<String, i32>>,
    all_players: HashMap<String, Option<i32>>,
}

impl LocalCache {
    pub fn new() -> Self {
        Self {
            given_players: None,
            all_players: HashMap::new(),
        }
    }

    pub fn load_player_lut(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let map: HashMap<String, i32> = serde_json::from_reader(reader)?;
        self.given_players = Some(map);
        Ok(())
    }

    fn get_team_id_from_list(&self, player_id: &str) -> Option<i32> {
        match &self.given_players {
            Some(player_map) => player_map.get(player_id).cloned(),
            None => None,
        }
    }
}

#[async_trait]
impl PlayerCache for LocalCache {
    async fn fetch_team_id_for_player(&mut self, id: &str) -> Option<i32> {
        if self.all_players.contains_key(id) {
            return *self.all_players.get(id).unwrap();
        }

        let id_opt = self.get_team_id_from_list(id).and_then(|team_id| {
            info!("Found team_id from LUT");
            Some(team_id)
        });

        let id_opt = match id_opt {
            Some(x) => Some(x),
            None => {
                info!("Couldn't find team_id from LUT, querying RGL for {}", id);
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

struct Args {
    fetch_new_logs: bool,
    dump_log_cache: bool,
    insert_into_db: bool,
    team_list_path: String,
    read_logs_path: Option<String>,
    player_lut_path: Option<String>,
    start: i32,
    end: Option<i32>,
}

fn get_config_dir() -> String {
    let path: String = match home::home_dir() {
        Some(dir) => dir.to_str().unwrap().to_owned(),
        None => "/".to_owned(),
    };
    path + "/.insights"
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let help: &str =
        "USAGE: importer -t | --team-list FILE [-l FILE] [-fdi] [-s NUMBER] [-e NUMBER] [-r FILE]
        -t --team-list  Read teams and players from JSON file
        [-l --list]     Key-value JSON file for players' team IDs.
        [-f --fetch]    Fetch new logs from logs.tf
        [-d --dump]     Dump fetched logs to file
        [-i --insert]   Insert collected logs into database
        [-s --start]    Minimum date of log in Unix timestamp, if none supplied use last run
        [-e --end]      Maximum date of log in Unix timestamp, if none supplied use current time
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

    let start_args: Result<String, pico_args::Error> = env_args.value_from_str(["-s", "--start"]);

    let start: Option<i32> = match start_args {
        Ok(start_str) => start_str.parse::<i32>().ok(),
        Err(e) => match e {
            pico_args::Error::MissingOption(_) => {
                match std::fs::read_to_string(get_config_dir() + "/last_run") {
                    Ok(o) => o.parse::<i32>().ok(),
                    Err(err) => {
                        warn!("Error reading from last_run: {}", err);
                        None
                    }
                }
            }
            err => {
                warn!("Found other error {:?}", err);
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
        player_lut_path: env_args.opt_value_from_str(["-l", "--list"])?,
        start: start.unwrap_or_else(|| {
            println!("Error reading offset, using S12 registration deadline as fallback");
            // TODO: automate team/seasons
            // Mon May 15 2023 03:59:00 GMT+0000 (S12 Team Registration Deadline) = 1684123140
            1684123140
        }),
        end: env_args.opt_value_from_str(["-e", "--end"])?,
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
    std::fs::create_dir_all(get_config_dir())?;

    let team_map: HashMap<String, collect::Team> = serde_json::from_reader(reader)?;
    println!("{:#?}", team_map.keys());

    // TODO: make the collecting and the adding on separate services so they don't block each other
    let log_collection_start = Instant::now();

    let start_date_time = NaiveDateTime::from_timestamp_opt(args.start as i64, 0).unwrap();
    let mut end_date_str = String::from("present");
    if let Some(end) = args.end {
        let end_date_time = NaiveDateTime::from_timestamp_opt(end as i64, 0).unwrap();
        end_date_str = end_date_time.format("%d/%m/%Y %H:%M").to_string();
    }
    println!(
        "Starting log collection from timestamp {} to {}",
        start_date_time.format("%d/%m/%Y %H:%M"),
        end_date_str
    );

    let mut collector = Collector::new(args.start, args.end);

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
        collector.dump_cache_to_file("dumped_logs.csv").await?;
    }

    let log_collection_time = log_collection_start.elapsed();

    // Save current timestamp as next offset
    let utc_now: DateTime<Utc> = Utc::now();
    tokio::fs::write(
        get_config_dir() + "/last_run",
        utc_now.timestamp().to_string(),
    )
    .await?;

    if !args.insert_into_db {
        return Ok(());
    }

    let log_insert_start = Instant::now();

    let url = std::env::var("LOG_DATABASE_URL").expect("LOG_DATABASE_URL not set");
    let pool = sqlx::PgPool::connect(&url).await?;

    let mut cache = LocalCache::new();

    if let Some(lut_path) = args.player_lut_path {
        cache.load_player_lut(&lut_path)?;
        println!("Is past season? {}", args.end.is_some());
    }

    let mut importer = Importer::new(&pool, cache, args.end.is_some());

    for log_id in logs_cache.iter() {
        let start_time = Instant::now();

        let log: LogSerialized = match insights::log::fetch_log(&log_id).await {
            Ok(log) => log,
            Err(x) => {
                eprintln!("Log: {}, Error: {:?}", log_id, x);
                continue;
            }
        };

        info!(
            "Request recieved from logs.tf in {} seconds",
            start_time.elapsed().as_secs_f32()
        );

        // Why do we bother processing if it is past our offset anyways
        // Covering the case where a log is in the file but past the desired start time
        if log.info.date < collector.start {
            println!("Log {} too old. Skipping...", log_id);
            continue;
        }

        importer.import_log(*log_id, &log).await?;
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
