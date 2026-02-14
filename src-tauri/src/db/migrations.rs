use anyhow::Result;
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );",
    )?;

    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        conn.execute_batch(
            "
            CREATE TABLE domains (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL UNIQUE,
                target_host TEXT NOT NULL DEFAULT '127.0.0.1',
                target_port INTEGER NOT NULL,
                protocol    TEXT NOT NULL DEFAULT 'http',
                wildcard    INTEGER NOT NULL DEFAULT 0,
                enabled     INTEGER NOT NULL DEFAULT 1,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE audit_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                action      TEXT NOT NULL,
                domain_id   TEXT,
                details     TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT INTO schema_version (version) VALUES (1);
            ",
        )?;
    }

    if version < 2 {
        conn.execute_batch(
            "
            ALTER TABLE domains ADD COLUMN access_log INTEGER NOT NULL DEFAULT 0;
            INSERT OR REPLACE INTO schema_version (version) VALUES (2);
            ",
        )?;
    }

    if version < 3 {
        conn.execute_batch(
            "
            ALTER TABLE domains ADD COLUMN tunnel_subdomain TEXT NOT NULL DEFAULT '';
            ALTER TABLE domains ADD COLUMN tunnel_domain TEXT NOT NULL DEFAULT '';
            INSERT OR REPLACE INTO schema_version (version) VALUES (3);
            ",
        )?;
    }

    if version < 4 {
        conn.execute_batch(
            "
            ALTER TABLE domains ADD COLUMN domain_type TEXT NOT NULL DEFAULT 'proxy';
            ALTER TABLE domains ADD COLUMN document_root TEXT NOT NULL DEFAULT '';
            INSERT OR REPLACE INTO schema_version (version) VALUES (4);
            ",
        )?;
    }

    Ok(())
}
