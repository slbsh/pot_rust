use serde::Deserialize;

// struct to load the config into
#[derive(Deserialize, Clone)]
pub struct Conf {
    pub token: Option<String>,
    pub token_file: Option<String>,
    pub permissions: Perms,
    pub status: Stat,
    pub replies: Reply,
}

#[derive(Deserialize, Clone)]
pub struct Perms {
    pub owners: Vec<String>,
    pub mods: Vec<String>,
    pub replies: Vec<String>,
}

// config struct for statuses
#[derive(Deserialize, Clone)]
pub struct Stat {
    pub enable: bool,
    pub status_delay: u16,
    pub randomize: bool,
    pub status_list: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct Reply {
    pub enable: bool,
    pub chance: u8,
    pub iterations: u8,
    pub url_blacklist: bool,
    pub match_blacklist: Vec<String>,
    pub list: Vec<String>,
    pub trigger: Vec<String>,
}
