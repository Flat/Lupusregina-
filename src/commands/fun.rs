/*
 * Copyright 2020 Kenneth Swenson
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

use chrono::{Datelike, Local, NaiveDate};
use rand::prelude::*;

use crate::{Context, Error};
use lazy_static::lazy_static;
use poise::serenity_prelude::Colour;
use std::cmp::Ordering;

lazy_static! {
    static ref DS1FILLERS: Vec<&'static str> =
        include_str!("data/ds1fillers.txt").split('\n').collect();
    static ref DS1TEMPLATES: Vec<&'static str> =
        include_str!("data/ds1templates.txt").split('\n').collect();
    static ref DS3TEMPLATES: Vec<&'static str> =
        include_str!("data/ds3templates.txt").split('\n').collect();
    static ref DS3FILLERS: Vec<&'static str> =
        include_str!("data/ds3fillers.txt").split('\n').collect();
    static ref DS3CONJUNCTIONS: Vec<&'static str> = include_str!("data/ds3conjunctions.txt")
        .split('\n')
        .collect();
    static ref BBTEMPLATES: Vec<&'static str> =
        include_str!("data/bbtemplates.txt").split('\n').collect();
    static ref BBFILLERS: Vec<&'static str> =
        include_str!("data/bbfillers.txt").split('\n').collect();
    static ref BBCONJUNCTIONS: Vec<&'static str> = include_str!("data/bbconjunctions.txt")
        .split('\n')
        .collect();
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
    static ref DAPOSTLES: Vec<&'static str> =
        vec!["Mungday", "Mojoday", "Syaday", "Zaraday", "Maladay",];
    static ref DHOLIDAYS: Vec<&'static str> =
        vec!["Chaosflux", "Discoflux", "Confuflux", "Bureflux", "Afflux",];
}

struct Dday {
    season_day: u32,
    day: u32,
    year: i32,
    tibs_day: bool,
}

impl From<NaiveDate> for Dday {
    fn from(date: NaiveDate) -> Self {
        let year = date.year() + 1166;
        let mut day_of_year = date.ordinal0();
        let mut tibs_day = false;
        if year % 4 == 0 && year % 100 != 0 || year % 100 == 0 && year % 400 == 0 {
            match day_of_year.cmp(&60) {
                Ordering::Equal => tibs_day = true,
                Ordering::Greater => day_of_year -= 1,
                Ordering::Less => (),
            }
        };
        let day_of_season = day_of_year % 73 + 1;
        Dday {
            year,
            season_day: day_of_season,
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
                match self.season_day {
                    5 => DAPOSTLES[(self.day / 73) as usize],
                    50 => DHOLIDAYS[(self.day / 73) as usize],
                    _ => DDAYS[(self.day % 5) as usize],
                },
                self.season_day,
                parse_int_ordinal_suffix(self.season_day),
                DSEASONS[(self.day / 73) as usize],
                self.year
            )
        }
    }
}

#[poise::command(
    slash_command,
    description_localized(
        "en-US",
        "Ask the magic eight ball your question and receive your fortune."
    ),
    aliases("8ball")
)]
pub async fn eightball(context: Context<'_>, question: String) -> Result<(), Error> {
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
    let mut rng = rand::rngs::StdRng::from_entropy();
    let num = rng.gen_range(0..=19);
    let choice = answers[num];
    let nick = context.author_member().await.and_then(|m| m.nick.clone());

    context
        .send(|m| {
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
                .description(question)
                .author(|mut a| {
                    if let Some(nick) = nick {
                        a.name(nick);
                    } else {
                        a.name(context.author().name.clone());
                    }
                    a = a.icon_url(context.author().face());
                    a
                })
                .field("🎱Eightball🎱", choice, false)
            })
        })
        .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Display a randomly generated Dark Souls message."),
    aliases("ds")
)]
pub async fn darksouls(context: Context<'_>) -> Result<(), Error> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    let template = DS1TEMPLATES[rng.gen_range(0..=DS1TEMPLATES.len())];
    let filler = DS1FILLERS[rng.gen_range(0..=DS1FILLERS.len())];
    let message = template.replace("{}", filler);
    context.say(message).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Display a randomly generated Dark Souls 3 message."),
    aliases("ds3")
)]
pub async fn darksouls3(context: Context<'_>) -> Result<(), Error> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    let has_conjunction: u8 = rng.gen_range(0..=2);
    if has_conjunction == 1 {
        let conjunction = DS3CONJUNCTIONS[rng.gen_range(0..=DS3CONJUNCTIONS.len())];
        let mut message: String = String::new();
        for x in 0..2 {
            if x == 0 {
                message.push_str(
                    &DS3TEMPLATES[rng.gen_range(0..=DS3TEMPLATES.len())]
                        .replace("{}", DS3FILLERS[rng.gen_range(0..=DS3FILLERS.len())]),
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
                    &DS3TEMPLATES[rng.gen_range(0..=DS3TEMPLATES.len())]
                        .replace("{}", DS3FILLERS[rng.gen_range(0..=DS3FILLERS.len())]),
                );
            }
        }
        context.say(message).await?;
        Ok(())
    } else {
        let template = DS3TEMPLATES[rng.gen_range(0..=DS3TEMPLATES.len())];
        let filler = DS3FILLERS[rng.gen_range(0..=DS3FILLERS.len())];
        let message = template.replace("{}", filler);
        context.say(message).await?;
        Ok(())
    }
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Display a randomly generated Bloodborne note."),
    aliases("bb")
)]
pub async fn bloodborne(context: Context<'_>) -> Result<(), Error> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    let has_conjunction: u8 = rng.gen_range(0..=2);
    if has_conjunction == 1 {
        let conjunction = BBCONJUNCTIONS[rng.gen_range(0..=BBCONJUNCTIONS.len())];
        let mut message: String = String::new();
        for x in 0..2 {
            if x == 0 {
                message.push_str(
                    &BBTEMPLATES[rng.gen_range(0..=BBTEMPLATES.len())]
                        .replace("{}", BBFILLERS[rng.gen_range(0..=BBFILLERS.len())]),
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
                    &BBTEMPLATES[rng.gen_range(0..=BBTEMPLATES.len())]
                        .replace("{}", BBFILLERS[rng.gen_range(0..=BBFILLERS.len())]),
                );
            }
        }
        context.say(message).await?;
        Ok(())
    } else {
        let template = BBTEMPLATES[rng.gen_range(0..=BBTEMPLATES.len())];
        let filler = BBFILLERS[rng.gen_range(0..=BBFILLERS.len())];
        let message = template.replace("{}", filler);
        context.say(message).await?;
        Ok(())
    }
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Display the current date of the Discordian/Erisian Calendar"),
    aliases("dd")
)]
pub async fn ddate(context: Context<'_>) -> Result<(), Error> {
    let today = Local::now();
    let message = Dday::from(today.date_naive());
    context.say(format!("{}", message)).await?;
    Ok(())
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
