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

use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};

use serenity::model::channel::Message;

use crate::db;
use crate::util::{DbPool, Prefixes};

#[command]
#[description = "Sets the prefix for the current Guild."]
#[owner_privilege]
#[only_in("guilds")]
#[required_permissions("ADMINISTRATOR")]
async fn setprefix(context: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg = args.single::<String>().map_err(|_| "Arg.single was None")?;
    let guild_id = msg.guild_id.ok_or("guild_id was None")?;
    let pool = context
        .data
        .read()
        .await
        .get::<DbPool>()
        .ok_or("Unable to get DB Pool.")?
        .clone();
    {
        let mut data = context.data.write().await;
        let prefixes = data
            .get_mut::<Prefixes>()
            .ok_or("Unable to get Prefix HashMap from context data.")?;
        prefixes.insert(*guild_id.as_u64(), arg.clone());
    }
    db::set_guild_prefix(&pool, guild_id, arg)
        .await
        .map_err(|e| e.to_string())?;
    msg.channel_id.say(context, "Set prefix!").await?;
    Ok(())
}
