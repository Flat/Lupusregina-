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

use std::env;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use dotenv::dotenv;
use poise::{Event, serenity_prelude as serenity};
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing::log::trace;

use util::Data;

use crate::util::get_configuration;

pub mod commands;
pub mod util;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BOT_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[tokio::main]
async fn main() -> Result<(), poise::serenity_prelude::Error> {
    dotenv().expect("Failed to load .env file!");

    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![
            register(),
            commands::general::ping(),
            commands::general::about(),
            commands::general::guildinfo(),
            commands::general::userinfo(),
            commands::owner::info(),
            commands::owner::nickname(),
            commands::owner::presence(),
            commands::owner::reload(),
            commands::owner::rename(),
            commands::owner::setavatar(),
            commands::weeb::anime(),
            commands::weeb::manga(),
            commands::weeb::vtuber(),
            commands::fun::bloodborne(),
            commands::fun::darksouls(),
            commands::fun::darksouls3(),
            commands::fun::eightball(),
            commands::fun::ddate()

        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        /// The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        /// This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                trace!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                trace!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        listener: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                match event {
                    Event::Ready { data_about_bot } => {
                        let mut shards = String::new();
                        if let Some(shard) = data_about_bot.shard {
                            shards = format!(" on shard {}/{}", shard[0], shard[1]);
                        }
                        info!("Connected as {:?}{}", data_about_bot.user, shards)
                    },
                    _ => {}
                }
                Ok(())
            })
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(
            env::var("BOT_TOKEN")
                .expect("Missing `BOT_TOKEN` env var."),
        )
        .user_data_setup(move |_ctx, _ready, framework| {
            Box::pin(async move {
                Ok(Data {
                    config: Mutex::new(get_configuration()?),
                    uptime: Arc::new(Utc::now()),
                    shard_manager: framework.shard_manager().clone(),
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
}
