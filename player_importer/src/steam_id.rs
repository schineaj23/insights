use regex::Regex;

// TODO: use this code
#[allow(dead_code)]
pub struct SteamId {
    id: i64,
    instance: i64,
    account_type: i64,
    universe: i64,
}

#[allow(dead_code)]
impl SteamId {
    pub fn new(universe: i64, id: i64, instance: i64, account_type: i64) -> SteamId {
        SteamId {
            id,
            instance,
            account_type,
            universe,
        }
    }
}

pub fn from_steamid3(id_str: &str) -> Option<i64> {
    let id3 = Regex::new(r"(\w):([0-9]{1}):([0-9]+)").unwrap();
    let captures = id3.captures(id_str).unwrap();

    let account_type: i64 = match captures.get(1).unwrap().as_str() {
        "U" => 1,
        "M" => 2,
        _ => {
            println!("Invalid Account Type");
            return None;
        }
    };

    let universe: i64 = captures.get(2).unwrap().as_str().parse::<i64>().unwrap();
    let id: i64 = captures.get(3).unwrap().as_str().parse::<i64>().unwrap();
    let instance = 1;

    Some((universe << 56) | (account_type << 52) | (instance << 32) | id)
}
