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
use rand::Rng;
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

#[command]
#[description = "Display a randomly generated Dark Souls message."]
#[aliases("ds")]
fn darksouls(context: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let templates = vec![
        ("{} ahead", 1),
        ("Be wary of {}", 1),
        ("Try {}", 1),
        ("Need {}", 1),
        ("Imminent {}...", 1),
        ("Weakness: {}", 1),
        ("{}", 1),
        ("{}?", 1),
        ("Good Luck", 0),
        ("I did it!", 0),
        ("Here!", 0),
        ("I can't take this...", 0),
        ("Praise the Sun!", 0),
    ];
    let fillers = vec![
        "Enemy",
        "Tough enemy",
        "Hollow",
        "Soldier",
        "Knight",
        "Sniper",
        "Caster",
        "Giant",
        "Skeleton",
        "Ghost",
        "Bug",
        "Poison bug",
        "Lizard",
        "Drake",
        "Flier",
        "Golem",
        "Statue",
        "Monster",
        "Strange creature",
        "Demon",
        "Darkwraith",
        "Dragon",
        "Boss",
        "Saint",
        "Wretch",
        "Charmer",
        "Miscreant",
        "Liar",
        "Fatty",
        "Beanpole",
        "Merchant",
        "Blacksmith",
        "Master",
        "Prisoner",
        "Bonfire",
        "Fog wall",
        "Humanity",
        "Lever",
        "Switch",
        "Key",
        "Treasure",
        "Chest",
        "Weapon",
        "Shield",
        "Projectile",
        "Armour",
        "Item",
        "Ring",
        "Sorcery scroll",
        "Pyromancy scroll",
        "Miracle scroll",
        "Ember",
        "Trap",
        "Covenant",
        "Amazing key",
        "Amazing treasure",
        "Amazing chest",
        "Amazing Weapon",
        "Amazing shield",
        "Amazing projectile",
        "Amazing armour",
        "Amazing item",
        "Amazing ring",
        "Amazing sorcery scroll",
        "Amazing pyromancy scroll",
        "Amazing miracle scroll",
        "Amazing ember",
        "Amazing trap",
        "Close-ranged combat",
        "Ranged battle",
        "Eliminating one at a time",
        "Luring it out",
        "Beating to a pulp",
        "Lying in ambush",
        "Stealth",
        "Mimicry",
        "Pincer attack",
        "Hitting them in one swoop",
        "Fleeing",
        "Charing",
        "Stabbing in the back",
        "Sweeping attack",
        "Shield breaking",
        "Shield breaking",
        "Head shots",
        "Sorcery",
        "Pyromancy",
        "Miracles",
        "Jumping off",
        "Sliding down",
        "Dashing through",
        "Rolling",
        "Backstepping",
        "Jumping",
        "Attacking",
        "Holding with both hands",
        "Kicking",
        "A plunging attack",
        "Blocking",
        "Parrying",
        "Locking-on",
        "Path",
        "Hidden path",
        "Shortcut",
        "Detour",
        "Illusionary wall",
        "Dead end",
        "Swamp",
        "Lava",
        "Forest",
        "Cave",
        "Labyrinth",
        "Safe zone",
        "Danger zone",
        "Sniper spot",
        "Bright spot",
        "Dark spot",
        "Open area",
        "Tight spot",
        "Hidden place",
        "Exchange",
        "Gorgeous view",
        "Fall",
        "Front",
        "Back",
        "Left",
        "Right",
        "Up",
        "Down",
        "Feet",
        "Head",
        "Back",
        "Head",
        "Neck",
        "Stomach",
        "Arm",
        "Leg",
        "Heel",
        "Rear",
        "Tail",
        "Wings",
        "Anywhere",
        "Stike",
        "Thrust",
        "Slash",
        "Magic",
        "Fire",
        "Lightning",
        "Critical hits",
        "Bleeding",
        "Poison",
        "Strong poison",
        "Curses",
        "Divine",
        "Occult",
        "Crystal",
        "Chance",
        "Hint",
        "Secret",
        "Happiness",
        "Sorrow",
        "Life",
        "Death",
        "Undead",
        "Elation",
        "Grief",
        "Hope",
        "Despair",
        "Light",
        "Dark",
        "Bravery",
        "Resignation",
        "Comfort",
        "Tears",
    ];
    let mut rng = thread_rng();
    let (template, filler_count) = templates[rng.gen_range(0, templates.len())];
    if filler_count > 0 {
        let mut values: Vec<&str> = Vec::new();
        for _ in 0..filler_count {
            values.push(fillers[rng.gen_range(0, fillers.len())])
        }
        let mut string_to_send = template.to_string();
        for x in values {
            string_to_send = string_to_send.replacen("{}", x, 1);
        }
        msg.channel_id
            .say(&context, string_to_send)
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
    } else {
        msg.channel_id
            .say(&context, template)
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
    }
}
