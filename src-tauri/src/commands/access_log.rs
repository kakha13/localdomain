use crate::error::AppError;
use crate::state::AppState;
use localdomain_shared::protocol::AccessLogEntry;
use tauri::State;

#[tauri::command]
pub fn get_access_log(
    state: State<AppState>,
    domain: String,
    limit: Option<u64>,
) -> Result<Vec<AccessLogEntry>, AppError> {
    let client = state.daemon_client.lock().unwrap();
    let entries = client
        .get_access_log(&domain, limit)
        .map_err(|e| AppError::Daemon(e.to_string()))?;
    Ok(entries)
}

#[tauri::command]
pub fn clear_access_log(state: State<AppState>, domain: String) -> Result<(), AppError> {
    let client = state.daemon_client.lock().unwrap();
    client
        .clear_access_log(&domain)
        .map_err(|e| AppError::Daemon(e.to_string()))?;
    Ok(())
}
