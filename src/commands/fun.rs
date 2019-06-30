/*
 * Copyright 2019 Kenneth Swenson
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use rand::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

#[command]
#[description = "Ask the magic eight ball your question and receive your fortune."]
#[min_args(1)]
#[aliases("8ball")]
fn eightball(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let answers = vec![
        "It is certain.",
        "It is decidedly so.",
        "Without a doubt.",
        "Yes- definitely.",
        "You may rely on it.",
        "As I see it, yes",
        "Most likely",
        "Outlook good.",
        "Yes.",
        "Signs point to yes.",
        "Reply hazy, try again.",
        "Ask again later.",
        "Better not tell you now",
        "Cannot predict now.",
        "Concentrate and ask again.",
        "Don't count on it.",
        "My reply is no.",
        "My sources say no.",
        "Outlook not so good.",
        "Very doubtful.",
    ];
    let mut rng = thread_rng();
    let num = rng.gen_range(0, 19);
    let choice = answers[num];
    msg.channel_id
        .send_message(&context, |m| {
            m.embed(|e| {
                e.colour({
                    if num <= 9 {
                        Colour::new(0x28_A7_45)
                    } else if num <= 14 {
                        Colour::new(0xFF_C1_07)
                    } else {
                        Colour::new(0xDC_35_45)
                    }
                })
                .description(args.rest())
                .author(|mut a| {
                    if msg.is_private() {
                        a = a.name(&msg.author.name);
                    } else if let Some(nick) = msg.guild_id.and_then(|guild_id| {
                        context
                            .cache
                            .read()
                            .member(guild_id, msg.author.id)
                            .and_then(|member| member.nick)
                    }) {
                        a = a.name(nick);
                    } else {
                        a = a.name(&msg.author.name);
                    }
                    a = a.icon_url(&msg.author.face());
                    a
                })
                .field("🎱Eightball🎱", choice, false)
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
