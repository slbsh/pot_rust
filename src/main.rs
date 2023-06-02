use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::{Ready, Activity};

use std::{fs, time};
use tokio::time::sleep;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::process::exit;
use once_cell::sync::Lazy;

use crate::config::*;
use crate::commands::*;

mod config;
mod commands;

//loading the config into a static
pub static CONFIG: Lazy<Conf> = Lazy::new(|| {
    // load from a file
    let contents = fs::read_to_string(&read_args())
       .expect("Err reading Config");
        
    // return the parsed struct
    toml::from_str::<Conf>(&contents)
        .expect("Err parsing Config")
        .clone()
}); 

//passed to commands n such
pub struct Handler { 
    database: sqlx::SqlitePool,
}

static HELP_MESSAGE: &str = "
pot - The WickedWiz Discord Bot!
Usage: pot <OPTION>

Options:
  -h, --help     Show This Message
  -c, --config   Change the config file (toml)";

fn read_args() -> String {
    // Handling stdin args
    let args: Vec<String> = std::env::args().collect();
    let config_file: &str;

    // check if any args are given
    if let Some(arg) = args.get(1) {
       match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP_MESSAGE);
                exit(0);
            },
            "-c" | "--config" => config_file = &args
                .get(2)
                .expect("No config File Given after the -c arg!"),
            _ => {
                println!("Invalid Argument!");
                exit(2);
            }
       } 
    } else {
        println!("No args provided! Using the defaults.");
        config_file = "config.toml";
    }

    config_file.to_string()
} 


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


        // quick shorthand
        let repl = &CONFIG.replies;

        // enabled? no bitches?
        if !repl.enable { return; }

        let message = msg.content.to_lowercase();

        // only send the message contains a trigger word or 1 in x chance
        if !repl.trigger.iter().any(|s| message.contains(s)) 
            || !thread_rng().gen_range(0..repl.chance) == 0 
        { return; }

        // check if a random reply and the message share a word
        // if not, pick another random reply 
        // after 3 failed attempts just send the last one
        let mut attempts = 0;
        loop {
            // pick a random reply from the list
            let rand_reply: &str = repl.list
                .choose(&mut thread_rng())
                .expect("Failed to Pick a Pot Reply");
                
            // dont check for matching words if it's a link
            // on discord that also means image, gif, &c.
            if message.starts_with("http") {
                send(&ctx, &msg, rand_reply).await;
                break;
            }

            // split the random reply into separate words so we can compare
            let reply_words: Vec<&str> = rand_reply
                .split_whitespace()
                .collect();

            attempts += 1;

            if reply_words.iter().any(|w| message.contains(w)) || attempts >= repl.match_iter {
                send(&ctx, &msg, rand_reply).await;
                break;
            }
        }
    }
}

/*
 * The Legendary main() [this is where the big guns at]
 */
#[tokio::main]
async fn main() {
    let token = fs::read_to_string(&CONFIG.token_file)
        .expect("Err Reading Token File!");

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
