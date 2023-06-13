enum Universe {
    Individual = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
    RC = 5
}

#[allow(dead_code)]
struct SteamId {
    id: u64
}

impl SteamId {
    pub fn from_steamid3(self, id_str: String) {

    }
}