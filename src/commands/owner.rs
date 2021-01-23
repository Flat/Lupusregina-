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

use std::sync::Arc;

use chrono::Utc;
#[cfg(target_os = "linux")]
use procfs::process::Process;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::user::OnlineStatus;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::util;
use serenity::model::prelude::{Activity, ActivityType};

#[command]
async fn info(context: &Context, msg: &Message) -> CommandResult {
    let uptime = {
        let data = context.data.read().await;
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

    let cache = &context.cache;
    let current_user = cache.current_user().await;
    let name = current_user.name.to_owned();
    let face = current_user.face();
    let guilds = cache.guilds().await.len().to_string();
    let channels = cache.private_channels().await.len().to_string();
    let users = cache.users().await.len();

    let mut desc = format!(
        "**Software version**: `{} - v{}`",
        &crate::BOT_NAME,
        &crate::VERSION
    );
    desc.push_str(&format!("\n**Uptime**: `{}`", &uptime));

    #[cfg(target_os = "linux")]
    if let Ok(process) = Process::myself() {
        if let Ok(page_size) = procfs::page_size() {
            if let Ok(statm) = process.statm() {
                desc.push_str(&format!(
                    "\n**Memory Usage**: `{:.2}MB`",
                    ((statm.resident * page_size as u64) - (statm.shared * page_size as u64))
                        as f64
                        / 1048576_f64
                ));
            }
        }
        if let Ok(ticks) = procfs::ticks_per_second() {
            if let Ok(kstats) = procfs::KernelStats::new() {
                let cpu_usage = 100
                    * (((process.stat.utime
                        + process.stat.stime
                        + process.stat.cutime as u64
                        + process.stat.cstime as u64)
                        / ticks as u64)
                        / (kstats.btime - (process.stat.starttime / ticks as u64)));
                desc.push_str(&format!("\n**CPU Usage**: `{}%`", cpu_usage))
            }
        }
    };

    desc.push_str(&format!("\n**Guilds**: `{}`", guilds));
    desc.push_str(&format!("\n**Users**: `{}`", users));
    desc.push_str(&format!("\n**DM Channels**: `{}`", channels));

    msg.channel_id
        .send_message(context, |m| {
            m.embed(|e| {
                e.colour(Colour::FABLED_PINK)
                    .author(|mut a| {
                        a = a.name(&name);
                        a = a.icon_url(&face);
                        a
                    })
                    .title("Running Information")
                    .description(desc)
            })
        })
        .await?;
    Ok(())
}

#[command]
async fn reload(context: &Context, msg: &Message) -> CommandResult {
    let conf = util::get_configuration()?;
    {
        let mut data = context.data.write().await;
        let data_conf = data
            .get_mut::<crate::util::Config>()
            .ok_or("Failed to read config from Client Data")?;
        *data_conf = Arc::new(conf);
    }
    msg.channel_id.say(context, "Reloaded config!").await?;
    Ok(())
}

#[command]
#[description = "Changes the bot's username. YOU MAY LOSE THE DISCRIMINATOR UPON CHANGING BACK!"]
#[usage = "\"<Username>\""]
#[example = "\"Shalltear Bloodfallen\""]
#[min_args(1)]
async fn rename(context: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>()?;
    context
        .cache
        .current_user()
        .await
        .edit(&context, |p| p.username(name))
        .await?;
    Ok(())
}

#[command]
#[description = "Changes the bot's nickname for the current guild."]
#[usage = "\"[<Username>]\""]
#[example = "\"Shalltear Bloodfallen\""]
#[only_in("guilds")]
async fn nickname(context: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        if let Some(guild_id) = msg.guild_id {
            context
                .http
                .edit_nickname(guild_id.0, None)
                .await
                .map_err(|e| e.to_string())?
        }
    } else {
        let nick = args.single_quoted::<String>()?;
        if let Some(guild_id) = msg.guild_id {
            context
                .http
                .edit_nickname(guild_id.0, Some(&nick))
                .await
                .map_err(|e| e.to_string())?
        }
    }

    Ok(())
}

#[command]
#[description = "(Un)sets the bot's avatar. Takes a url, nothing, or an attachment."]
#[usage = "[<avatar_url>]"]
#[example = "https://s4.anilist.co/file/anilistcdn/character/large/126870-DKc1B7cvoUu7.jpg"]
async fn setavatar(context: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut p = serenity::builder::EditProfile::default();
    if !msg.attachments.is_empty() {
        let url = &msg
            .attachments
            .get(0)
            .ok_or("Failed to get attachment")?
            .url;
        let response = reqwest::get(url).await?;
        let headers = response.headers().to_owned();
        if let Some(content_type) = headers.get("Content-Type") {
            let image = response.bytes().await?;
            p.avatar(Some(&format!(
                "data:{};base64,{}",
                content_type.to_str()?,
                &base64::encode(image)
            )));
            let map = serenity::utils::hashmap_to_json_map(p.0);
            context.http.edit_profile(&map).await?;
        } else {
            return Err("Unable to determine content-type.".into());
        }
    } else if args.is_empty() {
        p.avatar(None);
        let map = serenity::utils::hashmap_to_json_map(p.0);
        context.http.edit_profile(&map).await?;
    } else {
        let url = args.single::<String>()?;
        let response = reqwest::get(&url).await?;
        let headers = response.headers().to_owned();
        if let Some(content_type) = headers.get("Content-Type") {
            let image = response.bytes().await?;
            p.avatar(Some(&format!(
                "data:{};base64,{}",
                content_type.to_str()?,
                &base64::encode(image)
            )));
            let map = serenity::utils::hashmap_to_json_map(p.0);
            context.http.edit_profile(&map).await?;
        } else {
            return Err("Unable to determine content-type.".into());
        }
    }
    Ok(())
}

#[command]
async fn online(context: &Context, _msg: &Message) -> CommandResult {
    context.online().await;
    Ok(())
}

#[command]
async fn idle(context: &Context, _msg: &Message) -> CommandResult {
    context.idle().await;
    Ok(())
}

#[command]
async fn dnd(context: &Context, _msg: &Message) -> CommandResult {
    context.dnd().await;
    Ok(())
}

#[command]
async fn invisible(context: &Context, _msg: &Message) -> CommandResult {
    context.invisible().await;
    Ok(())
}

#[command]
async fn reset(context: &Context, _msg: &Message) -> CommandResult {
    context.reset_presence().await;
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
async fn set(context: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let status = match args.single::<String>()?.to_ascii_uppercase().as_ref() {
        "ONLINE" => OnlineStatus::Online,
        "IDLE" => OnlineStatus::Idle,
        "DND" => OnlineStatus::DoNotDisturb,
        "INVISIBLE" => OnlineStatus::Invisible,
        "OFFLINE" => OnlineStatus::Offline,
        _ => return Err("Invalid status".into()),
    };
    let kind = match args.single::<String>()?.to_ascii_uppercase().as_ref() {
        "PLAYING" => ActivityType::Playing,
        "LISTENING" => ActivityType::Listening,
        "STREAMING" => ActivityType::Streaming,
        _ => return Err("Invalid type".into()),
    };
    match kind {
        ActivityType::Playing => {
            context
                .set_presence(Some(Activity::playing(args.rest())), status)
                .await
        }
        ActivityType::Listening => {
            context
                .set_presence(Some(Activity::listening(args.rest())), status)
                .await
        }
        ActivityType::Streaming => {
            let url = args.single::<String>()?;
            context
                .set_presence(Some(Activity::streaming(args.rest(), &url)), status)
                .await
        }
        _ => return Err("Invalid type".into()),
    }
    Ok(())
}
