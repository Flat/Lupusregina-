use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};

use serenity::model::channel::Message;

use crate::db;

#[command]
#[description = "Sets the prefix for the current Guild."]
#[required_permissions("ADMINISTRATOR")]
fn setprefix(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single::<String>() {
        Ok(arg) => match msg.guild_id {
            Some(guild_id) => match db::set_guild_prefix(guild_id, &arg) {
                Ok(_) => {
                    log_error!(msg.channel_id.say(context, "Set prefix!"));
                    Ok(())
                }
                Err(e) => {
                    log_error!(msg
                        .channel_id
                        .say(context, format!("Failed to set prefix: {}", e)));
                    Err(CommandError(e.to_string()))
                }
            },
            None => {
                log_error!(msg.channel_id.say(context, "Invalid channel"));
                Err(CommandError("Invalid Channel".into()))
            }
        },
        Err(e) => {
            log_error!(msg.channel_id.say(context, format!("Invalid prefix {}", e)));
            Err(CommandError(e.to_string()))
        }
    }
}
