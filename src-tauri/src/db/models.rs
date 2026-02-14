use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub target_host: String,
    pub target_port: i32,
    pub protocol: String,
    pub wildcard: bool,
    pub enabled: bool,
    pub access_log: bool,
    pub created_at: String,
    pub updated_at: String,
    pub tunnel_subdomain: String,
    pub tunnel_domain: String,
    pub domain_type: String,
    pub document_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDomainRequest {
    pub name: String,
    pub target_host: Option<String>,
    pub target_port: Option<i32>,
    pub protocol: Option<String>,
    pub wildcard: Option<bool>,
    pub domain_type: Option<String>,
    pub document_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDomainRequest {
    pub id: String,
    pub name: Option<String>,
    pub target_host: Option<String>,
    pub target_port: Option<i32>,
    pub protocol: Option<String>,
    pub wildcard: Option<bool>,
    pub enabled: Option<bool>,
    pub domain_type: Option<String>,
    pub document_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub action: String,
    pub domain_id: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

pub fn list_domains(conn: &Connection) -> Result<Vec<Domain>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, target_host, target_port, protocol, wildcard, enabled, access_log, created_at, updated_at, tunnel_subdomain, tunnel_domain, domain_type, document_root FROM domains ORDER BY name",
    )?;

    let domains = stmt
        .query_map([], |row| {
            Ok(Domain {
                id: row.get(0)?,
                name: row.get(1)?,
                target_host: row.get(2)?,
                target_port: row.get(3)?,
                protocol: row.get(4)?,
                wildcard: row.get::<_, i32>(5)? != 0,
                enabled: row.get::<_, i32>(6)? != 0,
                access_log: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                tunnel_subdomain: row.get(10)?,
                tunnel_domain: row.get(11)?,
                domain_type: row.get(12)?,
                document_root: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(domains)
}

pub fn get_domain(conn: &Connection, id: &str) -> Result<Option<Domain>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, target_host, target_port, protocol, wildcard, enabled, access_log, created_at, updated_at, tunnel_subdomain, tunnel_domain, domain_type, document_root FROM domains WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Domain {
            id: row.get(0)?,
            name: row.get(1)?,
            target_host: row.get(2)?,
            target_port: row.get(3)?,
            protocol: row.get(4)?,
            wildcard: row.get::<_, i32>(5)? != 0,
            enabled: row.get::<_, i32>(6)? != 0,
            access_log: row.get::<_, i32>(7)? != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            tunnel_subdomain: row.get(10)?,
            tunnel_domain: row.get(11)?,
            domain_type: row.get(12)?,
            document_root: row.get(13)?,
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_domain(conn: &Connection, req: &CreateDomainRequest) -> Result<Domain> {
    let id = uuid::Uuid::new_v4().to_string();
    let target_host = req.target_host.as_deref().unwrap_or("127.0.0.1");
    let target_port = req.target_port.unwrap_or(0);
    let protocol = req.protocol.as_deref().unwrap_or("http");
    let wildcard = req.wildcard.unwrap_or(false);
    let domain_type = req.domain_type.as_deref().unwrap_or("proxy");
    let document_root = req.document_root.as_deref().unwrap_or("");

    conn.execute(
        "INSERT INTO domains (id, name, target_host, target_port, protocol, wildcard, domain_type, document_root) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, req.name, target_host, target_port, protocol, wildcard as i32, domain_type, document_root],
    )?;

    Ok(get_domain(conn, &id)?.unwrap())
}

pub fn update_domain(conn: &Connection, req: &UpdateDomainRequest) -> Result<Option<Domain>> {
    let existing = get_domain(conn, &req.id)?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let target_host = req.target_host.as_deref().unwrap_or(&existing.target_host);
    let target_port = req.target_port.unwrap_or(existing.target_port);
    let protocol = req.protocol.as_deref().unwrap_or(&existing.protocol);
    let wildcard = req.wildcard.unwrap_or(existing.wildcard);
    let enabled = req.enabled.unwrap_or(existing.enabled);
    let domain_type = req.domain_type.as_deref().unwrap_or(&existing.domain_type);
    let document_root = req.document_root.as_deref().unwrap_or(&existing.document_root);

    conn.execute(
        "UPDATE domains SET name = ?1, target_host = ?2, target_port = ?3, protocol = ?4, wildcard = ?5, enabled = ?6, domain_type = ?7, document_root = ?8, updated_at = datetime('now') WHERE id = ?9",
        params![name, target_host, target_port, protocol, wildcard as i32, enabled as i32, domain_type, document_root, req.id],
    )?;

    Ok(get_domain(conn, &req.id)?)
}

pub fn delete_domain(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM domains WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn toggle_domain(conn: &Connection, id: &str, enabled: bool) -> Result<Option<Domain>> {
    conn.execute(
        "UPDATE domains SET enabled = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![enabled as i32, id],
    )?;
    get_domain(conn, id)
}

pub fn insert_audit_log(
    conn: &Connection,
    action: &str,
    domain_id: Option<&str>,
    details: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (action, domain_id, details) VALUES (?1, ?2, ?3)",
        params![action, domain_id, details],
    )?;
    Ok(())
}

pub fn get_audit_log(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<AuditLogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, action, domain_id, details, created_at FROM audit_log ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )?;

    let entries = stmt
        .query_map(params![limit, offset], |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                action: row.get(1)?,
                domain_id: row.get(2)?,
                details: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(entries)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )?;
    Ok(())
}

pub fn clear_audit_log(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM audit_log", [])?;
    Ok(())
}

pub fn save_tunnel_config(conn: &Connection, id: &str, subdomain: &str, domain: &str) -> Result<()> {
    conn.execute(
        "UPDATE domains SET tunnel_subdomain = ?1, tunnel_domain = ?2, updated_at = datetime('now') WHERE id = ?3",
        params![subdomain, domain, id],
    )?;
    Ok(())
}

pub fn set_access_log(conn: &Connection, id: &str, enabled: bool) -> Result<Option<Domain>> {
    conn.execute(
        "UPDATE domains SET access_log = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![enabled as i32, id],
    )?;
    get_domain(conn, id)
}
