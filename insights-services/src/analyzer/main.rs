use insights::analyzer::analyzer::{BombAttempt, BombAttemptAnalyzer};
use log::debug;
use pico_args::Arguments;
use std::{collections::HashMap, fs, time::Instant};
use tf_demo_parser::{Demo, DemoParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();
    let demo: String = args.value_from_str(["-d", "--demo"]).unwrap_or_else(|_| {
        eprintln!(
            "USAGE: analyzer [-d --demo file.dem] [-a]
            -a          Retrieve and analyze demos from database
            -d, --demo  Analyze specific demo"
        );
        std::process::exit(1);
    });

    let file = fs::read(demo)?;
    let demo = Demo::new(&file);

    let start = Instant::now();

    let parser = DemoParser::new_with_analyser(demo.get_stream(), BombAttemptAnalyzer::new());
    let (_header, (bomb_attempts, users)) = parser.parse()?;

    let t = start.elapsed().as_secs_f32();

    debug!("All attempts: {:?}", bomb_attempts);

    let bomb_dmg: Vec<&BombAttempt> = bomb_attempts
        .iter()
        .filter(|attempt| attempt.damage > 0)
        .collect();

    let mut dmg: HashMap<u16, (i32, i32, u32)> = HashMap::new();

    let mut a = 0;
    bomb_attempts.iter().for_each(|x| {
        a += x.damage;
        dmg.entry(x.user)
            .and_modify(|u| {
                u.0 += 1;
                u.1 += i32::from(x.damage > 0);
                u.2 += x.damage as u32;
            })
            .or_insert((1, 0, x.damage as u32));
    });

    for (uid, (cnt, cnt_dmg, dmg)) in dmg {
        println!(
            "Uid: {:?}, NumBombs: {}, Dmg/Bomb: {:.2}, NonzeroDmgBombs: {} BombEff: {:.2}%",
            insights::steam_id::from_steamid3(&users.get(&uid.into()).unwrap().steam_id),
            cnt,
            dmg as f64 / cnt as f64,
            cnt_dmg,
            100f32 * cnt_dmg as f32 / cnt as f32
        );
    }

    println!(
        "Average dmg/bombattempt: {}",
        (a as f32 / bomb_attempts.len() as f32)
    );

    println!("Num dmg>0: {}", bomb_dmg.len());
    println!("Num Total: {}", bomb_attempts.len());
    println!("Took {} seconds to analyze demo.", t);

    Ok(())
}
