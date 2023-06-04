use std::fmt::Write;
use chrono::Local;
use serenity::model::channel::Message;
use serenity::client::Context;
use std::process::{exit, Command};
use crate::{Handler, CONFIG};
use rand::thread_rng;
use sqlx::query;
use rand::prelude::SliceRandom;

// func for sending messages with batteries included
pub async fn send(ctx: &Context, msg: &Message, cnt: &str) {
    if let Err(why) = msg.reply(ctx, cnt).await {
        println!("Message Err: {:?}", why);
    }
} 

fn idiot_reply() -> String {
    let idiot_reply: String = CONFIG.permissions.replies
        .choose(&mut thread_rng())
        .expect("Err Choosing an Idiot Reply!")
        .to_string();

    idiot_reply
}

// get user permissions
// 0 - none, 1 - mod, 2 - owner
fn get_perms(msg: &Message) -> u8 { 
    if CONFIG.permissions.mods.contains(&msg.author.id.to_string()) {
        return 1;
    }

    if CONFIG.permissions.owners.contains(&msg.author.id.to_string()) {
        return 2;
    }
    
    return 0;
}

// list warns
pub async fn command_ls(handler: &Handler, ctx: &Context, msg: &Message) {
    // check if user is allowed to do that
    if get_perms(msg) == 0 {
        send(&ctx, &msg, &idiot_reply()).await;
        return;
    } 

    // read the whole warns table
    let warns_list = query!("SELECT * FROM warns;")
        .fetch_all(&handler.database)
        .await
        .expect("Database Err: Read");

    let mut message: String = Default::default();    

    // append each row as a new line
    for warn in warns_list {
        writeln!(message, "{} | {} | <@{}> | {}",
            warn.usr.as_deref().expect("Database Err: No such Field"),
            warn.rsn.as_deref().expect("Database Err: No such Field"),
            warn.mdr.as_deref().expect("Database Err: No such Field"),
            warn.tme.as_deref().expect("Database Err: No such Field"),
        ).expect("Shit Went Down!");
    }

    if message.is_empty() {
        send(&ctx, &msg, "No Results!").await;
        return;
    }       

    // the title for tellin the user which field is what
    let title = "Member | Reason | Moderator | Timestamp\n";
    message.insert_str(0, title);

    send(&ctx, &msg, &message).await;
}

// remove a warn
pub async fn command_rm(handler: &Handler, ctx: &Context, msg: &Message, arg: &str) {
    // handling perms
    if get_perms(msg) == 0 {
        send(&ctx, &msg, &idiot_reply()).await;
        return;
    }

    let arg = arg.trim();

    if arg.is_empty() {
        send(&ctx, &msg, "Specify a User, stoopid").await;
        return;
    }

    let exists: bool = query!("SELECT usr FROM warns;")
            .fetch_all(&handler.database)
            .await
            .expect("Database Read Err!")
            .iter()
            .any(|row| row.usr.as_deref() == Some(arg));
            
    if !exists {
        send(&ctx, &msg, "No such User in the Database!").await;
        return;
    }

    query!("DELETE FROM warns WHERE usr = ?1", arg,)
        .execute(&handler.database)
        .await
        .expect("Err Deleting from Database");
}

pub async fn command_roll(ctx: &Context, msg: &Message, arg: &str) {
    // run the roll command
    let out = Command::new("roll")
        .arg("-vs").arg(arg)
        .output()
        .expect("Failed to run Roll Command!")
        .stdout;
    let out = String::from_utf8_lossy(&out).to_string();

    // split output into separate lines
    let mut out_lines: Vec<&str> = out.lines().collect(); 
            
    // remove the second to last line of the output
    if out_lines.len() > 1 {
        out_lines.remove(out_lines.len() - 2);
    }

    // merge the lines back together
    let out = out_lines.join("\n");

    if out.is_empty() {
        send(&ctx, &msg, &idiot_reply()).await;
        panic!("Shit Went Down!");
    } 
    
    send(&ctx, &msg, &out).await;
}

pub async fn command_shutdown(ctx: &Context, msg: &Message) {
    // check if the user is in the owner group
    if get_perms(msg) <= 1 {
        send(&ctx, &msg, &idiot_reply()).await;
        return;
    }

    send(&ctx, &msg, "Are you Sure? (Y/n)").await;

    // wait for the confirmation message from the same user
    if let Some(response) = msg.author.await_reply(&ctx).await {
        // check if the response is y or Y
        if response.content.eq_ignore_ascii_case("y") {
            send(&ctx, &msg, "As You Wish...").await;
            println!("!shutdown recieved; Exiting...");
            exit(0); // goodbye everybody!

        } else {
            send(&ctx, &msg, "Canceled!").await;
        }
    }
}

pub async fn command_warn(handler: &Handler, ctx: &Context, msg: &Message, arg: &str) {
    // check perms
    if get_perms(msg) == 0 { 
        send(&ctx, &msg, &idiot_reply()).await;
        return; 
    }
            
    // split into tuple
    let arg = arg.split_once(" ").unwrap_or(("", ""));

    // check if tuple is empty
    if arg.0.is_empty() || arg.1.is_empty() {
        send(&ctx, &msg, "Invalid Format\nTry: !warn <@member> <reason>").await; 
        return;
    }

    let current_time = Local::now()
        .format("%d-%m-%Y %H:%M")
        .to_string();
    let user: String = msg.author.id.to_string();

    // database shenanigans
    // collecting: moderator that did the warn, current time, 
    // user that has been warned, the reason for the warn
    // TODO check if the user actually exist before adding to database
    query!("INSERT INTO warns (usr, rsn, mdr, tme) VALUES (?1, ?2, ?3, ?4);",
        arg.0, arg.1, user, current_time,
    )
    .execute(&handler.database)
    .await
    .expect("Err Insering into Database");
            
    send(&ctx, &msg, "Warned!").await;
}
