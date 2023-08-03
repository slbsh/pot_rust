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
pub struct Handler { }

static HELP_MESSAGE: &str =
"pot - The WickedWiz Discord Bot!
Usage: pot <OPTION>

Options:
  -h, --help          Show This Message

Enviroment:
  POT_CONFIG=<path>   Specify the Config File (toml)";

async fn command_handler(ctx: &Context, msg: &Message) -> Result<bool, Box<dyn std::error::Error>> {
    // split args into the command and its arguments
    let arg = msg.content.split_once(' ').unwrap_or(("", ""));

    // returned by the handler, indicates if a command was detected and ran
    let mut cflg = true;

    // match to existing commands
    match arg.0 {
        "!ls" => command_ls(ctx, msg).await?,
        "!rm" => command_rm(ctx, msg, arg.1).await?,
        "!reload" => command_reload(ctx, msg).await?,
        "!r" => command_roll(ctx, msg, arg.1).await?,
        "!shutdown" => command_shutdown(ctx, msg).await?,
        "!warn" => command_warn(ctx, msg, arg.1).await?,
        "!delay" => command_delay(ctx, msg, arg.1).await?,
        "!test" => command_test(ctx, msg).await?,
        &_ => cflg = false,
    } 

    Ok(cflg)
}


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Ready! Connected as {}", ready.user.name);
        // load stuff from config
        println!("Loading Config...");
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
                println!("Shuffling Status List...");
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

        // here we parse the input and call commands if needed
        match command_handler(&ctx, &msg).await {
            Err(why) => { 
                eprintln!("Err running the Remove command!: {}", why);
                return;
            },
            // return if a command was ran
            Ok(flag) => if flag { return; }
        }

        // if there's no valid command, run the reply handler!
        if let Err(why) = handle_reply(&ctx, &msg).await {
            eprintln!("Err running the List command!: {}", why);
        }
    }
}

/*
 * The Legendary main() [this is where the big guns at]
 */
#[tokio::main]
async fn main() {
    // initialize the config file
    reload_config().await
        .expect("Failed to initialize config!");

    // initialize Brenoulli for replies
    init_bern().await
        .expect("Failed to initialize the Bernoulli Distribution!");
    // println!("Brenoulli Initialized! {}", bern_lock().await);
    if env::args().skip(1).any(|a| a == "-h" || a == "--help") {
        println!("{}", HELP_MESSAGE);
        exit(0);
    }

    // read the token file
    let token_file = get_config().await.unwrap().token_file;
    let token = fs::read_to_string(token_file)
            .expect("Could not read token file!");
    
    let handler = Handler { };

    // additional intents go here!
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Err Creating Client!");

    if let Err(why) = client.start().await {
        eprintln!("Client Err: {:?}", why)
    }
}
