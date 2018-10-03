use chrono::Utc;
use crate::util;
use ini::Ini;
use serenity::model::gateway::Game;
use serenity::model::gateway::GameType;
use serenity::model::user::OnlineStatus;
use serenity::utils::Colour;
use serenity::CACHE;
use std::sync::Arc;

command!(info(context, msg, _args) {

    let uptime = {
        let data = context.data.lock();
        match data.get::<util::Uptime>() {
        Some(time) => {
            if let Some(boottime) = time.get("boot") {
                let now = Utc::now();
                let duration = now.signed_duration_since(*boottime);
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
             },
        None => "Uptime not available.".to_owned()
        }
    };


    let (name, face, guilds, channels) = {
        let cache = CACHE.read();
        (cache.user.name.to_owned(), cache.user.face(), cache.guilds.len().to_string(),
            cache.private_channels.len().to_string())
    };

    let _ = msg.channel_id.send_message(|m| m
      .embed(|e| e
        .colour(Colour::FABLED_PINK)
        .description(&format!("Currently running {} - {}", &crate::BOT_NAME, &crate::VERSION))
        .title("Running Information")
        .author(|mut a| {
          a = a.name(&name);
          a = a.icon_url(&face);
          a
        })
        .field("Uptime", &uptime, false)
        .field("Guilds", guilds, false)
        .field("Private Channels", channels, false)
        )
      );

});

command!(reload(context, msg, _args){
    let result: Result<(), Box<std::error::Error>> = try {
        let config_path =  crate::util::get_project_dirs().ok_or("Failed to get project dirs")?
            .config_dir().join("settings.ini");
        let conf = Ini::load_from_file(config_path)?;
        {
            let mut data = context.data.lock();
            let data_conf = data.get_mut::<crate::util::Config>()
                .ok_or("Failed to read config from Client Data")?;
            *data_conf = Arc::new(conf);
        }
        ()
    };

    match result {
        Ok(_) => { log_error!(msg.channel_id.say("Reloaded config!")); },
        Err(e) => {
            error!("Failed to reload config: {}", e);
            log_error!(msg.channel_id.say("Failed to reload config!"));
        }
    }

});

command!(ping(_context, msg, _args) {
    let result: Result<(), Box<std::error::Error>> = try {
        let now = Utc::now();
        let mut msg = msg.channel_id.say("Ping!")?;
        let finish = Utc::now();
        let lping = ((finish.timestamp() - now.timestamp()) * 1000) + (finish.timestamp_subsec_millis() as i64  - now.timestamp_subsec_millis() as i64);
        msg.edit(|m| m.content(&format!("{}ms", lping)))?
    };
    match result {
        Ok(()) => (),
        Err(e) => error!("{}", e)
    };

});

command!(online(context, _msg, _args){
    context.online();
});

command!(idle(context, _msg, _args){
    context.idle();
});

command!(dnd(context, _msg, _args){
    context.dnd();
});

command!(invisible(context, _msg, _args){
    context.invisible();
});

command!(reset(context, _msg, _args){
    context.reset_presence();
});

command!(game(context, msg, args){
    let result: Result<(), Box<std::error::Error>> = try {
        let status = match args.single::<String>()?.to_ascii_uppercase().as_ref() {
            "ONLINE" => OnlineStatus::Online,
            "IDLE" => OnlineStatus::Idle,
            "DND" => OnlineStatus::DoNotDisturb,
            "INVISIBLE" => OnlineStatus::Invisible,
            "OFFLINE" => OnlineStatus::Offline,
            _ => Err("Invalid status")?
        };
        let kind = match args.single::<String>()?.to_ascii_uppercase().as_ref() {
            "PLAYING" => GameType::Playing,
            "LISTENING" => GameType::Listening,
            "STREAMING" => GameType::Streaming,
            _ => Err("Invalid type")?
        };
        match kind {
            GameType::Playing => context.set_presence(Some(Game::playing(args.rest())), status),
            GameType::Listening => context.set_presence(Some(Game::listening(args.rest())), status),
            GameType::Streaming => {
                let url = args.single::<String>()?;
                context.set_presence(Some(Game::streaming(args.rest(), &url)), status)
            }
        }
        ()

    };
    match result {
        Ok(_) => (),
        Err(e) => {
            error!("Error setting presence: {:?}", e);
            log_error!(msg.channel_id.say(&format!("{}", e)));
        }
    }
});
