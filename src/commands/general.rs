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


use chrono::Utc;
use poise::serenity_prelude::{Colour, Permissions, ShardId, User};
use crate::{Context, Error};


#[poise::command(slash_command, description_localized("en", "Shows information about the bot."))]
pub async fn about(context: Context<'_>) -> Result<(), Error> {
    let current_user = context.discord()
        .cache
        .current_user().clone();
    let face = current_user.face();
    let invite_url = current_user.invite_url(&context.discord(), Permissions::SEND_MESSAGES
        | Permissions::EMBED_LINKS
        | Permissions::ADD_REACTIONS
        | Permissions::USE_EXTERNAL_EMOJIS
        | Permissions::CHANGE_NICKNAME)
        .await?;
    context
        .send( |m| {
            m.embed(|e| {
                e.url(&invite_url)
                    .colour(Colour::new(0x00D2_5148))
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
            }).ephemeral(true)
        })
        .await?;
    Ok(())
}

#[poise::command(context_menu_command = "User information", guild_only)]
pub async fn userinfo(context: Context<'_>, #[description = "The user to show information about."] user: User) -> Result<(), Error> {
    let guild_id = context.guild_id().ok_or("Failed to get GuildID from Message.")?;
    let member =
        guild_id
            .member(
                context.discord(),
                user.id
            )
            .await?;

    let nickname = member.nick.map_or("None".to_owned(), |nick| nick);
    let member_joined = member
        .joined_at
        .map_or("Unavailable".to_owned(), |d| format!("{}", d));

    context.send(move |m| {
            m.embed(move |e| {
                e.author(|a| a.name(&user.name).icon_url(&user.face()))
                    .field("Discriminator", format!("#{:04}", user.discriminator), true)
                    .field("User ID", user.id, true)
                    .field("Nickname", nickname, true)
                    .field("User Created", user.created_at(), true)
                    .field("Joined Server", member_joined, true)
            }).ephemeral(true)
        })
        .await?;
    Ok(())
}

#[poise::command(slash_command, guild_only, description_localized("en", "Shows various information about the guild."))]
pub async fn guildinfo(context: Context<'_>) -> Result<(), Error> {
    let guild_id = context.guild_id().ok_or("Failed to get GuildID from Message.")?;
    let guild = guild_id
        .to_guild_cached(&context.discord())
        .ok_or("Failed to get Guild from GuildID")?;

    context.send(move |m| {
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
            }).ephemeral(true)
        })
        .await?;
    Ok(())
}

#[poise::command(slash_command, description_localized("en", "Responds with the current latency to Discord."))]
pub async fn ping(context: Context<'_>) -> Result<(), Error> {
    let now = Utc::now();
    let msg = context.say("Ping!").await?;
    let finish = Utc::now();
    let lping = ((finish.timestamp() - now.timestamp()) * 1000)
        + (i64::from(finish.timestamp_subsec_millis()) - i64::from(now.timestamp_subsec_millis()));
    let shard_manager = context
        .data().shard_manager.clone();
    let shard_latency = shard_manager
        .lock()
        .await
        .runners
        .lock()
        .await
        .get(&ShardId(context.discord().shard_id))
        .ok_or("Failed to get Shard.")?
        .latency
        .ok_or("Failed to get latency from shard.")?
        .as_millis();
    msg.edit(context, |m| {
        m.content(&format!(
            "Rest API: {}ms\nShard Latency: {}ms",
            lping, shard_latency
        ))
    })
    .await?;
    Ok(())
}
