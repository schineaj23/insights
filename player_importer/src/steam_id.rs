use regex::Regex;

// TODO: use this code
#[allow(dead_code)]
pub struct SteamId {
    id: u64,
    instance: u64,
    account_type: u64,
    universe: u64,
}

#[allow(dead_code)]
impl SteamId {
    pub fn new(universe: u64, id: u64, instance: u64, account_type: u64) -> SteamId {
        SteamId {
            id,
            instance,
            account_type,
            universe,
        }
    }
}

pub fn from_steamid3(id_str: String) -> Option<u64> {
    let id3 = Regex::new(r"(\w):([0-9]{1}):([0-9]+)").unwrap();
    let captures = id3.captures(id_str.as_str()).unwrap();

    let account_type: u64 = match captures.get(1).unwrap().as_str() {
        "U" => 1,
        "M" => 2,
        _ => {
            println!("Invalid Account Type");
            return None;
        }
    };

    let universe: u64 = captures.get(2).unwrap().as_str().parse::<u64>().unwrap();
    let id: u64 = captures.get(3).unwrap().as_str().parse::<u64>().unwrap();
    let instance = 1;

    Some((universe << 56) | (account_type << 52) | (instance << 32) | id)
}
