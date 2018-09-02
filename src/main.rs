#![feature(try_trait)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate serenity;

extern crate chrono;
extern crate directories;
extern crate env_logger;
extern crate ini;
extern crate kankyo;
extern crate rand;
extern crate rusqlite;
extern crate typemap;

#[macro_use]
pub mod util;
pub mod commands;
pub mod db;

use chrono::Utc;
use ini::Ini;
use serenity::framework::standard::{help_commands, HelpBehaviour, StandardFramework};
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Arc;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        let mut data = ctx.data.lock();
        match data.get_mut::<util::Uptime>() {
            Some(uptime) => {
                uptime.entry(String::from("boot")).or_insert_with(Utc::now);
            }
            None => error!("Unable to insert boot time into client data."),
        };
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BOT_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn main() {
    kankyo::init().expect("Failed to load .env file");
    env_logger::init();

    let conf: Ini;

    if let Some(project_dirs) = util::get_project_dirs() {
        let config_path = project_dirs.config_dir().join("settings.ini");
        if !config_path.exists() {
            match fs::create_dir_all(match &config_path.parent() {
                Some(pth) => pth,
                None => panic!("Failed to get parent directory"),
            }) {
                Ok(_) => match fs::File::create(&config_path) {
                    Ok(_) => panic!(
                        "Settings have not been configured. {}",
                        &config_path.to_string_lossy()
                    ),
                    Err(e) => panic!("Failed to create settings file: {}", e),
                },
                Err(e) => panic!("Failed to create settings directory: {}", e),
            }
        }
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
            .configure(|c| {
                c.dynamic_prefix(|_, msg| {
                    if msg.is_private() {
                        return Some("".to_owned());
                    }
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
                }).on_mention(true)
            }).on_dispatch_error(|_ctx, _msg, _error| {})
            .customised_help(help_commands::with_embeds, |c| {
                c.lacking_permissions(HelpBehaviour::Hide)
            }).group("Fun", |g| {
                g.command("eightball", |c| {
                    c.cmd(commands::fun::eightball)
                        .desc("Ask the magic eight ball your question and receive your fortune.")
                        .known_as("8ball")
                        .min_args(1)
                })
            }).group("General", |g| {
                g.command("about", |c| c.cmd(commands::general::about))
                    .command("avatar", |c| c.cmd(commands::general::avatar).desc("Shows your current avatar or the avatar of the person mentioned."))
            }).group("Admin", |g| {
                g.command("setprefix", |c| {
                    c.cmd(commands::admin::setprefix)
                        .check(commands::checks::admin_check)
                        .guild_only(true)
                        .desc("Sets the command prefix for this guild.")
                        .min_args(1)
                })
            }).group("Owner", |g| {
                g.command("info", |c| {
                    c.cmd(commands::owner::info)
                        .check(commands::checks::owner_check)
                        .desc(
                            "Information about the currently running bot service and connections.",
                        )
                })
            }),
    );

    {
        let mut data = client.data.lock();
        data.insert::<util::Config>(Arc::clone(&Arc::new(conf)));
        data.insert::<util::Uptime>(HashMap::default());
    }

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
