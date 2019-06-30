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

use std::sync::Arc;

use chrono::Utc;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::model::user::OnlineStatus;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::util;
use serenity::model::prelude::{Activity, ActivityType};
use std::fs::File;
use std::io::copy;

#[command]
fn info(context: &mut Context, msg: &Message) -> CommandResult {
    let uptime = {
        let data = context.data.read();
        match data.get::<util::Uptime>() {
            Some(time) => {
                if let Some(boot_time) = time.get("boot") {
                    let now = Utc::now();
                    let duration = now.signed_duration_since(boot_time.to_owned());
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
            }
            None => "Uptime not available.".to_owned(),
        }
    };

    let (name, face, guilds, channels) = {
        let cache = context.cache.read();
        (
            cache.user.name.to_owned(),
            cache.user.face(),
            cache.guilds.len().to_string(),
            cache.private_channels.len().to_string(),
        )
    };

    msg.channel_id
        .send_message(context, |m| {
            m.embed(|e| {
                e.colour(Colour::FABLED_PINK)
                    .description(&format!(
                        "Currently running {} - {}",
                        &crate::BOT_NAME,
                        &crate::VERSION
                    ))
                    .title("Running Information")
                    .author(|mut a| {
                        a = a.name(&name);
                        a = a.icon_url(&face);
                        a
                    })
                    .field("Uptime", &uptime, false)
                    .field("Guilds", guilds, false)
                    .field("Private Channels", channels, false)
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
fn reload(context: &mut Context, msg: &Message) -> CommandResult {
    let conf = util::get_configuration()?;
    {
        let mut data = context.data.write();
        let data_conf = data
            .get_mut::<crate::util::Config>()
            .ok_or("Failed to read config from Client Data")?;
        *data_conf = Arc::new(conf);
    }
    msg.channel_id
        .say(context, "Reloaded config!")
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Changes the bot's username. YOU MAY LOSE THE DISCRIMINATOR UPON CHANGING BACK!"]
#[usage = "\"<Username>\""]
#[example = "\"Shalltear Bloodfallen\""]
#[min_args(1)]
fn rename(context: &mut Context, _msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>()?;
    context
        .cache
        .write()
        .user
        .edit(&context, |p| p.username(name))
        .map_err(|e| CommandError(e.to_string()))
}

#[command]
#[description = "Changes the bot's nickname for the current guild."]
#[usage = "\"[<Username>]\""]
#[example = "\"Shalltear Bloodfallen\""]
#[only_in("guilds")]
fn nickname(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        if let Some(guild_id) = msg.guild_id {
            context
                .http
                .edit_nickname(guild_id.0, None)
                .map_err(|e| CommandError(e.to_string()))?
        }
    } else {
        let nick = args.single_quoted::<String>()?;
        if let Some(guild_id) = msg.guild_id {
            context
                .http
                .edit_nickname(guild_id.0, Some(&nick))
                .map_err(|e| CommandError(e.to_string()))?
        }
    }

    Ok(())
}

#[command]
#[description = "(Un)sets the bot's avatar. Takes a url, nothing, or an attachment."]
#[usage = "[<avatar_url>]"]
#[example = "https://s4.anilist.co/file/anilistcdn/character/large/126870-DKc1B7cvoUu7.jpg"]
fn setavatar(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut p = serenity::builder::EditProfile::default();
    if !msg.attachments.is_empty() {
        let url = &msg
            .attachments
            .get(0)
            .ok_or_else(|| CommandError("Failed to get attachment".to_owned()))?
            .url;
        let tmpdir = tempfile::tempdir()?;
        let mut response = reqwest::get(url)?;
        let (mut outfile, out_path) = {
            let filename = response
                .url()
                .path_segments()
                .and_then(|seg| seg.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .ok_or_else(|| CommandError("Failed to get filename from url.".to_owned()))?;
            let filename = tmpdir.path().join(filename);
            (File::create(filename.clone())?, filename)
        };
        copy(&mut response, &mut outfile)?;
        let base64 = serenity::utils::read_image(out_path)?;
        p.avatar(Some(&base64));
        let map = serenity::utils::hashmap_to_json_map(p.0);
        context.http.edit_profile(&map)?;
    } else if args.is_empty() {
        p.avatar(None);
        let map = serenity::utils::hashmap_to_json_map(p.0);
        context.http.edit_profile(&map)?;
    } else {
        let url = args.single::<String>()?;
        let tmpdir = tempfile::tempdir()?;
        let mut response = reqwest::get(&url)?;
        let (mut outfile, outpath) = {
            let filename = response
                .url()
                .path_segments()
                .and_then(|seg| seg.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .ok_or_else(|| CommandError("Failed to get filename from url.".to_owned()))?;
            let filename = tmpdir.path().join(filename);
            (File::create(filename.clone())?, filename)
        };
        copy(&mut response, &mut outfile)?;
        let base64 = serenity::utils::read_image(outpath)?;
        p.avatar(Some(&base64));
        let map = serenity::utils::hashmap_to_json_map(p.0);
        context.http.edit_profile(&map)?;
    }
    Ok(())
}

#[command]
fn online(context: &mut Context, _msg: &Message) -> CommandResult {
    context.online();
    Ok(())
}

#[command]
fn idle(context: &mut Context, _msg: &Message) -> CommandResult {
    context.idle();
    Ok(())
}

#[command]
fn dnd(context: &mut Context, _msg: &Message) -> CommandResult {
    context.dnd();
    Ok(())
}

#[command]
fn invisible(context: &mut Context, _msg: &Message) -> CommandResult {
    context.invisible();
    Ok(())
}

#[command]
fn reset(context: &mut Context, _msg: &Message) -> CommandResult {
    context.reset_presence();
    Ok(())
}

#[command]
#[description = "Sets the currently playing game name. This command takes 3 or 4 arguments: \
                 status type name.\nValid statuses are: Online, Idle, DND, Offline and Invisible.\
                 Valid types are: Playing, Streaming, and Listening.\
                 If the type is streaming a URL is required as well."]
#[usage = "<state> <activity> [<twitch url>] <status text>"]
#[example = "online streaming https://twitch.tv/HeyZeusHeresToast Bloodborne"]
#[min_args(3)]
fn set(context: &mut Context, _msg: &Message, mut args: Args) -> CommandResult {
    let status = match args.single::<String>()?.to_ascii_uppercase().as_ref() {
        "ONLINE" => OnlineStatus::Online,
        "IDLE" => OnlineStatus::Idle,
        "DND" => OnlineStatus::DoNotDisturb,
        "INVISIBLE" => OnlineStatus::Invisible,
        "OFFLINE" => OnlineStatus::Offline,
        _ => Err("Invalid status")?,
    };
    let kind = match args.single::<String>()?.to_ascii_uppercase().as_ref() {
        "PLAYING" => ActivityType::Playing,
        "LISTENING" => ActivityType::Listening,
        "STREAMING" => ActivityType::Streaming,
        _ => Err("Invalid type")?,
    };
    match kind {
        ActivityType::Playing => context.set_presence(Some(Activity::playing(args.rest())), status),
        ActivityType::Listening => {
            context.set_presence(Some(Activity::listening(args.rest())), status)
        }
        ActivityType::Streaming => {
            let url = args.single::<String>()?;
            context.set_presence(Some(Activity::streaming(args.rest(), &url)), status)
        }
        _ => Err("Invalid type")?,
    }
    Ok(())
}
