use chrono::DateTime;
use chrono::Utc;
use directories::ProjectDirs;
use ini::Ini;
use serenity::model::id::UserId;
use serenity::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use typemap::Key;

pub struct Config;

impl Key for Config {
    type Value = Arc<Ini>;
}

pub struct Uptime;

impl Key for Uptime {
    type Value = HashMap<String, DateTime<Utc>>;
}

pub fn get_owner(ctx: &mut Context) -> Result<UserId, Box<Error>> {
    let data = ctx.data.lock();
    if let Some(config) = data.get::<Config>() {
        if let Some(general) = config.section(Some("General")) {
            match general.get("owner") {
                Some(owner) => Ok(UserId::from(owner.parse::<u64>()?)),
                None => Err(From::from("Unable to load 'owner' from config.")),
            }
        } else {
            Err(From::from("Error loading [General] section of config."))
        }
    } else {
        Err(From::from("Error loading config from client data."))
    }
}

pub fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("moe.esoteric", "flat", "Lupusreginaβ")
}

#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {{
        match $msg {
            Ok(_) => (),
            Err(e) => error!("Failed to send message: {}", e),
        }
    }};
}
