use rand::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult, macros::command};
use serenity::model::channel::Message;
use serenity::utils::Colour;

#[command]
#[description = "Ask the magic eight ball your question and receive your fortune."]
#[aliases("8ball")]
fn eightball(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let answers = vec!["It is certain.", "It is decidedly so.", "Without a doubt.",
         "Yes- definitely.", "You may rely on it.", "As I see it, yes", "Most likely", "Outlook good.",
         "Yes.", "Signs point to yes.", "Reply hazy, try again.", "Ask again later.",
         "Better not tell you now", "Cannot predict now.", "Concentrate and ask again.",
         "Don't count on it.", "My reply is no.", "My sources say no.", "Outlook not so good.",
         "Very doubtful."];
    let mut rng = thread_rng();
    let num = rng.gen_range(0,19);
    let choice = answers[num];
    log_error!(msg.channel_id.send_message(context, |m| m
    .embed(|e| e
        .colour({ if num <= 9 {
            Colour::new(0x28A_745)
        } else if num <= 14 {
            Colour::new(0xFFC_107)
        } else {
            Colour::new(0xDC3_545)
        } })
        .description(args.rest())
        .author(|mut a| {
          a = a.name(&msg.author.name);
          // Bot avatar URL
          a = a.icon_url(&msg.author.face());
          a
        })
        .field("🎱Eightball🎱", choice, false)
        )));
    Ok(())
}
