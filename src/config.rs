use std::{env, fs};
use std::error::Error;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;


lazy_static! {
    static ref CONFIG: Mutex<Option<Conf>> = Mutex::new(None);
}
// struct to load the config into
#[derive(Serialize, Deserialize, Clone)]
pub struct Conf {
    pub token_file: String,
    pub permissions: Perms,
    pub status: Stat,
    pub replies: Reply,
    pub warns: Option<Vec<Warn>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Perms {
    pub owners: Vec<String>,
    pub mods: Vec<String>,
    pub replies: Vec<String>,
}

// config struct for statuses
#[derive(Serialize, Deserialize, Clone)]
pub struct Stat {
    pub enable: bool,
    pub status_delay: u16,
    pub randomize: bool,
    pub status_list: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Reply {
    pub enable: bool,
    pub chance: u8,
    pub iterations: u8,
    pub url_blacklist: bool,
    pub match_blacklist: Vec<String>,
    pub list: Vec<String>,
    pub trigger: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Warn {
    pub user: u64,
    pub reason: String,
    pub moderator: u64,
    pub time: u64,
}

// reload config func
pub async fn reload_config() -> Result<(), Box<dyn Error>> {
    let mut config = CONFIG.lock().await;

    // check env var, if empty pick the default
    let config_file = env::var("POT_CONFIG")
        .unwrap_or("config.toml".to_string());

    // load from a file
    let contents = fs::read_to_string(config_file)?;
        
    // return the parsed struct
    let parsed_config = toml::from_str::<Conf>(&contents)?.clone();

    // modify the config
    *config = Some(parsed_config);

    Ok(())
}

pub async fn get_config() -> Result<Conf, Box<dyn Error>> {
    let config = CONFIG.lock().await;
    // try retrieving config
    if let Some(config) = &*config {
        Ok(config.clone())
    } else {
        // if all fails...
        Err("Err: Config not Initialized".into())
    }
}

pub async fn modify_config(new_conf: Conf) -> Result<(), Box<dyn Error>> {
    // read end modify the config
    let mut conf = CONFIG.lock().await;
    *conf = Some(new_conf);

    // parse the conf into toml
    let parsed_new = toml::to_string(&*conf)?;

    // write the parsed conf to file
    fs::write("config.toml", parsed_new)?;

    Ok(())
}
