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

use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};

use serenity::model::channel::Message;

use crate::db;

#[command]
#[description = "Sets the prefix for the current Guild."]
#[owner_privilege]
#[required_permissions("ADMINISTRATOR")]
fn setprefix(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg = args.single::<String>().map_err(|_| "Arg.single was None")?;
    let guild_id = msg.guild_id.ok_or_else(|| "guild_id was None")?;
    db::set_guild_prefix(guild_id, arg)
        .and_then(|_| {
            msg.channel_id
                .say(context, "Set prefix!")
                .map_err(|e| e.into())
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
