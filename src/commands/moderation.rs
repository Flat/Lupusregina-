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

use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::Context;

#[command]
#[only_in("guilds")]
#[required_permissions("BAN_MEMBERS")]
async fn ban(context: &Context, msg: &Message) -> CommandResult {
    if !msg.mentions.is_empty() {
        try {
            msg.guild_id
                .ok_or("Failed to get GuildId from Message")?
                .to_guild_cached(&context)
                .await
                .ok_or("Failed to get Guild from GuildId")?
                .read()
                .await
                .member(context, msg.mentions[0].id)
                .await?
                .ban(context, 0u8)
                .await?
        }
    } else {
        Err(serenity::framework::standard::CommandError(String::from(
            "No mentioned target.",
        )))
    }
}

#[command]
#[min_args(1)]
#[only_in("guilds")]
#[required_permissions("BAN_MEMBERS")]
async fn unban(context: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg
        .guild_id
        .ok_or("Failed to get GuildId from Message")?
        .to_guild_cached(&context)
        .await
        .ok_or("Failed to get Guild from GuildId")?
        .read()
        .await
        .clone();
    let bans = guild.bans(context).await?;

    for banned in bans {
        if banned.user.tag() == args.rest() {
            guild.unban(context, banned.user.id).await?;
        }
    }
    Ok(())
}

#[command]
#[min_args(1)]
#[only_in("guilds")]
#[required_permissions("MANAGE_CHANNELS")]
async fn setslowmode(context: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let seconds = &args.single::<u64>()?;
    msg.channel_id
        .edit(context, |c| c.slow_mode_rate(*seconds))
        .await
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
