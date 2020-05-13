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

use std::collections::HashMap;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use directories::ProjectDirs;
use ini::Ini;
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::{Mutex, TypeMapKey};
use std::fs;

pub struct Config;

impl TypeMapKey for Config {
    type Value = Arc<Ini>;
}

pub struct Uptime;

impl TypeMapKey for Uptime {
    type Value = HashMap<String, DateTime<Utc>>;
}

pub struct ClientShardManager;

impl TypeMapKey for ClientShardManager {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct Prefixes;

impl TypeMapKey for Prefixes {
    type Value = HashMap<u64, String>;
}

pub struct DBPool;

impl TypeMapKey for DBPool {
    type Value = Arc<sqlx::SqlitePool>;
}

pub fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("moe.esoteric", "flat", "Lupusreginaβ")
}

pub fn get_configuration() -> Result<Ini, Box<dyn std::error::Error>> {
    let project_dirs = get_project_dirs().ok_or("Failed to get proejct directories!")?;
    let config_path = project_dirs.config_dir().join("settings.ini");
    if !config_path.exists() {
        fs::create_dir_all(
            &config_path
                .parent()
                .ok_or("Failed to get parent of path!")?,
        )?;
        fs::File::create(&config_path)?;
    }
    Ini::load_from_file(config_path).map_err(|e| e.into())
}
