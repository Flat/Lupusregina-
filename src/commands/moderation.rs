use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::Context;

#[command]
#[only_in("guilds")]
#[required_permissions("BAN_MEMBERS")]
fn ban(context: &mut Context, msg: &Message) -> CommandResult {
    if !msg.mentions.is_empty() {
        try {
            msg.guild_id
                .ok_or("Failed to get GuildId from Message")?
                .to_guild_cached(&context)
                .ok_or("Failed to get Guild from GuildId")?
                .read()
                .member(&context, msg.mentions[0].id)?
                .ban(&context, &0)?
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
fn unban(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg
        .guild_id
        .ok_or("Failed to get GuildId from Message")?
        .to_guild_cached(&context)
        .ok_or("Failed to get Guild from GuildId")?
        .read()
        .clone();
    let bans = guild.bans(&context)?;

    for banned in bans {
        if banned.user.tag() == args.rest() {
            guild.unban(&context, banned.user.id)?;
        }
    }
    Ok(())
}

#[command]
#[min_args(1)]
#[only_in("guilds")]
#[required_permissions("MANAGE_CHANNELS")]
fn setslowmode(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let seconds = &args.single::<u64>()?;
    msg.channel_id
        .edit(context, |c| c.slow_mode_rate(*seconds))
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
