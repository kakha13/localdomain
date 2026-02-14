pub mod migrations;
pub mod models;

use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub fn get_db_path() -> PathBuf {
    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.localdomain.app");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("localdomain.db")
}

pub fn init_db() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}
