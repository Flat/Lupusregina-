use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::permissions::Permissions;
use serenity::prelude::Context;
use serenity::utils::Colour;

#[command]
#[description = "Shows information about the bot."]
fn about(context: &mut Context, msg: &Message) -> CommandResult {
    let (invite_url, face) = {
        let face = context.cache.read().user.face();
        match context.cache.read().user.invite_url(
            &context,
            Permissions::READ_MESSAGES
                | Permissions::SEND_MESSAGES
                | Permissions::EMBED_LINKS
                | Permissions::ADD_REACTIONS
                | Permissions::READ_MESSAGE_HISTORY
                | Permissions::USE_EXTERNAL_EMOJIS
                | Permissions::CONNECT
                | Permissions::USE_VAD
                | Permissions::CHANGE_NICKNAME,
        ) {
            Ok(s) => (s, face),
            Err(why) => {
                error!("Failed to get invite url: {:?}", why);
                return Err(From::from(why));
            }
        }
    };
    log_error!(msg.channel_id.send_message(&context, |m| m.embed(|e| e
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
        .field("Source Code", "https://github.com/flat/lupusregina-", false))));
    Ok(())
}
#[command]
#[description = "Shows the avatar for the user or specified user."]
fn avatar(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let face = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.author.face()
        } else {
            let result: Result<String, Box<dyn std::error::Error>> = try {
                msg.guild_id
                    .ok_or("Failed to get GuildId from Message")?
                    .to_guild_cached(&context)
                    .ok_or("Failed to get Guild from GuildId")?
                    .read()
                    .members_starting_with(args.rest(), false, true)
                    .first()
                    .ok_or("Could not find member")?
                    .user_id()
                    .to_user(&context)?
                    .face()
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
    log_error!(msg
        .channel_id
        .send_message(&context, |m| m.embed(|e| e.image(face))));
    Ok(())
}
