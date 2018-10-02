command!(ban(_context, msg, _args) {
    if !msg.mentions.is_empty() {
        let result: Result<(), Box<std::error::Error>> = try {
                msg.guild_id.ok_or("Failed to get GuildId from Message")?
                .to_guild_cached().ok_or("Failed to get Guild from GuildId")?
                .read().member(msg.mentions[0].id)?.ban(&0)?
            };
            match result {
                Ok(()) => (),
                Err(e) => {
                       log_error!(msg.channel_id.say(format!("Failed to ban: {}", e)));
                }
            }
    };
});

command!(unban(_context, msg, args) {
        let result: Result<(), Box<std::error::Error>> = try {
                let guild = msg.guild_id.ok_or("Failed to get GuildId from Message")?
                            .to_guild_cached().ok_or("Failed to get Guild from GuildId")?.read().clone();
                let bans = guild.bans()?;

                for banned in bans {
                    if banned.user.tag() == args.full() {
                        guild.unban(banned.user.id)?
                    }
                }
            };
            match result {
                Ok(()) => (),
                Err(e) => {
                       log_error!(msg.channel_id.say(format!("Failed to ban: {}", e)));
                }
            }
});