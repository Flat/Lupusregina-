use chrono::Utc;
use serenity::utils::Colour;
use serenity::CACHE;
use util;

command!(info(context, msg, _args) {
   // Get startup time from context.data
  let data = context.data.lock();
  let uptime = data.get::<util::Uptime>().unwrap();


  if let Some(boottime) = uptime.get("boot") {
    let now = Utc::now();
    let duration = now.signed_duration_since(boottime.to_owned());
    // Transform duration into days, hours, minutes, seconds.
    // There's probably a cleaner way to do this.
    let mut seconds = duration.num_seconds();
    let mut minutes = seconds / 60;
    seconds %= 60;
    let mut hours = minutes / 60;
    minutes %= 60;
    let days = hours / 24;
    hours %= 24;

    let (name, face, guilds, channels) = {
        let cache = CACHE.read();
        (cache.user.name.to_owned(), cache.user.face(), cache.guilds.len().to_string(), cache.private_channels.len().to_string())
    };

    let _ = msg.channel_id.send_message(|m| m
      .embed(|e| e
        .colour(Colour::FABLED_PINK)
        .description(&format!("Currently running {} - {}", &::BOT_NAME, &::VERSION))
        .title("Running Information")
        .author(|mut a| {
          a = a.name(&name);
          a = a.icon_url(&face);
          a
        })
        .field("Uptime", &format!("{}d{}h{}m{}s", days, hours , minutes, seconds), false)
        .field("Guilds", guilds, false)
        .field("Private Channels", channels, false)
        )
      );
  }
  // If we can't read the context.data give up
  else {
    let _ = msg.channel_id.say("Unable to get startup time");
  }

});
