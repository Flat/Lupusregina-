use crate::util::get_project_dirs;
use rusqlite::{Connection, NO_PARAMS};
use serenity::model::id::GuildId;
use std::error::Error;
use std::fs;

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
    let mut statement = conn.prepare(&format!(
        "SELECT * FROM Prefix WHERE guild_id == {};",
        guild_id.as_u64()
    ))?;
    let mut rows = statement.query(NO_PARAMS)?;
    Ok(rows.next()?.ok_or("Guild not found.")?.get(1)?)
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
