// struct to load the config into
#[derive(serde::Deserialize, Clone)]
pub struct Conf {
    pub token: Option<String>,
    pub token_file: Option<String>,
    pub permissions: Perms,
    pub status: Stat,
    pub warns: Warns,
    pub replies: Reply,
}

#[derive(serde::Deserialize, Clone)]
pub struct Perms {
    pub owners: Vec<String>,
    pub mods: Vec<String>,
    pub replies: Vec<String>,
}

// config struct for statuses
#[derive(serde::Deserialize, Clone)]
pub struct Stat {
    pub enable: bool,
    pub status_delay: u16,
    pub randomize: bool,
    pub status_list: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Reply {
    pub enable: bool,
    pub chance: u8,
    pub iterations: u8,
    pub iter_enable: bool,
    pub match_blacklist: Vec<String>,
    pub list: Vec<String>,
    pub trigger: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Warns {
    pub enable: bool,
}
