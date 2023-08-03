use serenity::client::Context;
use serenity::model::channel::Message;

use rand::distributions::{Bernoulli, Distribution};
use rand::thread_rng;
use rand::seq::SliceRandom;

use std::error::Error;






use crate::config::*;

pub async fn reply_handler(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    
    let repl = &CONFIG.read().await.replies;

    // enabled? no bitches?
    if !repl.enable { return Ok(()); }

    let message = msg.content.to_lowercase();

    // ignore links if they're disabled in config
    // on discord that means images, gifs, &c
    if !repl.url_blacklist && message.trim_start().starts_with("http") 
    { return Ok(()); }
    
    // read the chance from config, check if 0
    let chance = match CONFIG.read().await.replies.chance {
        0 => return Err("reply chance cannot be 0".into()),
        n => n as f64,
    };
    // do the bern
    let bern = Bernoulli::new(1.0 / chance)?;
    
    // only send the message contains a trigger word or 1 in x chance
    if !message.contains("pot") && !bern.sample(&mut thread_rng())
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
