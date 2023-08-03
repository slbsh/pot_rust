use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::{Ready, Activity};

use std::{fs, time, env};
use tokio::time::sleep;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::process::exit;

use crate::config::*;
use crate::commands::*;
use crate::replies::reply_handler;

mod config;
mod commands;
mod helpers;
mod replies;
mod warns;

//passed to commands n such
pub struct Handler;

static HELP_MESSAGE: &str =
"pot - The WickedWiz Discord Bot!
Usage: pot <OPTION>

Options:
  -h, --help          Show This Message

Enviroment:
  POT_CONFIG=<path>   Specify the Config File (toml)";

async fn command_handler(ctx: &Context, msg: &Message) -> Result<(), Box<dyn std::error::Error>> {
    // split args into the command and its arguments
    let (cmd, arg) = match msg.content.split_once(' ') {
        Some(a) => a,
        None => {
            msg.reply(&ctx, "Missing Args").await?;
            return Ok(());
        },
    };

    // match to existing commands
    match cmd {
        "!ls"       => command_ls(ctx, msg).await?,
        "!rm"       => command_rm(ctx, msg, arg).await?,
        "!r"        => command_roll(ctx, msg, arg).await?,
        "!shutdown" => command_shutdown(ctx, msg).await?,
        "!warn"     => command_warn(ctx, msg, arg).await?,
        &_ => { msg.reply(&ctx, &format!("Invalid Cmd `{cmd}`")).await?; },
    } 

    Ok(())
}


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Ready! Connected as {}", ready.user.name);

        // return if the status feature isn't enabled
        if !CONFIG.status.enable { return; }

        let mut list = CONFIG.status.status_list.clone();

        loop {
            // shuffle the list if it's enabled in config
            if CONFIG.status.randomize {
                list.shuffle(&mut thread_rng());
            }

            for s in &list {
                ctx.set_activity(Activity::playing(s)).await;
                sleep(time::Duration::from_secs(CONFIG.status.status_delay.into())).await;
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // check if user is bot to prevent unwanted replies
        if msg.author.bot { return; }

        // check prefix
        if msg.content.starts_with(CONFIG.prefix) {
            // here we parse the input and call commands if needed
            if let Err(why) = command_handler(&ctx, &msg).await {
                eprintln!("CMDERR: {}", why);
                return;
            }
        }

        if let Err(why) = reply_handler(&ctx, &msg).await {
            eprintln!("RPLERR: {}", why);
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

    // read the token file
    let token = fs::read_to_string(&CONFIG.token_file)
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
