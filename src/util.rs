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

use std::fs;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use directories::ProjectDirs;
use html2text::render::text_renderer::{TaggedLine, TextDecorator};
use ini::Ini;
use poise::serenity_prelude::ShardManager;
use tokio::sync::Mutex;

pub struct Data {
    pub(crate) config: Mutex<Ini>,
    pub(crate) uptime: Arc<DateTime<Utc>>,
    pub(crate) shard_manager: Arc<Mutex<ShardManager>>,
}

pub fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("moe.esoteric", "flat", "Lupusreginaβ")
}

pub fn get_configuration() -> Result<Ini> {
    let project_dirs =
        get_project_dirs().ok_or_else(|| anyhow!("Failed to get project directories!"))?;
    let config_path = project_dirs.config_dir().join("settings.ini");
    if !config_path.exists() {
        fs::create_dir_all(
            &config_path
                .parent()
                .ok_or_else(|| anyhow!("Failed to get parent of path!"))?,
        )?;
        fs::File::create(&config_path)?;
    }
    Ini::load_from_file(config_path).map_err(|e| e.into())
}

#[derive(Clone)]
pub struct DiscordMarkdownDecorator {
    links: Vec<String>,
}

impl DiscordMarkdownDecorator {
    /// Create a new `DiscordMarkdownDecorator`.
    #[cfg_attr(feature = "clippy", allow(new_without_default_derive))]
    pub fn new() -> DiscordMarkdownDecorator {
        DiscordMarkdownDecorator { links: Vec::new() }
    }
}

impl Default for DiscordMarkdownDecorator {
    fn default() -> Self {
        Self::new()
    }
}

impl TextDecorator for DiscordMarkdownDecorator {
    type Annotation = ();

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        ("".to_string(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        "".to_string()
    }

    fn decorate_em_start(&mut self) -> (String, Self::Annotation) {
        ("*".to_string(), ())
    }

    fn decorate_em_end(&mut self) -> String {
        "*".to_string()
    }

    fn decorate_strong_start(&mut self) -> (String, Self::Annotation) {
        ("**".to_string(), ())
    }

    fn decorate_strong_end(&mut self) -> String {
        "**".to_string()
    }

    fn decorate_strikeout_start(&mut self) -> (String, Self::Annotation) {
        ("--".to_string(), ())
    }

    fn decorate_strikeout_end(&mut self) -> String {
        "--".to_string()
    }

    fn decorate_code_start(&mut self) -> (String, Self::Annotation) {
        ("```".to_string(), ())
    }

    fn decorate_code_end(&mut self) -> String {
        "```".to_string()
    }

    fn decorate_preformat_first(&mut self) -> Self::Annotation {}

    fn decorate_preformat_cont(&mut self) -> Self::Annotation {}

    fn decorate_image(&mut self, title: &str) -> (String, Self::Annotation) {
        (title.to_string(), ())
    }

    fn finalise(self) -> Vec<TaggedLine<()>> {
        Vec::new()
    }

    fn make_subblock_decorator(&self) -> Self {
        DiscordMarkdownDecorator::new()
    }
}
