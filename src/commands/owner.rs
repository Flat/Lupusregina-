use std::sync::Arc;

use chrono::Utc;
use ini::Ini;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::user::OnlineStatus;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::util;
use serenity::model::prelude::{Activity, ActivityType};

#[command]
fn info(context: &mut Context, msg: &Message) -> CommandResult {
    let uptime = {
        let data = context.data.read();
        match data.get::<util::Uptime>() {
            Some(time) => {
                if let Some(boottime) = time.get("boot") {
                    let now = Utc::now();
                    let duration = now.signed_duration_since(boottime.to_owned());
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

    log_error!(msg.channel_id.send_message(context, |m| m.embed(|e| e
        .colour(Colour::FABLED_PINK)
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
        .field("Private Channels", channels, false))));
    Ok(())
}

#[command]
fn reload(context: &mut Context, msg: &Message) -> CommandResult {
    let result = try {
        let config_path = crate::util::get_project_dirs()
            .ok_or("Failed to get project dirs")?
            .config_dir()
            .join("settings.ini");
        let conf = Ini::load_from_file(config_path)?;
        {
            let mut data = context.data.write();
            let data_conf = data
                .get_mut::<crate::util::Config>()
                .ok_or("Failed to read config from Client Data")?;
            *data_conf = Arc::new(conf);
        }
        ()
    };

    match result {
        Ok(_) => {
            log_error!(msg.channel_id.say(context, "Reloaded config!"));
            Ok(())
        }
        Err(e) => {
            error!("Failed to reload config: {:?}", e);
            log_error!(msg.channel_id.say(context, "Failed to reload config!"));
            Err(e)
        }
    }
}

#[command]
fn ping(context: &mut Context, msg: &Message) -> CommandResult {
    try {
        let now = Utc::now();
        let mut msg = msg.channel_id.say(&context, "Ping!")?;
        let finish = Utc::now();
        let lping = ((finish.timestamp() - now.timestamp()) * 1000)
            + (finish.timestamp_subsec_millis() as i64 - now.timestamp_subsec_millis() as i64);
        msg.edit(&context, |m| m.content(&format!("{}ms", lping)))?
    }
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
                         \nValid types are: Playing, Streaming, and Listening.\
                         If the type is streaming a URL is required as well. \n
                         For example: game online playing Overlord III \
                         \n game online streaming http://twitch.tv/ Overlord III"]
#[min_args(3)]
fn game(context: &mut Context, _msg: &Message, mut args: Args) -> CommandResult {
    try {
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
            ActivityType::Playing => {
                context.set_presence(Some(Activity::playing(args.rest())), status)
            }
            ActivityType::Listening => {
                context.set_presence(Some(Activity::listening(args.rest())), status)
            }
            ActivityType::Streaming => {
                let url = args.single::<String>()?;
                context.set_presence(Some(Activity::streaming(args.rest(), &url)), status)
            }
            _ => (),
        }
    }
}
