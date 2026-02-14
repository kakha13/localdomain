use crate::db::models::{self, AuditLogEntry};
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_audit_log(
    state: State<AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let conn = state.db.lock().unwrap();
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    Ok(models::get_audit_log(&conn, limit, offset)?)
}

#[tauri::command]
pub fn clear_audit_log(state: State<AppState>) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    models::clear_audit_log(&conn)?;
    Ok(())
}
