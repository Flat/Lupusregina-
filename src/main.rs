#[macro_use] extern crate log;
extern crate serenity;

extern crate env_logger;
extern crate kankyo;

mod commands;

use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn message(&self, _: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say("Pong!") {
                error!("Error sending message: {:?}", why);
            }
        }
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

fn main() {
    kankyo::load().expect("Failed to load .env file");
    env_logger::init();

    let token = env::var("DISCORD_TOKEN")
    .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).expect("Error creating client");

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
