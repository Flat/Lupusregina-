#![feature(try_blocks)]
#![feature(result_map_or_else)]
extern crate env_logger;
#[macro_use]
extern crate log;

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;

use chrono::Utc;
use serenity::framework::standard::{
    help_commands,
    macros::{group, help},
    Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::model::prelude::Message;
use serenity::prelude::*;

use crate::commands::{admin::*, fun::*, general::*, moderation::*, owner::*};
use crate::util::get_configuration;

pub mod commands;
pub mod db;
pub mod util;

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
    commands: [info, reload, ping]
});

group!({
   name: "Presence",
   options: {
        prefixes: ["presence"],
        owners_only: true
   },
   commands: [online, idle, dnd, invisible, reset, set]
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
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    kankyo::init().expect("Failed to load .env file");

    env_logger::init();

    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");

    let conf = get_configuration()?;

    db::create_db();

    let mut client = Client::new(&token, Handler).expect("Error creating client");

    let (owner, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application information: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.dynamic_prefix(|_, msg| {
                    if msg.is_private() {
                        return Some("".into());
                    }
                    if let Some(guild_id) = msg.guild_id {
                        let prefix =
                            db::get_guild_prefix(guild_id).map_or_else(|_| ".".into(), |pref| pref);
                        Some(prefix)
                    } else {
                        Some(".".into())
                    }
                })
                .on_mention(Some(bot_id))
                .owners(owner)
            })
            .after(|context, message, command, result| match result {
                Ok(()) => message
                    .react(context, '\u{2705}')
                    .map_or_else(|_| (), |_| ()),
                Err(e) => {
                    error!("{:?}: {:?}", command, e);
                    message
                        .react(context, '\u{274C}')
                        .map_or_else(|_| (), |_| ());
                }
            })
            .on_dispatch_error(|_, message, error| {
                error!("{} failed: {:?}", message.content, error);
            })
            .help(&MY_HELP_HELP_COMMAND)
            .group(&GENERAL_GROUP)
            .group(&FUN_GROUP)
            .group(&ADMIN_GROUP)
            .group(&OWNER_GROUP)
            .group(&PRESENCE_GROUP)
            .group(&MODERATION_GROUP),
    );

    {
        let mut data = client.data.write();
        data.insert::<util::Config>(Arc::clone(&Arc::new(conf)));
        data.insert::<util::Uptime>(HashMap::default());
    }

    client.start_autosharded().map_err(|e| e.into())
}
