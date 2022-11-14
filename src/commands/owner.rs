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

use chrono::Utc;
use poise::serenity_prelude::json::hashmap_to_json_map;
use poise::serenity_prelude::{Activity, Colour, OnlineStatus};
#[cfg(target_os = "linux")]
use procfs::process::Process;

use crate::{serenity, util, Context, Error};

#[poise::command(
    slash_command,
    owners_only,
    description_localized("en-US", "Shows information about the bot")
)]
pub async fn info(context: Context<'_>) -> Result<(), Error> {
    let boot_time = context.data().uptime.clone();
    let now = Utc::now();
    let duration = now.signed_duration_since(*boot_time);
    // Transform duration into days, hours, minutes, seconds.
    // There's probably a cleaner way to do this.
    let mut seconds = duration.num_seconds();
    let mut minutes = seconds / 60;
    seconds %= 60;
    let mut hours = minutes / 60;
    minutes %= 60;
    let days = hours / 24;
    hours %= 24;
    let uptime = format!("{}d{}h{}m{}s", days, hours, minutes, seconds);

    let cache = &context.discord().cache;
    let current_user = cache.current_user();
    let name = current_user.name.to_owned();
    let face = current_user.face();
    let guilds = cache.guilds().len().to_string();
    let channels = cache.private_channels().len().to_string();
    let users = cache.users().len();

    let mut desc = format!(
        "**Software version**: `{} - v{}`",
        &crate::BOT_NAME,
        &crate::VERSION
    );
    use std::fmt::Write as _;
    let _ = write!(desc, "\n**Uptime**: `{}`", &uptime);

    #[cfg(target_os = "linux")]
    if let Ok(process) = Process::myself() {
        if let Ok(page_size) = procfs::page_size() {
            if let Ok(statm) = process.statm() {
                let _ = write!(
                    desc,
                    "\n**Memory Usage**: `{:.2}MB`",
                    ((statm.resident * page_size as u64) - (statm.shared * page_size as u64))
                        as f64
                        / 1048576_f64
                );
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
                let _ = write!(desc, "\n**CPU Usage**: `{}%`", cpu_usage);
            }
        }
    };

    let _ = write!(desc, "\n**Guilds**: `{}`", guilds);
    let _ = write!(desc, "\n**Users**: `{}`", users);
    let _ = write!(desc, "\n**DM Channels**: `{}`", channels);

    context
        .send(|m| {
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
            .ephemeral(true)
        })
        .await?;
    Ok(())
}

#[poise::command(slash_command, owners_only)]
pub async fn reload(context: Context<'_>) -> Result<(), Error> {
    let conf = util::get_configuration()?;
    {
        let mut ini = context.data().config.lock().await;
        *ini = conf;
    }
    context.say("context, Reloaded config!").await?;
    Ok(())
}

#[poise::command(
    slash_command,
    owners_only,
    description_localized(
        "en-US",
        "Changes the bot's username. YOU MAY LOSE THE DISCRIMINATOR UPON CHANGING BACK!"
    )
)]
pub async fn rename(
    context: Context<'_>,
    #[description = "New username for the bot"] username: String,
) -> Result<(), Error> {
    context
        .discord()
        .cache
        .current_user()
        .edit(&context.discord(), |p| p.username(username))
        .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    owners_only,
    guild_only,
    description_localized("en-US", "Changes the bot's nickname")
)]
pub async fn nickname(
    context: Context<'_>,
    #[description = "New username for the bot"] nickname: Option<String>,
) -> Result<(), Error> {
    if let Some(guild_id) = context.guild_id() {
        context
            .discord()
            .http
            .edit_nickname(u64::from(guild_id), nickname.as_deref())
            .await?
    }

    Ok(())
}

#[poise::command(slash_command, owners_only)]
pub async fn setavatar(
    context: Context<'_>,
    attachment: serenity::Attachment,
) -> Result<(), Error> {
    let mut p = serenity::builder::EditProfile::default();
    p.avatar(Some(&format!(
        "data:{};base64,{}",
        attachment
            .content_type
            .as_ref()
            .ok_or("Unable to determine content type")?,
        base64::encode(attachment.download().await?)
    )));
    let map = hashmap_to_json_map(p.0);
    context.discord().http.edit_profile(&map).await?;
    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum OnlineStatusChoice {
    Online,
    Idle,
    DoNotDisturb,
    Invisible,
    Offline,
}

impl From<OnlineStatusChoice> for OnlineStatus {
    fn from(online_status: OnlineStatusChoice) -> Self {
        match online_status {
            OnlineStatusChoice::DoNotDisturb => OnlineStatus::DoNotDisturb,
            OnlineStatusChoice::Idle => OnlineStatus::Idle,
            OnlineStatusChoice::Invisible => OnlineStatus::Invisible,
            OnlineStatusChoice::Offline => OnlineStatus::Offline,
            OnlineStatusChoice::Online => OnlineStatus::Online,
        }
    }
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum ActivityTypeChoice {
    Playing,
    Listening,
    Streaming,
    Watching,
    Competing,
}

#[derive(Debug, poise::Modal)]
struct StatusTextModal {
    text: String,
}

#[derive(Debug, poise::Modal)]
struct TwitchModal {
    url: String,
    text: String,
}

#[poise::command(slash_command, owners_only)]
pub async fn presence(
    context: poise::ApplicationContext<'_, crate::Data, Error>,
    status: OnlineStatusChoice,
    activity: Option<ActivityTypeChoice>,
) -> Result<(), Error> {
    if let Some(activity_type_choice) = activity {
        match activity_type_choice {
            ActivityTypeChoice::Listening
            | ActivityTypeChoice::Playing
            | ActivityTypeChoice::Watching
            | ActivityTypeChoice::Competing => {
                use poise::Modal as _;
                let data = StatusTextModal::execute(context).await?;
                match activity_type_choice {
                    ActivityTypeChoice::Playing => {
                        context
                            .discord
                            .set_presence(Some(Activity::playing(data.text)), status.into())
                            .await;
                    }
                    ActivityTypeChoice::Listening => {
                        context
                            .discord
                            .set_presence(Some(Activity::listening(data.text)), status.into())
                            .await;
                    }
                    ActivityTypeChoice::Streaming => {}
                    ActivityTypeChoice::Watching => {
                        context
                            .discord
                            .set_presence(Some(Activity::watching(data.text)), status.into())
                            .await;
                    }
                    ActivityTypeChoice::Competing => {
                        context
                            .discord
                            .set_presence(Some(Activity::competing(data.text)), status.into())
                            .await;
                    }
                }
            }
            ActivityTypeChoice::Streaming => {
                use poise::Modal as _;
                let data = TwitchModal::execute(context).await?;
                context
                    .discord
                    .set_presence(
                        Some(Activity::streaming(data.text, data.url)),
                        status.into(),
                    )
                    .await;
            }
        }
    };
    Ok(())
}
