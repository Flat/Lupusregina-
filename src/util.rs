use std::collections::HashMap;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use directories::ProjectDirs;
use ini::Ini;
use typemap::Key;
use std::fs;

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

pub fn get_configuration() -> Result<Ini, Box<dyn std::error::Error>> {
    let project_dirs = get_project_dirs().ok_or("Failed to get proejct directories!")?;
    let config_path = project_dirs.config_dir().join("settings.ini");
    if !config_path.exists() {
        fs::create_dir_all(&config_path.parent().ok_or("Failed to get parent of path!")?)?;
        fs::File::create(&config_path)?;
    }
    Ini::load_from_file(config_path).map_err(|e| e.into())
}
