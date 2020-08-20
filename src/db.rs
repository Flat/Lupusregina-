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

use crate::util::get_project_dirs;
use serenity::model::id::GuildId;
use sqlx::sqlite::SqliteQueryAs;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(sqlx::FromRow)]
struct PrefixRow {
    guild_id: String,
    prefix: String,
}
pub async fn create_db_and_pool() -> Result<SqlitePool, Box<dyn Error>> {
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
        let db_as_str = &db
            .to_str()
            .ok_or("Unable to convert database path to UTF-8 string.")?;
        let pool = SqlitePool::new(&format!("sqlite://{}", db_as_str)).await?;
        match sqlx::query(
            "CREATE TABLE IF NOT EXISTS Prefix (guild_id TEXT PRIMARY KEY, prefix TEXT);",
        )
        .execute(&pool)
        .await
        {
            Ok(_) => Ok(pool.clone()),
            Err(why) => Err(format!("Unable to create database: {}", why).into()),
        }
    } else {
        Err("Could not open project directory when creating database".into())
    }
}

pub async fn get_guild_prefix(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<String, Box<dyn Error>> {
    let prefix_row = sqlx::query_as::<_, PrefixRow>("SELECT * FROM Prefix WHERE guild_id == ?;")
        .bind(guild_id.as_u64().to_string())
        .fetch_one(pool)
        .await?;
    Ok(prefix_row.prefix)
}

pub async fn get_all_prefixes(pool: &SqlitePool) -> Result<HashMap<u64, String>, Box<dyn Error>> {
    let prefix_rows = sqlx::query_as::<_, PrefixRow>("SELECT * FROM Prefix")
        .fetch_all(pool)
        .await?;
    let mut prefixes = HashMap::<u64, String>::new();
    for row in prefix_rows {
        prefixes.insert(row.guild_id.parse()?, row.prefix);
    }
    Ok(prefixes)
}

pub async fn set_guild_prefix(
    pool: &SqlitePool,
    guild_id: GuildId,
    prefix: String,
) -> Result<u64, Box<dyn Error>> {
    sqlx::query("INSERT OR REPLACE INTO Prefix (guild_id, prefix) values (?, ?)")
        .bind(&guild_id.as_u64().to_string())
        .bind(&prefix)
        .execute(pool)
        .await
        .map_err(|why| why.into())
}
