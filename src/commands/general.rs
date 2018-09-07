use serenity::model::permissions::Permissions;
use serenity::utils::Colour;
use serenity::CACHE;

command!(about(_context, msg, _args) {
  let (invite_url, face) = {
    let cache = CACHE.read();
    match cache.user.invite_url(Permissions::READ_MESSAGES | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS | Permissions::ADD_REACTIONS | Permissions::READ_MESSAGE_HISTORY | Permissions::USE_EXTERNAL_EMOJIS | Permissions::CONNECT | Permissions::USE_VAD | Permissions::CHANGE_NICKNAME) {
      Ok(s) => (s, cache.user.face()),
      Err(why) => {
        println!("Failed to get invite url: {:?}", why);
        return Err(From::from(why));
      }
    }};
  log_error!(msg.channel_id.send_message(|m| m
      .embed(|e| e
        .url(&invite_url)
        .colour(Colour::new(0xD25_148))
        .description("A battle maid for the Great Tomb of Nazarick")
        .title(&crate::BOT_NAME)
        .author(|mut a| {
          a = a.name(&crate::BOT_NAME);
          // Bot avatar URL
          a = a.icon_url(&face);
          a
        })
        .field("Authors", &crate::AUTHORS, false)
        .field("Source Code", "https://github.com/flat/lupusregina-", false)
        )
  ));

});

command!(avatar(_context, msg, args) {
    let face = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.author.face()
        } else {
            let result: Result<String, Box<std::error::Error>> = try {
                msg.guild_id.ok_or("Failed to get GuildId from Message")?
                .to_guild_cached().ok_or("Failed to get Guild from GuildId")?
                .read().members_starting_with(args.full(), false, true).first().ok_or("Could not find member")?
                .user_id().to_user()?.face()
            };
            match result {
                Ok(face) => face,
                Err(e) => {
                    error!("While searching for user: {}", e);
                    msg.author.face()
                }
            }

        }
    } else {
        msg.mentions[0].face()
    };
    log_error!(msg.channel_id.send_message(|m| m
      .embed(|e| e
        .image(face)
        )
  ));

});
