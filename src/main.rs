#![feature(try_trait)]
#[macro_use]
extern crate log;
extern crate serenity;

extern crate directories;
extern crate env_logger;
extern crate ini;
extern crate kankyo;
extern crate rusqlite;
extern crate typemap;

pub mod commands;
pub mod db;
pub mod util;

use ini::Ini;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;

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
    kankyo::init().expect("Failed to load .env file");
    env_logger::init();

    let conf: Ini;

    if let Some(project_dirs) = util::get_project_dirs() {
        let config_path = project_dirs.config_dir().join("settings.ini");
        if let Ok(_conf) = Ini::load_from_file(config_path) {
            conf = _conf;
        } else {
            panic!(
                "Failed to load {:?}",
                project_dirs.config_dir().join("settings.ini")
            )
        };
        project_dirs.config_dir();
    } else {
        panic!("Failed to get config dir!");
    }

    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");

    db::create_db();

    let mut client = Client::new(&token, Handler).expect("Error creating client");

    client.with_framework(
        StandardFramework::new()
            .on("ping", |_, msg, _| {
                msg.channel_id.say("pong")?;
                Ok(())
            }).configure(|c| {
                c.dynamic_prefix(|_, msg| {
                    let default = ".".to_owned();
                    if let Some(guild_id) = msg.guild_id {
                        if let Ok(prefix) = db::get_guild_prefix(guild_id) {
                            Some(prefix)
                        } else {
                            Some(default)
                        }
                    } else {
                        Some(default)
                    }
                })
            }),
    );

    {
        let mut data = client.data.lock();
        data.insert::<util::Config>(Arc::clone(&Arc::new(conf)));
    }

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
