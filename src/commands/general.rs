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

use crate::util::ClientShardManager;
use chrono::Utc;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::model::permissions::Permissions;
use serenity::prelude::Context;
use serenity::utils::Colour;

#[command]
#[description = "Shows information about the bot."]
async fn about(context: &mut Context, msg: &Message) -> CommandResult {
    let (invite_url, face) = {
        let face = context.cache.read().await.user.face();
        match context
            .cache
            .read()
            .await
            .user
            .invite_url(
                &context,
                Permissions::READ_MESSAGES
                    | Permissions::SEND_MESSAGES
                    | Permissions::EMBED_LINKS
                    | Permissions::ADD_REACTIONS
                    | Permissions::READ_MESSAGE_HISTORY
                    | Permissions::USE_EXTERNAL_EMOJIS
                    | Permissions::CONNECT
                    | Permissions::USE_VAD
                    | Permissions::CHANGE_NICKNAME,
            )
            .await
        {
            Ok(s) => (s, face),
            Err(why) => {
                error!("Failed to get invite url: {:?}", why);
                return Err(From::from(why));
            }
        }
    };
    msg.channel_id
        .send_message(&context, |m| {
            m.embed(|e| {
                e.url(&invite_url)
                    .colour(Colour::new(0xD25_148))
                    .description("A battle maid for the Great Tomb of Nazarick")
                    .title(&crate::BOT_NAME)
                    .author(|mut a| {
                        a = a.name(&crate::BOT_NAME);
                        // Bot avatar URL
                        a = a.icon_url(&face);
                        a
                    })
                    .field("Authors", &crate::AUTHORS, false)
                    .field("Source Code", "https://github.com/flat/lupusregina-", false)
            })
        })
        .await
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows the avatar for the user or specified user."]
async fn avatar(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let face = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.author.face()
        } else {
            let result: Result<String, Box<dyn std::error::Error>> = try {
                msg.guild_id
                    .ok_or("Failed to get GuildId from Message")?
                    .to_guild_cached(&context)
                    .await
                    .ok_or("Failed to get Guild from GuildId")?
                    .read()
                    .await
                    .members_starting_with(args.rest(), false, true)
                    .await
                    .first()
                    .ok_or("Could not find member")?
                    .0
                    .user_id()
                    .await
                    .to_user(&context)
                    .await?
                    .face()
            };
            match result {
                Ok(face) => face,
                Err(e) => {
                    error!("While searching for user: {}", e);
                    msg.author.face()
                }
            }
        }
    } else {
        msg.mentions[0].face()
    };
    msg.channel_id
        .send_message(&context, |m| m.embed(|e| e.image(face)))
        .await
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows various information about a user"]
#[only_in("guilds")]
async fn userinfo(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("Failed to get GuildID from Message.")?;
    let member = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.member(&context).await.ok_or("Could not find member.")?
        } else {
            (*(guild_id
                .to_guild_cached(&context)
                .await
                .ok_or("Failed to get Guild from GuildId")?
                .read()
                .await
                .members_starting_with(args.rest(), false, true)
                .await
                .first()
                .ok_or("Could not find member")?))
            .0
            .clone()
        }
    } else {
        guild_id
            .member(
                &context,
                msg.mentions
                    .first()
                    .ok_or("Failed to get user mentioned.")?,
            )
            .await?
    };

    let user = member.user.read().await;
    let nickname = member.nick.map_or("None".to_owned(), |nick| nick);
    let member_joined = member
        .joined_at
        .map_or("Unavailable".to_owned(), |d| format!("{}", d));

    msg.channel_id
        .send_message(&context, move |m| {
            m.embed(move |e| {
                e.author(|a| a.name(&user.name).icon_url(&user.face()))
                    .field("Discriminator", format!("#{:04}", user.discriminator), true)
                    .field("User ID", user.id, true)
                    .field("Nickname", nickname, true)
                    .field("User Created", user.created_at(), true)
                    .field("Joined Server", member_joined, true)
            })
        })
        .await
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows various information about the guild."]
#[only_in("guilds")]
async fn guildinfo(context: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| "Failed to get GuildID from Message.")?;
    let guild = guild_id
        .to_guild_cached(&context)
        .await
        .ok_or("Failed to get Guild from GuildID")?
        .read()
        .await
        .clone();

    msg.channel_id
        .send_message(&context, move |m| {
            m.embed(move |e| {
                e.author(|a| {
                    a.name(&guild.name);
                    if let Some(guild_icon) = &guild.icon_url() {
                        a.icon_url(guild_icon);
                    }
                    a
                })
                .field("Guild ID", format!("{}", guild_id), true)
                .field("Members", guild.member_count, true)
                .field("Region", &guild.region, true)
                .field("Features", format!("{:?}", guild.features), true)
                .field(
                    "Nitro Boost Level",
                    format!("{:?}", guild.premium_tier),
                    true,
                )
                .field("Nitro Boosts", guild.premium_subscription_count, true);
                if let Some(splash) = guild.splash_url() {
                    e.image(splash);
                }
                e.footer(|f| f.text(format!("Guild created at {}", guild_id.created_at())))
            })
        })
        .await
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Responds with the current latency to Discord."]
async fn ping(context: &mut Context, msg: &Message) -> CommandResult {
    try {
        let now = Utc::now();
        let mut msg = msg.channel_id.say(&context, "Ping!").await?;
        let finish = Utc::now();
        let lping = ((finish.timestamp() - now.timestamp()) * 1000)
            + (i64::from(finish.timestamp_subsec_millis())
                - i64::from(now.timestamp_subsec_millis()));
        let shard_manager = context
            .data
            .read()
            .await
            .get::<ClientShardManager>()
            .ok_or_else(|| "Failed to get ClientShardManager.")?
            .clone();
        let shard_latency = shard_manager
            .lock()
            .await
            .runners
            .lock()
            .await
            .get(&ShardId(context.shard_id))
            .ok_or_else(|| "Failed to get Shard.")?
            .latency
            .ok_or_else(|| "Failed to get latency from shard.")?
            .as_millis();
        msg.edit(&context, |m| {
            m.content(&format!(
                "Rest API: {}ms\nShard Latency: {}ms",
                lping, shard_latency
            ))
        })
        .await?
    }
}
