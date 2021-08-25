/*
 * Copyright 2020 Kenneth Swenson
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

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;

use chrono::Utc;
use dotenv::dotenv;
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

use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::DispatchError::Ratelimited;
use serenity::http::Http;
use serenity::model::channel::MessageType::ApplicationCommand;
use serenity::model::interactions::Interaction;

use tracing::{error, info};


pub mod commands;
pub mod util;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _ctx: Context, guilds: Vec<GuildId>) {
        info!("Connected to {} guilds.", guilds.len());
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

        let mut data = ctx.data.write();
        match data.await.get_mut::<util::Uptime>() {
            Some(uptime) => {
                uptime.entry(String::from("boot")).or_insert_with(Utc::now);
            }
            None => error!("Unable to insert boot time into client data."),
        };
        let slash_commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("avatar").description("Shows the avatar for the user or specified user.").create_option(|option| {
                    option.name("user").description("The target user").kind(ApplicationCommandType::User).required(false)
                })
            })
        }).await;

        info!("The following global slash commands are registered: {:#?}", slash_commands);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        info!("Received command {:?} from user {:?}", interaction.data.name.as_str(), interaction.user);
        let result = match command.data.name.as_str() {
            "avatar" => crate::commands::general::avatar(&ctx, interaction).await
        };
        match result {
            Ok(_) => (),
            Err(why) => error!("Command failed: {:?}", why)
        }
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BOT_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Failed to load .env file!");
    tracing_subscriber::fmt::init();

    let token = env::var("BOT_TOKEN")?;

    let http = Http::new_with_token(&token);

    match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .intents(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_BANS
                | GatewayIntents::GUILD_PRESENCES
                | GatewayIntents::GUILD_VOICE_STATES,
        )
        .await
        .expect("Error creating client!");

    {
        let mut data = client.data.write().await;
        data.insert::<util::ClientShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<util::Uptime>(HashMap::default());
    }

    client.start_autosharded().await.map_err(|e| e.into())
}
