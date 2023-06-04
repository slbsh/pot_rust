use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::{Ready, Activity};

use std::{fs, time, env};
use tokio::time::sleep;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::process::exit;

use crate::config::{get_config, reload_config};
use crate::commands::*;
use crate::replies::{handle_reply, init_bern};

mod config;
mod commands;
mod helpers;
mod replies;

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
        let conf = &mut get_config().await
            .unwrap()
            .clone()
            .status;

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

        /*
         * Command Block! 
         * here we parse the input and call commands if needed
         */

        if msg.content.trim() == "!ls" {
            command_ls(&self, &ctx, &msg).await;
            return;
        }

        if let Some(arg) = msg.content.strip_prefix("!rm") {
            command_rm(&self, &ctx, &msg, arg).await;
            return;
        }

        if msg.content.trim() == "!reload" {
            command_reload(&ctx, &msg).await;
            return;
        }

        if let Some(arg) = msg.content.strip_prefix("!r") {
            command_roll(&ctx, &msg, arg).await;
            return;
        }

        if msg.content.trim() == "!shutdown" {
            command_shutdown(&ctx, &msg).await;
            return;
        }

        if let Some(arg) = msg.content.strip_prefix("!warn") {
            command_warn(&self, &ctx, &msg, arg).await;
            return;
        }
        
        // if there's no valid command, run the reply handler!
        handle_reply(&ctx, &msg).await;
    }
}

/*
 * The Legendary main() [this is where the big guns at]
 */
#[tokio::main]
async fn main() {
    // initialize the config file
    reload_config().await.unwrap();

    // initialize Brenoulli for replies
    init_bern().await.unwrap();

    if env::args().skip(1).any(|a| a == "-h" || a == "--help") {
        println!("{}", HELP_MESSAGE);
        exit(0);
    }

    // read the token file
    let token_file = get_config().await.unwrap().token_file;
    let token = fs::read_to_string(token_file)
            .expect("Could not read token file!");

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
