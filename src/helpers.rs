use serenity::client::Context;
use serenity::model::channel::Message;

use std::time;
use rand::thread_rng;
use rand::prelude::SliceRandom;
use tokio::time::sleep;

use crate::get_config;

// func for sending messages with batteries included
pub async fn send(ctx: &Context, msg: &Message, cnt: &str) {
    if let Err(why) = msg.reply(ctx, cnt).await {
        println!("Message Err: {:?}", why);
    }
} 

// generate a random 'idiot reply'
pub async fn idiot_reply() -> String {
    get_config().await
        .unwrap()
        .permissions.replies
        .choose(&mut thread_rng())
        .expect("Err Choosing an Idiot Reply!")
        .to_string() as String
}

// returns true if they match
pub async fn check_perms(ctx: &Context, msg: &Message, level: u8) -> bool { 
    let perms_config = get_config().await.unwrap().permissions;
    let user = &msg.author.id.to_string();


    let user_level = match user {
        _ if perms_config.mods.contains(user) => 1, 
        _ if perms_config.owners.contains(user) => 2, 
        _ => 0,
    };

    if user_level < level { 
        send(ctx, msg, &idiot_reply().await).await;
    }

    user_level >= level
}

// prompt utility, for making SURE a user wants to do something
pub async fn prompt_util(ctx: &Context, msg: &Message) -> bool {
    // send prompt message
    send(&ctx, &msg, "Are you Sure? (Y/n)").await;

    // wait for the confirmation message from the same user
    if let Some(response) = msg.author.await_reply(&ctx).await {
        // check if the response is y or Y
        if response.content.eq_ignore_ascii_case("y") {
            send(&ctx, &msg, "As You Wish...").await;
            return true;

        } else {
            send(&ctx, &msg, "Canceled!").await;
            return false;
        }
    }
    sleep(time::Duration::from_secs(15)).await;
    return false;
}
