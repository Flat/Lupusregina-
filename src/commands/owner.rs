use chrono::Utc;
use crate::util;
use serenity::utils::Colour;
use serenity::CACHE;

command!(info(context, msg, _args) {

    let uptime = {
        let data = context.data.lock();
        match data.get::<util::Uptime>() {
        Some(time) => {
            if let Some(boottime) = time.get("boot") {
                let now = Utc::now();
                let duration = now.signed_duration_since(*boottime);
                // Transform duration into days, hours, minutes, seconds.
                // There's probably a cleaner way to do this.
                let mut seconds = duration.num_seconds();
                let mut minutes = seconds / 60;
                seconds %= 60;
                let mut hours = minutes / 60;
                minutes %= 60;
                let days = hours / 24;
                hours %= 24;
                format!("{}d{}h{}m{}s", days, hours, minutes, seconds)
            } else {
                "Uptime not available".to_owned()
            }
             },
        None => "Uptime not available.".to_owned()
        }
    };


    let (name, face, guilds, channels) = {
        let cache = CACHE.read();
        (cache.user.name.to_owned(), cache.user.face(), cache.guilds.len().to_string(), cache.private_channels.len().to_string())
    };

    let _ = msg.channel_id.send_message(|m| m
      .embed(|e| e
        .colour(Colour::FABLED_PINK)
        .description(&format!("Currently running {} - {}", &crate::BOT_NAME, &crate::VERSION))
        .title("Running Information")
        .author(|mut a| {
          a = a.name(&name);
          a = a.icon_url(&face);
          a
        })
        .field("Uptime", &uptime, false)
        .field("Guilds", guilds, false)
        .field("Private Channels", channels, false)
        )
      );

});
