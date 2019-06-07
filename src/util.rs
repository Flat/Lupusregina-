use std::collections::HashMap;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use directories::ProjectDirs;
use ini::Ini;
use typemap::Key;

pub struct Config;

impl Key for Config {
    type Value = Arc<Ini>;
}

pub struct Uptime;

impl Key for Uptime {
    type Value = HashMap<String, DateTime<Utc>>;
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