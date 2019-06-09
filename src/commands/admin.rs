use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};

use serenity::model::channel::Message;

use crate::db;

#[command]
#[description = "Sets the prefix for the current Guild."]
#[required_permissions("ADMINISTRATOR")]
fn setprefix(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg = args
        .single::<String>()
        .map_err(|_| CommandError("Arg.single was None".into()))?;
    let guild_id = msg
        .guild_id
        .ok_or_else(|| CommandError("guild_id was None".into()))?;
    db::set_guild_prefix(guild_id, arg)
        .and_then(|_| {
            msg.channel_id
                .say(context, "Set prefix!")
                .map_err(|e| e.into())
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
