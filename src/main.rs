#![feature(try_blocks)]
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::sync::Arc;

use chrono::Utc;
#[macro_use]
extern crate log;
extern crate env_logger;
use ini::Ini;
use serenity::framework::standard::{macros::{group, help}, StandardFramework, HelpOptions, Args, CommandGroup, CommandResult, help_commands};
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::commands::admin::SETPREFIX_COMMAND;
use crate::commands::general::ABOUT_COMMAND;
use crate::commands::general::AVATAR_COMMAND;
use crate::commands::fun::EIGHTBALL_COMMAND;
use crate::commands::owner::INFO_COMMAND;
use crate::commands::owner::RELOAD_COMMAND;
use crate::commands::owner::PING_COMMAND;
use crate::commands::owner::ONLINE_COMMAND;
use crate::commands::owner::IDLE_COMMAND;
use crate::commands::owner::DND_COMMAND;
use crate::commands::owner::INVISIBLE_COMMAND;
use crate::commands::owner::RESET_COMMAND;
use crate::commands::owner::GAME_COMMAND;
use crate::commands::moderation::BAN_COMMAND;
use crate::commands::moderation::UNBAN_COMMAND;
use serenity::model::id::UserId;
use serenity::model::prelude::Message;

#[macro_use]
pub mod util;
pub mod commands;
pub mod db;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        let mut data = ctx.data.write();
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

group!({
    name: "General",
    options: {},
    commands: [about, avatar]
});

group!({
    name: "Fun",
    options: {},
    commands: [eightball]
});

group!({
    name: "Admin",
    options: {},
    commands: [setprefix]
});

group!({
    name: "Owner",
    options: {
        owners_only: true
    },
    commands: [info, reload, game, ping]
});

group!({
   name: "Presence",
   options: {
        prefixes: ["presence"],
        owners_only: true
   },
   commands: [online, idle, dnd, invisible, reset]
});

group!({
    name: "Moderation",
    commands: [ban, unban]
});

#[help]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

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

    let (owner, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        },
        Err(why) => panic!("Could not access application information: {:?}", why),
    };

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
                }).on_mention(Some(bot_id))
                    .owners(owner)
            })
            .help(&MY_HELP_HELP_COMMAND)
            .group(&GENERAL_GROUP)
            .group(&FUN_GROUP)
            .group(&ADMIN_GROUP)
            .group(&OWNER_GROUP)
            .group(&PRESENCE_GROUP)
            .group(&MODERATION_GROUP)
    );

    {
        let mut data = client.data.write();
        data.insert::<util::Config>(Arc::clone(&Arc::new(conf)));
        data.insert::<util::Uptime>(HashMap::default());
    }

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
