use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::config::*;
use std::{fs, env};

pub static WARNS: Lazy<Mutex<Vec<Warns>>> = Lazy::new(|| Mutex::new(Warns::init()));

#[derive(Serialize, Deserialize)]
pub struct Warns {
    pub user: u64,
    pub reason: String,
    pub moderator: u64,
    pub time: u64,
}

// reload config func
//
impl Warns {
    fn init() -> Vec<Self> {
        // load from a file
        let contents = fs::read_to_string(get_warns_file())
            .expect("Failed to read config");
            
        // return the parsed struct
        toml::from_str::<Vec<Warns>>(&contents)
            .expect("Failed to Parse Config")
    }

    pub fn write(thing: &Vec<Self>) {
        // parse the conf into toml
        let parsed_new = toml::to_string(thing)
            .expect("Failed to Parse to toml");

        fs::write(get_warns_file(), parsed_new)
            .expect("Failed to Write to File");
    }
}

fn get_warns_file() -> String {
    // check env var, if empty pick the default
    env::var("POT_CONFIG").unwrap_or("warns.toml".to_string())
}
