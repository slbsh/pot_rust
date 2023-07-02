use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::client::Context;

use std::fmt::Write;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use ndm::RollSet;

use crate::helpers::*; 
use crate::config::*;

// list warns
pub async fn command_ls(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    // check if user is allowed to do that
    if !check_perms(&ctx, &msg, 1).await? { return Ok(()); } 

    // get warns from config
    let warns_list = get_config().await?.warns;

    // gett all of the warns
    if warns_list.is_none() || warns_list.clone().unwrap().is_empty() {
        msg.reply(&ctx, "No Results!").await?;
        return Ok(());
    }       

    let mut message: String = Default::default();    

    // append each row as a new line
    for warn in warns_list.unwrap() {
        // parse the timestamp into local time
        let parsed_time = Utc.timestamp_opt(warn.time as i64, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M")
            .to_string();

        writeln!(message, "User: <@{}> \nReason: {} \nModerator: <@{}> \nTime: {}\n",
            warn.user, warn.reason, warn.moderator, parsed_time
        )?;
    }

    msg.reply(&ctx, &message).await?;
    Ok(())
}

// remove a warn
pub async fn command_rm(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // handling perms
    if !check_perms(&ctx, &msg, 1).await? { return Ok(()); } 

    let user = arg
        .trim()
        .replace(&['<', '>', '@'][..], "")
        .parse::<u64>();

    if user.is_err() {
        msg.reply(&ctx, &idiot_reply().await).await?;
        return Ok(());
    }

    let mut config = get_config().await?.clone();

    // remove the user that matches
    config.warns = config.warns.map(|v| {
        v.into_iter()
            .filter(|w| w.user != user.clone().unwrap())
            .collect()
    });
    
    // commit changes!
    modify_config(config).await?;

    msg.reply(&ctx, &format!("Removed <@{}> from Warns!", user.unwrap())).await?;
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
    if !check_perms(&ctx, &msg, 2).await? { return Ok(()); } 

    if prompt_util(ctx, msg).await? {
        println!("!shutdown recieved; Exiting...");
        std::process::exit(0); // goodbye everybody!
    }
    Ok(())
}

pub async fn command_warn(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // check perms
    if !check_perms(&ctx, &msg, 1).await? { return Ok(()); } 
            
    // split into user and reason
    let arg = arg.trim().split_once(" ").unwrap_or(("", ""));

    // make sure none are empty
    if arg.0.is_empty() || arg.1.is_empty() {
        msg.reply(&ctx, "Invalid Format\nTry: !warn <@member> <reason>").await?; 
        return Ok(());
    }

    // get current timestamp
    let timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    //parse arg into userid 
    let moderator: u64 = msg.author.id.into();
    let user: u64 = arg.0
        .replace(&['<', '>', '@'][..], "")
        .parse()
        .unwrap_or(0);

    // check if the user exists
    if let Err(_) = UserId::to_user(UserId(user), &ctx.http).await {
        msg.reply(&ctx, &idiot_reply().await).await?;
        return Ok(());
    } 

    let mut config = get_config().await?.clone();

    // fill out them fields boss
    let new_warn = Warn {
        user: user,
        reason: arg.1.to_string(),
        moderator: moderator,
        time: timestamp,
    };

    // add the new warn to the list
    if config.warns.is_none() {
        config.warns = Some(vec![new_warn])
    } else {
        config.warns.as_mut().map(|v| v.push(new_warn));
    }

    // commit the changes
    modify_config(config).await?;

    // confirm to user
    msg.reply(&ctx, "Warned!").await?;
    Ok(())
}

pub async fn command_reload(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    // perms check
    if !check_perms(&ctx, &msg, 2).await? { return Ok(()); } 

    if prompt_util(ctx, msg).await? {
        println!("Reloading Config...");
        reload_config().await?;
    }
    Ok(())
}


pub async fn command_delay(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // parse args into an int
    let delay = arg.trim().parse::<u16>();

    // check if the parsing returns an error
    if delay.is_err() {
        msg.reply(&ctx, "Invalid Arguments!").await?;
        return Ok(());
    }

    // if not, unwrap
    let delay = delay.unwrap();
    let mut config = get_config().await?.clone();

    config.status.status_delay = delay;

    // commit the changes
    modify_config(config).await?;

    // confirm to user
    msg.reply(&ctx, format!("Changed Status Delay to {} Seconds", delay)).await?;
    Ok(())
}
