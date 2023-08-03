use std::{env, fs};

use serde::Deserialize;
use once_cell::sync::Lazy;

// private function to process config file
// pub static CONFIG: Lazy<RwLock<Conf>> = Lazy::new(|| RwLock::new(Conf::init()));


// read config file and initialize a conf struct
fn init_config() -> Conf {
    let contents = fs::read_to_string(get_config_file())
        .expect("Failed to read config");

    toml::from_str(&contents)
        .expect("Failed to parse config")
}

// get the config file path from the enviroment
fn get_config_file() -> String {
    match env::var("POT_CONFIG") {
        Ok(path) => path,
        Err(_) => panic!("POT_CONFIG not set"),
    }
}

// get the warns file path from the enviroment
fn get_warns_file() -> String {
    match env::var("POT_WARNS") {
        Ok(path) => path,
        Err(_) => panic!("POT_WARNS not set"),
    }
}

// convert a string to a u64
fn to_u64(s: &str) -> u64 {
    match s.parse::<u64>() {
        Ok(n) => n,
        Err(_) => panic!("Failed to parse string"),
    }
}

// convert a string to a u8
fn to_u8(s: &str) -> u8 {
    match s.parse::<u8>() {
        Ok(n) => n,
        Err(_) => panic!("Failed to parse string"),
    }
}



// struct to load the config into
#[derive(Deserialize)]
pub struct Conf {
    pub token_file: String,
    pub prefix: char,
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

#[derive(Deserialize)]
pub struct Reply {
    pub enable: bool,
    pub chance: u8,
    pub iterations: u8,
    pub url_blacklist: bool,
    pub match_blacklist: Vec<String>,
    pub list: Vec<String>,
    pub trigger: Vec<String>,
}

impl Conf {
    fn init() -> Self {
        // check env var, if empty pick the default
        let config_file = env::var("POT_CONFIG")
            .unwrap_or("config.toml".to_string());

        // load from a file
        let contents = fs::read_to_string(config_file)
            .expect("Failed to read config");
            
        // return the parsed struct
        toml::from_str::<Conf>(&contents)
            .expect("Failed to Parse Config")
    }
}

