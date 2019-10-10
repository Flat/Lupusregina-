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

use core::fmt;

use chrono::{Date, Datelike, Local};
use rand::prelude::*;
use rand::Rng;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

use lazy_static::lazy_static;
use std::cmp::Ordering;

lazy_static! {
    static ref DS1FILLERS: Vec<&'static str> =
        { include_str!("data/ds1fillers.txt").split('\n').collect() };
    static ref DS1TEMPLATES: Vec<&'static str> =
        { include_str!("data/ds1templates.txt").split('\n').collect() };
    static ref DS3TEMPLATES: Vec<&'static str> =
        { include_str!("data/ds3templates.txt").split('\n').collect() };
    static ref DS3FILLERS: Vec<&'static str> =
        { include_str!("data/ds3fillers.txt").split('\n').collect() };
    static ref DS3CONJUNCTIONS: Vec<&'static str> = {
        include_str!("data/ds3conjunctions.txt")
            .split('\n')
            .collect()
    };
    static ref DDAYS: Vec<&'static str> = vec![
        "Sweetmorn",
        "Boomtime",
        "Pungenday",
        "Prickle-Prickle",
        "Setting Orange",
    ];
    static ref DSEASONS: Vec<&'static str> = vec![
        "Chaos",
        "Discord",
        "Confusion",
        "Bureaucracy",
        "The Aftermath",
    ];
}

struct Dday {
    season: u32,
    day: u32,
    year: i32,
    tibs_day: bool,
}

impl From<Date<Local>> for Dday {
    fn from(date: Date<Local>) -> Self {
        let year = date.year() + 1166;
        let mut day_of_year = date.ordinal();
        let mut season = 0;
        let mut tibs_day = false;
        if year % 4 == 2 {
            match day_of_year.cmp(&59) {
                Ordering::Equal => tibs_day = true,
                Ordering::Greater => day_of_year -= 1,
                Ordering::Less => (),
            }
        };
        while day_of_year >= 73 {
            season += 1;
            day_of_year -= 73;
        }
        Dday {
            year,
            season,
            day: day_of_year,
            tibs_day,
        }
    }
}

impl fmt::Display for Dday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tibs_day {
            write!(f, "Today is St. Tib's Day in the YOLD {}", self.year)
        } else {
            write!(
                f,
                "Today is {}, the {}{} day of {} in the YOLD {}",
                DDAYS[(self.day % 5) as usize],
                self.day,
                parse_int_ordinal_suffix(self.day),
                DSEASONS[self.season as usize],
                self.year
            )
        }
    }
}

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

#[command]
#[description = "Display a randomly generated Dark Souls message."]
#[aliases("ds")]
fn darksouls(context: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut rng = thread_rng();
    let template = DS1TEMPLATES[rng.gen_range(0, DS1TEMPLATES.len())];
    let filler = DS1FILLERS[rng.gen_range(0, DS1FILLERS.len())];
    let message = template.replace("{}", filler);
    msg.channel_id
        .say(&context, message)
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Display a randomly generated Dark Souls 3 message."]
#[aliases("ds3")]
fn darksouls3(context: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut rng = thread_rng();
    let has_conjunction = rng.gen_range(0, 2);
    if has_conjunction == 1 {
        let conjunction = DS3CONJUNCTIONS[rng.gen_range(0, DS3CONJUNCTIONS.len())];
        let mut message: String = String::new();
        for x in 0..2 {
            if x == 0 {
                message.push_str(
                    &DS3TEMPLATES[rng.gen_range(0, DS3TEMPLATES.len())]
                        .replace("{}", DS3FILLERS[rng.gen_range(0, DS3FILLERS.len())]),
                );
                if conjunction != "," {
                    message.push(' ');
                    message.push_str(conjunction);
                    message.push(' ');
                } else {
                    message.push_str(conjunction);
                    message.push(' ');
                }
            } else {
                message.push_str(
                    &DS3TEMPLATES[rng.gen_range(0, DS3TEMPLATES.len())]
                        .replace("{}", DS3FILLERS[rng.gen_range(0, DS3FILLERS.len())]),
                );
            }
        }
        msg.channel_id
            .say(&context, message)
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
    } else {
        let template = DS3TEMPLATES[rng.gen_range(0, DS3TEMPLATES.len())];
        let filler = DS3FILLERS[rng.gen_range(0, DS3FILLERS.len())];
        let message = template.replace("{}", filler);
        msg.channel_id
            .say(&context, message)
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
    }
}

#[command]
#[description = "Display the current date of the Discordian/Erisian Calendar"]
#[aliases("dd")]
fn ddate(context: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let today = Local::today();
    let message = Dday::from(today);
    msg.channel_id
        .say(&context, message)
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

fn parse_int_ordinal_suffix(num: u32) -> &'static str {
    if num / 10 == 1 {
        "th"
    } else if num % 10 == 1 {
        "st"
    } else if num % 10 == 2 {
        "nd"
    } else if num % 10 == 3 {
        "rd"
    } else {
        "th"
    }
}
