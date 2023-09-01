use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

use dotenv::dotenv;
use insights::collect;
use pico_args::Arguments;
use sqlx::Postgres;

struct Args {
    team_list: String,
    season: i32,
}

fn parse_args() -> Args {
    let help: &str = "USAGE: teamporter -t FILE -s SEASON
        -t --team-list  Team list in JSON file
        -s --season     Season number of players
    ";

    let mut args: Vec<_> = std::env::args_os().collect();
    args.remove(0);

    if args.len() == 0 {
        println!("{help}");
        std::process::exit(-1);
    }

    let mut env_args = Arguments::from_vec(args);

    if env_args.contains(["-h", "--help"]) {
        println!("{help}");
        std::process::exit(-1);
    }

    Args {
        team_list: env_args.value_from_str(["-t", "--team-list"]).unwrap(),
        season: env_args.value_from_str(["-s", "--season"]).unwrap(),
    }
}

// Creates teams in the database if they do not exist already.
// Give this a file with the teams and players, and a season id
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let args = parse_args();

    let url = dotenv::var("LOG_DATABASE_URL").expect("LOG_DATABASE_URL not set");
    let pool = sqlx::PgPool::connect(&url).await?;

    let file = File::open(args.team_list)?;
    let reader = BufReader::new(file);
    let team_map: HashMap<String, collect::Team> = serde_json::from_reader(reader)?;

    // First create all the teams
    for (team_name, team) in team_map {
        let res = sqlx::query::<Postgres>("insert into team (team_id, team_name, season) values ($1, $2, $3) on conflict do nothing")
        .bind(&team.id)
        .bind(&team_name)
        .bind(&args.season)
        .execute(&pool).await?;

        let mut chr = "=";
        if res.rows_affected() > 0 {
            chr = "+";
        }

        println!(
            "{}({}, {}, Season: {})",
            chr, team.id, team_name, args.season
        );

        // Then add the team id to each player's teams array
        for (name, id) in team.players {
            let steam_id = id.parse::<i64>()?;

            let res = sqlx::query::<Postgres>(
                "insert into player (steamid64, name, teams) values ($1, $2, ARRAY[$3]) on conflict (steamid64) do update set teams = array_append(player.teams, $3) where player.steamid64 = $1 and not (ARRAY[$3] <@ player.teams)",
            )
            .bind(&steam_id)
            .bind(&name)
            .bind(&team.id)
            .execute(&pool)
            .await?;

            let mut chr = "=";
            if res.rows_affected() > 0 {
                chr = "+";
            }

            println!("{}({}, {}, Player: {})", chr, team.id, team_name, name);
        }
    }

    Ok(())
}
