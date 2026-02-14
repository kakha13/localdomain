use rusqlite::Connection;
use std::sync::Mutex;

use crate::daemon_client::DaemonClient;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub daemon_client: Mutex<DaemonClient>,
}
