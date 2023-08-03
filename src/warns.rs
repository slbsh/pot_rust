use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use serenity::model::prelude::UserId;

use std::{fs, env};


pub static WARNS: Lazy<Mutex<Vec<Warns>>> = Lazy::new(|| Mutex::new(Warns::init()));

#[derive(Clone)]
pub struct Warns {
    pub user: UserId,
    pub resn: String,
    pub modr: UserId,
    pub time: u64,
}


impl Warns {
    fn init() -> Vec<Self> {
        // load from a file
        let contents = fs::read_to_string(get_warns_file())
            .expect("Failed to read config");

        if contents.is_empty() {
            return Vec::new();
        }

        let mut wrn = Warns {
            user: UserId(0),
            resn: String::new(),
            modr: UserId(0),
            time: 0,
        };

        let mut out: Vec<Warns> = Vec::new();

        for (i, ln) in contents.lines().enumerate() {
            let (key, val) = match ln.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => panic!("Failed to parse line {}", i+1),
            };

            match key {
                "user" => wrn.user = UserId(to_u64(val)),
                "resn" => wrn.resn = String::from(val),
                "modr" => wrn.modr = UserId(to_u64(val)),
                "time" => wrn.time = to_u64(val),
                &_ => panic!("Unrecognised key on line {}", i+1),
            }

            if (i + 1) % 5 == 0 {
                out.push(wrn.clone());
            }
        } out
    }

    pub fn write(warns: &[Self]) {
        let mut out = String::new();
        warns.iter().for_each(|w| out.push_str(&format!("user = {}\nresn = {}\nmodr = {}\ntime = {}\n\n", w.user, w.resn, w.modr, w.time)));

        fs::write(get_warns_file(), &out)
            .expect("Failed to Write to File");
    }
}


fn get_warns_file() -> String {
    // check env var, if empty pick the default
    env::var("POT_CONFIG").unwrap_or("warns.omf".to_string())
}


fn to_u64(s: &str) -> u64 {
    s.parse::<u64>().expect("Failed to parse &str -> u64")
}
