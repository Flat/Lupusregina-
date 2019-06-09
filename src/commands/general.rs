use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
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
    msg.channel_id
        .send_message(&context, |m| {
            m.embed(|e| {
                e.url(&invite_url)
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
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
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
    msg.channel_id
        .send_message(&context, |m| m.embed(|e| e.image(face)))
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows various information about a user"]
#[only_in("guilds")]
fn userinfo(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("Failed to get GuildID from Message.")?;
    let member = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.member(&context).ok_or("Could not find member.")?
        } else {
            (*(guild_id
                .to_guild_cached(&context)
                .ok_or("Failed to get Guild from GuildId")?
                .read()
                .members_starting_with(args.rest(), false, true)
                .first()
                .ok_or("Could not find member")?))
            .clone()
        }
    } else {
        guild_id.member(
            &context,
            msg.mentions
                .first()
                .ok_or("Failed to get user mentioned.")?,
        )?
    };

    let user = member.user.read();
    let nickname = member.nick.map_or("None".to_owned(), |nick| nick.clone());
    let member_joined = member
        .joined_at
        .map_or("Unavailable".to_owned(), |d| format!("{}", d));

    msg.channel_id
        .send_message(&context, move |m| {
            m.embed(move |e| {
                e.author(|a| a.name(&user.name).icon_url(&user.face()))
                    .field("Discriminator", format!("#{:04}", user.discriminator), true)
                    .field("User ID", user.id, true)
                    .field("Nickname", nickname, true)
                    .field("User Created", user.created_at(), true)
                    .field("Joined Server", member_joined, true)
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
