use serenity::client::Context;
use serenity::model::channel::Message;

use std::error::Error;
use rand::thread_rng;
use rand::prelude::SliceRandom;

use crate::config::*;


// generate a random 'idiot reply'
pub async fn idiot_reply() -> String {
    CONFIG.read().await
        .permissions.replies
        .clone()
        .choose(&mut thread_rng())
        .expect("Err choosing an idiot reply!")
        .to_string() as String
}

// returns true if they match
pub async fn check_perms(ctx: &Context, msg: &Message, level: u8) -> Result<bool, Box<dyn Error>> { 
    let perms_config = &CONFIG.read().await.permissions;
    let user = &msg.author.id.to_string();


    let user_level = match user {
        _ if perms_config.mods.contains(user) => 1, 
        _ if perms_config.owners.contains(user) => 2, 
        _ => 0,
    };

    if user_level < level { 
        msg.reply(&ctx, &idiot_reply().await).await?;
    }

    Ok(user_level >= level)
}

// prompt utility, for making SURE a user wants to do something
pub async fn prompt_util(ctx: &Context, msg: &Message) -> Result<bool, Box<dyn Error>> {
    // send prompt message
    msg.reply(&ctx, "Are you Sure? (Y/n)").await?;

    // wait for the confirmation message from the same user
    if let Some(response) = msg.author.await_reply(ctx).await {
        // check if the response is y or Y
        if response.content.eq_ignore_ascii_case("y") {
            msg.reply(&ctx, "As You Wish...").await?;
            return Ok(true);

        }

        msg.reply(&ctx, "Canceled!").await?;
    }

    Ok(false)
}

// bernoulli helper
