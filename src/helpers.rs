use serenity::client::Context;
use serenity::model::channel::Message;
use rand::distributions::Bernoulli;
use once_cell::sync::Lazy;
use std::error::Error;
use rand::thread_rng;
use rand::prelude::SliceRandom;

use crate::config::*;


// generate a random 'idiot reply'
pub async fn idiot_reply() -> String {
    CONFIG .permissions.replies
        .clone()
        .choose(&mut thread_rng())
        .expect("Err choosing an idiot reply!")
        .to_string() as String
}

#[derive(PartialEq)]
enum Perms {
    Mod   = 1,
    Owner = 2,
    None,
}

// returns true if they match
pub async fn check_perms(ctx: &Context, msg: &Message, level: u8) -> Result<bool, Box<dyn Error>> { 
    let prms = &CONFIG.permissions;
    let usr = msg.author.id;

    let user_level = match usr {
        u if prms.modr.contains(&u) => Perms::Mod, 
        u if prms.ownr.contains(&u) => Perms::Owner, 
        _ => Perms::None,
    };

    if user_level == Perms::None {
        return Ok(false);
    }

    let user_level = user_level as u8;
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
    let Some(response) = msg.author.await_reply(ctx).await else {
        return Ok(false);
    };

    // check if the response is y or Y
    if !response.content.eq_ignore_ascii_case("y") {
        msg.reply(&ctx, "Canceled!").await?;

    }

    msg.reply(&ctx, "As You Wish...").await?;
    Ok(true)

}

// bernoulli helper
pub static BERN: Lazy<Bernoulli> = Lazy::new(|| {
    let initial_chance = match CONFIG.replies.chance {
        0 => panic!("reply chance cannot be 0"),
        n=> n as f64,
    };
    Bernoulli::new(1.0 / initial_chance)
        .expect("Failed to initialize Bernoulli distribution")
});
