use db;

command!(setprefix(_context, msg, args) {
    match args.single::<String>() {
        Ok(arg) => { match msg.guild_id {
            Some(guild_id) => {match db::set_guild_prefix(guild_id, &arg) {
                Ok(_) => { log_error!(msg.channel_id.say("Set prefix!")); },
                Err(e) => { log_error!(msg.channel_id.say(format!("Failed to set prefix: {}", e))); }
            }},
            None => { log_error!(msg.channel_id.say("Invalid channel")); }
        }},
        Err(e) => { log_error!(msg.channel_id.say(format!("Invalid prefix {}", e))); }
    }
});
