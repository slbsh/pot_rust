use serenity::client::Context;
use serenity::model::channel::Message;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use rand::distributions::{Bernoulli, Distribution};

use rand::thread_rng;
use rand::seq::SliceRandom;
use std::error::Error;

use crate::config::get_config;

static REPLY_CHANCE: Lazy<Mutex<Option<Bernoulli>>> = Lazy::new(|| Mutex::new(None));

pub async fn init_bern() -> Result<(), Box<dyn Error>> {
    // lock ma boi bern so we can modify
    let mut bern_lock = REPLY_CHANCE.lock().await;

    // read the chance from config and set that as the new value
    let chance = get_config().await?.replies.chance as f64;
    *bern_lock = Some(Bernoulli::new(1.0 / chance)?);

    Ok(())
}

pub async fn handle_reply(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    
    let repl = get_config().await?.replies;

    // enabled? no bitches?
    if !repl.enable { return Ok(()); }

    let message = msg.content.to_lowercase();

    // ignore links if they're disabled in config
    // on discord that means images, gifs, &c
    if !repl.url_blacklist && message.trim().starts_with("http") 
    { return Ok(()); }
    
    let bern = REPLY_CHANCE.lock().await.unwrap().clone();

    // only send the message contains a trigger word or 1 in x chance
    if !(repl.trigger.iter().any(|t| message.contains(&t.to_lowercase())) 
        || bern.sample(&mut thread_rng()))
    { return Ok(()); }

    // shuffle the word list and pick as many as the iterations we want
    let mut rand_replies = repl.list.clone();
    rand_replies.shuffle(&mut thread_rng());
    let rand_replies: Vec<String> = rand_replies
        .into_iter()
        .take(repl.iterations as usize)
        .collect();

    // check if a random reply and the message share a word
    // if not, pick another random reply 
    // after x failed attempts just send the last one
    for (i, reply) in rand_replies.iter().enumerate() {
        // compare the words of the reply to the message,
        // ignoring blacklisted ones
        let is_match: bool = message
            .split_whitespace()
            .filter(|w| !repl.match_blacklist.contains(&w.to_string()))
            .any(|w| reply.contains(w));

        // send anyway if the number of attempts is over a threshold
        if i == repl.iterations as usize - 1 || is_match {
            msg.reply(&ctx, reply).await?;
            return Ok(());
        }
    }
    Ok(())
}
