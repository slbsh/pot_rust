use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::{Ready, Activity};

use std::{fs, time, env};
use tokio::time::sleep;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::process::exit;
use once_cell::sync::Lazy;
use rand::distributions::Bernoulli;

use crate::config::*;
use crate::commands::*;

mod config;
mod commands;

//loading the config into a static
pub static CONFIG: Lazy<Conf> = Lazy::new(|| {
    // check env var, if empty pick the default
    let config_file = env::var("POT_CONFIG")
        .unwrap_or("config.toml".to_string());

    // load from a file
    let contents = fs::read_to_string(config_file)
       .expect("Err reading Config");
        
    // return the parsed struct
    toml::from_str::<Conf>(&contents)
        .expect("Err parsing Config")
        .clone()
}); 

static REPLY_CHANCE: Lazy<Bernoulli> = Lazy::new(|| 
    Bernoulli::new(1.0 / CONFIG.replies.chance as f64)
        .expect("Err Creating a Bernoulli Distribution!")
);

//passed to commands n such
pub struct Handler { 
    database: sqlx::SqlitePool,
}

static HELP_MESSAGE: &str =
"pot - The WickedWiz Discord Bot!
Usage: pot <OPTION>

Options:
  -h, --help          Show This Message

Enviroment:
  POT_CONFIG=<path>   Specify the Config File (toml)";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Ready! Connected as {}", ready.user.name);
        // load stuff from config
        let conf = &mut CONFIG.clone().status;

        // return if the status feature isn't enabled
        if !conf.enable { return; }

        loop {
            // shuffle the list if it's enabled in config
            if conf.randomize {
                conf.status_list.shuffle(&mut thread_rng());
            }

            for status in &conf.status_list {
                // set the playing status
                ctx.set_activity(Activity::playing(status)).await;
                sleep(time::Duration::from_secs(conf.status_delay.into())).await;
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // check if user is bot to prevent unwanted replies
        if msg.author.bot { return; }

        if msg.content == "!ls" {
            command_ls(&self, &ctx, &msg).await;
            return;
        }

        if let Some(arg) = msg.content.strip_prefix("!rm") {
            command_rm(&self, &ctx, &msg, arg).await;
            return;
        }

        if let Some(arg) = msg.content.strip_prefix("!r") {
            command_roll(&ctx, &msg, arg).await;
            return;
        }

        if msg.content == "!shutdown" {
            command_shutdown(&ctx, &msg).await;
            return;
        }

        if let Some(arg) = msg.content.strip_prefix("!warn") {
            command_warn(&self, &ctx, &msg, arg).await;
            return;
        }

        /*
         *  Reply module!
         */

        // enabled? no bitches?
        if !CONFIG.replies.enable { return; }

        // shorthands
        let repl = &CONFIG.replies;
        let message = msg.content.to_lowercase();

        // ignore links if they're disabled in config
        // on discord that means images, gifs, &c
        if !repl.url_blacklist && message.trim().starts_with("http") 
        { return; }

        // only send the message contains a trigger word or 1 in x chance
        if !(repl.trigger.iter().any(|t| message.contains(&t.to_lowercase())) 
            || thread_rng().sample(&*REPLY_CHANCE))
        { return; }

        // shuffle the word list and pick as many as the iterations we want
        let mut rand_replies = repl.list.clone();
        rand_replies.shuffle(&mut thread_rng());
        let rand_replies: Vec<String> = rand_replies
            .into_iter()
            .take(repl.iterations as usize)
            .collect();

        // check if a random reply and the message share a word
        // if not, pick another random reply 
        // after x failed attempts just send the last one
        for (i, reply) in rand_replies.iter().enumerate() {
            // compare the words of the reply to the message,
            // ignoring blacklisted ones
            let is_match: bool = message
                .split_whitespace()
                .filter(|w| !repl.match_blacklist.contains(&w.to_string()))
                .any(|w| reply.contains(w));

            // send anyway if the number of attempts is over a threshold
            if i == repl.iterations as usize - 1 || is_match {
                send(&ctx, &msg, reply).await;
                return;
            }
        }
    }
}

/*
 * The Legendary main() [this is where the big guns at]
 */
#[tokio::main]
async fn main() {
    if env::args().skip(1).any(|a| a == "-h" || a == "--help") {
        println!("{}", HELP_MESSAGE);
        exit(0);
    }

    // read config for token, if empty read config for the token file
    // then read that file to extract the token
    let token: String = CONFIG.token.as_ref()
        .map(|t| t.to_string())
        .or_else(|| {
            CONFIG.token_file.as_ref().and_then(|file| {
                Some(fs::read_to_string(file)
                    .expect("Could not read token file!"))
            })
        })
        .expect("No token provided!");

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        // change this if more throughput is needed
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("pot_stuff.db")
                .create_if_missing(true),
        )
        .await
        .expect("Err Loading Database!");

    let handler = Handler { 
        database,
    };

    // additional intents go here!
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Err Creating Client!");

    if let Err(why) = client.start().await {
        println!("Client Err: {:?}", why)
    }
}
