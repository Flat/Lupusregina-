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

use crate::util::get_project_dirs;
use rusqlite::{Connection, NO_PARAMS};
use serenity::model::id::GuildId;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

struct PrefixRow {
    guild_id: String,
    prefix: String,
}
pub fn create_db() {
    if let Some(project_dirs) = get_project_dirs() {
        let db = project_dirs.data_dir().join("lupus.db");
        if !db.exists() {
            match fs::create_dir_all(&db.parent().unwrap()) {
                Ok(_) => match fs::File::create(&db) {
                    Ok(_) => (),
                    Err(e) => error!("Failed to create database file: {}", e),
                },
                Err(e) => error!("Error creating data directory: {}", e),
            }
        }
        if let Ok(connection) = Connection::open(&db) {
            match connection.execute(
                "CREATE TABLE IF NOT EXISTS Prefix (guild_id TEXT PRIMARY KEY, prefix TEXT);",
                NO_PARAMS,
            ) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        } else {
            error!(
                "Could not open connection to lupus.db ({})",
                &db.to_string_lossy()
            );
        }
    } else {
        error!("Could not open project directory when creating database");
    }
}

pub fn get_guild_prefix(guild_id: GuildId) -> Result<String, Box<dyn Error>> {
    let db = get_project_dirs()
        .ok_or("Could not open project directory")?
        .data_dir()
        .join("lupus.db");
    let conn = Connection::open(db)?;
    let mut statement = conn.prepare(&"SELECT * FROM Prefix WHERE guild_id == ?;")?;
    let mut rows = statement.query(&[guild_id.as_u64().to_string()])?;
    Ok(rows.next()?.ok_or("Guild not found.")?.get(1)?)
}

pub fn get_all_prefixes() -> Result<HashMap<u64, String>, Box<dyn Error>> {
    let db = get_project_dirs()
        .ok_or("Could not open project directory.")?
        .data_dir()
        .join("lupus.db");
    let conn = Connection::open(db)?;
    let mut statement = conn.prepare(&"SELECT * FROM Prefix")?;
    let rows = statement.query_map(NO_PARAMS, |row| {
        Ok(PrefixRow {
            guild_id: row.get(0)?,
            prefix: row.get(1)?,
        })
    })?;
    let mut prefixes = HashMap::<u64, String>::new();
    for row in rows {
        let row = row?;
        prefixes.insert(row.guild_id.parse()?, row.prefix);
    }
    Ok(prefixes)
}

pub fn set_guild_prefix(guild_id: GuildId, prefix: String) -> Result<(), Box<dyn Error>> {
    let db = get_project_dirs()
        .ok_or("Could not open project directory")?
        .data_dir()
        .join("lupus.db");
    let conn = Connection::open(db)?;
    match conn.execute(
        "INSERT OR REPLACE INTO Prefix (guild_id, prefix) values (?1, ?2)",
        &[&guild_id.as_u64().to_string(), &prefix],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
