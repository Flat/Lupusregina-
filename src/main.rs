/*
 * Copyright 2019 Kenneth Swenson
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

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
use serenity::model::prelude::{GuildId, Message};
use serenity::prelude::*;

use crate::commands::{admin::*, fun::*, general::*, moderation::*, owner::*, weeb::*};
use crate::util::get_configuration;
use serenity::framework::standard::DispatchError::Ratelimited;

pub mod commands;
pub mod db;
pub mod util;

struct Handler;

impl EventHandler for Handler {
    fn cache_ready(&self, _ctx: Context, guilds: Vec<GuildId>) {
        info!("Connected to {} guilds.", guilds.len());
    }

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

#[group]
#[commands(about, avatar, userinfo, guildinfo, ping)]
struct General;

#[group]
#[commands(eightball, darksouls, darksouls3, ddate)]
struct Fun;

#[group]
#[commands(setprefix)]
struct Admin;

#[group]
#[owners_only]
#[owner_privilege]
#[commands(info, reload, nickname, rename, setavatar)]
#[sub_groups(Presence)]
struct Owner;

#[group]
#[prefix("presence")]
#[owners_only]
#[commands(online, idle, dnd, invisible, reset, set)]
struct Presence;

#[group]
#[commands(ban, unban, setslowmode)]
struct Moderation;

#[group]
#[commands(anime, manga)]
struct Weeb;

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
    kankyo::load(true)?;
    env_logger::init();

    let token = env::var("BOT_TOKEN")?;

    let conf = get_configuration()?;

    db::create_db();

    let mut client = Client::new(&token, Handler)?;

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
                    error!(
                        "Command {:?} triggered by {}: {:?}",
                        command,
                        message.author.tag(),
                        e
                    );
                    message
                        .react(context, '\u{274C}')
                        .map_or_else(|_| (), |_| ());
                }
            })
            .on_dispatch_error(|context, message, error| match error {
                Ratelimited(e) => {
                    error!("{} failed: {:?}", message.content, error);
                    let _ = message.channel_id.say(
                        &context,
                        format!("Ratelimited: Try again in {} seconds.", e),
                    );
                }
                _ => error!("{} failed: {:?}", message.content, error),
            })
            .bucket("anilist", |b| b.time_span(60).limit(90))
            .help(&MY_HELP)
            .group(&GENERAL_GROUP)
            .group(&FUN_GROUP)
            .group(&ADMIN_GROUP)
            .group(&OWNER_GROUP)
            .group(&MODERATION_GROUP)
            .group(&WEEB_GROUP),
    );

    {
        let mut data = client.data.write();
        data.insert::<util::Config>(Arc::clone(&Arc::new(conf)));
        data.insert::<util::Uptime>(HashMap::default());
        data.insert::<util::ClientShardManager>(Arc::clone(&client.shard_manager));
    }

    client.start_autosharded().map_err(|e| e.into())
}
