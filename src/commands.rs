use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::client::Context;

use std::fmt::Write;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use ndm::RollSet;

use crate::helpers::*; 

use crate::warns::*;


// test command for debugging
pub async fn command_test(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    msg.reply(&ctx, "Test").await?;
    Ok(())
}


// list warns
pub async fn command_ls(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    // check if user is allowed to do that
    if !check_perms(ctx, msg, 1).await? { return Ok(()); } 

    let warns = WARNS.lock().await;

    if warns.is_empty() {
        msg.reply(&ctx, "No Results!").await?;
        return Ok(());
    }       

    let mut message = String::new();    

    // append each row as a new line
    for warn in warns.iter() {
        // parse the timestamp into local time
        let parsed_time = Utc.timestamp_opt(warn.time as i64, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M")
            .to_string();

        writeln!(message, "User: <@{}> \nReason: {} \nModerator: <@{}> \nTime: {}\n",
            warn.user, warn.resn, warn.modr, parsed_time
        )?;
    }

    msg.reply(&ctx, &message).await?;
    Ok(())
}

// remove a warn
pub async fn command_rm(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // handling perms
    if !check_perms(ctx, msg, 1).await? { return Ok(()); } 

    let arg = arg.trim().replace(&['<', '>', '@'][..], "");
    let Ok(user) = arg.parse::<u64>() else {
        msg.reply(&ctx, &idiot_reply().await).await?;
        return Ok(());
    };

    let mut warns = WARNS.lock().await;

    warns.retain(|w| w.user != user);
    Warns::write(&warns);

    msg.reply(&ctx, &format!("Removed <@{}> from Warns!", user)).await?;
    Ok(())
}

pub async fn command_roll(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    //parse the args into a dice roll
    if let Ok(roll) = arg.parse::<RollSet>() {
        msg.reply(&ctx, &format!("{}", roll)).await?;
        return Ok(());
    }
    // if the roll fails to parse
    msg.reply(&ctx, &idiot_reply().await).await?;
    Ok(())
}

pub async fn command_shutdown(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    // check if the user is in the owner group
    if !check_perms(ctx, msg, 2).await? { return Ok(()); } 

    if prompt_util(ctx, msg).await? {
        println!("!shutdown recieved; Exiting...");
        std::process::exit(0); // goodbye everybody!
    }
    Ok(())
}

pub async fn command_warn(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // check perms
    if !check_perms(ctx, msg, 1).await? { return Ok(()); } 
            
    // split into user and reason
    let (user, reason) = match arg.trim().split_once(' ') {
        Some(e) => e,
        None => {
            msg.reply(&ctx, "Invalid Format\nTry: !warn <@member> <reason>").await?; 
            return Ok(());
        },
    };

    // get current timestamp
    let time: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    //parse arg into userid 
    let modr: UserId = msg.author.id;
    let user = UserId(user
        .replace(&['<', '>', '@'][..], "")
        .parse()
        .unwrap_or(0));

    // check if the user exists
    if UserId::to_user(user, &ctx.http).await.is_err() {
        msg.reply(&ctx, &idiot_reply().await).await?;
        return Ok(());
    } 

    // fill out them fields boss
    let new_warn = Warns {
        user,
        resn: reason.to_string(),
        modr,
        time,
    };

    let mut warns = WARNS.lock().await;

    warns.push(new_warn);
    Warns::write(&warns);

    // confirm to user
    msg.reply(&ctx, "Warned!").await?;
    Ok(())
}

