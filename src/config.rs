// struct to load the config into
#[derive(serde::Deserialize, Clone)]
pub struct Conf {
    pub token_file: String,
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
    pub match_iter: u8,
    pub list: Vec<String>,
    pub trigger: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Warns {
    pub enable: bool,
}
