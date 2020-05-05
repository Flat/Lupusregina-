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
#![feature(async_closure)]
extern crate env_logger;
#[macro_use]
extern crate log;

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;

use chrono::Utc;
use serenity::async_trait;
use serenity::framework::standard::{
    help_commands,
    macros::{group, help, hook},
    Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
};
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::model::prelude::{GuildId, Message};
use serenity::prelude::*;

use crate::commands::{admin::*, fun::*, general::*, moderation::*, owner::*, weeb::*};
use crate::util::{get_configuration, Prefixes};
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::DispatchError::Ratelimited;
use serenity::http::Http;

pub mod commands;
pub mod db;
pub mod util;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        info!("Connected to {} guilds.", guilds.len());
        let shard_messenger = ctx.shard.lock().await;
        shard_messenger.chunk_guilds(guilds, None, None);
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            info!(
                "Connected as {} on shard {}/{}",
                ready.user.name,
                shard[0] + 1,
                shard[1]
            );
        } else {
            info!("Connected as {}", ready.user.name);
        }

        let data = ctx.data.write();
        match data.await.get_mut::<util::Uptime>() {
            Some(uptime) => {
                uptime.entry(String::from("boot")).or_insert_with(Utc::now);
            }
            None => error!("Unable to insert boot time into client data."),
        };
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
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
#[commands(bloodborne, darksouls, darksouls3, ddate, eightball)]
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
#[commands(anime, manga, vtuber)]
struct Weeb;

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => msg.react(ctx, '\u{2705}').await.map_or_else(|_| (), |_| ()),
        Err(e) => {
            error!(
                "Command {:?} triggered by {}: {:?}",
                command_name,
                msg.author.tag(),
                e
            );
            msg.react(ctx, '\u{274C}').await.map_or_else(|_| (), |_| ());
        }
    }
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) -> () {
    match error {
        Ratelimited(e) => {
            error!("{} failed: {:?}", msg.content, error);
            let _ = msg
                .channel_id
                .say(&ctx, format!("Ratelimited: Try again in {} seconds.", e));
        }
        _ => error!("{} failed: {:?}", msg.content, error),
    }
}

#[hook]
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    if msg.is_private() {
        return Some("".into());
    }
    if let Some(guild_id) = msg.guild_id {
        let prefixes = async { ctx.data.read().await.get::<Prefixes>().cloned() }.await;
        let prefix = prefixes.map_or_else(
            || ".".into(),
            |pref| {
                pref.get(guild_id.as_u64())
                    .map_or_else(|| ".".into(), |prefix| prefix.into())
            },
        );
        Some(prefix)
    } else {
        Some(".".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    kankyo::load(true)?;
    env_logger::init();

    let token = env::var("BOT_TOKEN")?;

    let conf = get_configuration()?;

    db::create_db();

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(dynamic_prefix)
                .on_mention(Some(bot_id))
                .owners(owners)
        })
        .after(after)
        .on_dispatch_error(dispatch_error)
        .bucket("anilist", |b| b.time_span(60).limit(90))
        .await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&FUN_GROUP)
        .group(&ADMIN_GROUP)
        .group(&OWNER_GROUP)
        .group(&MODERATION_GROUP)
        .group(&WEEB_GROUP);

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .add_intent(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_PRESENCES,
        )
        .await
        .expect("Error creating client!");

    let prefixes = db::get_all_prefixes().unwrap_or_else(|_| HashMap::<u64, String>::new());
    {
        let mut data = client.data.write().await;
        data.insert::<util::Config>(Arc::clone(&Arc::new(conf)));
        data.insert::<util::ClientShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<util::Uptime>(HashMap::default());
        data.insert::<util::Prefixes>(prefixes);
    }

    client.start_autosharded().await.map_err(|e| e.into())
}
