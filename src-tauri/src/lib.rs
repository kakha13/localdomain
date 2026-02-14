mod commands;
mod daemon_client;
mod db;
mod error;
pub mod paths;
mod state;
mod tray;
#[allow(dead_code)]
mod xampp;

use daemon_client::DaemonClient;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = db::init_db().expect("Failed to initialize database");
    let app_state = AppState {
        db: std::sync::Mutex::new(conn),
        daemon_client: std::sync::Mutex::new(DaemonClient::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            let state = app.state::<AppState>();

            // Check start_on_boot setting and auto-sync if enabled
            let should_start = {
                let conn = state.db.lock().unwrap();
                db::models::get_setting(&conn, "start_on_boot")
                    .ok()
                    .flatten()
                    .map(|v| v == "true")
                    .unwrap_or(false)
            };

            if should_start {
                // Sync state to daemon (configures hosts + Caddy + starts proxy)
                if let Err(e) = commands::domains::sync_state_to_daemon(&state) {
                    eprintln!("Auto-start: failed to sync state: {}", e);
                }

                // Start Caddy if not already running
                let client = state.daemon_client.lock().unwrap();
                if client.is_daemon_running() {
                    if let Err(e) = client.start_caddy() {
                        eprintln!("Auto-start: failed to start Caddy: {}", e);
                    }
                }
            }

            tray::setup_tray(app.handle())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::domains::list_domains,
            commands::domains::create_domain,
            commands::domains::update_domain,
            commands::domains::delete_domain,
            commands::domains::toggle_domain,
            commands::service::get_service_status,
            commands::service::start_service,
            commands::service::stop_service,
            commands::service::install_daemon,
            commands::service::uninstall_daemon,
            commands::service::start_apache,
            commands::service::stop_apache,
            commands::domains::toggle_access_log,
            commands::domains::trust_ca,
            commands::audit::get_audit_log,
            commands::audit::clear_audit_log,
            commands::access_log::get_access_log,
            commands::access_log::clear_access_log,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::detect_xampp_path,
            commands::settings::get_xampp_default_port,
            commands::settings::scan_xampp_vhosts,
            commands::settings::import_xampp_vhosts,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::list_tunnels,
            commands::tunnel::ensure_cloudflared,
            commands::tunnel::save_tunnel_config,
            commands::tunnel::cloudflare_check_login,
            commands::tunnel::cloudflare_login,
            commands::tunnel::cloudflare_setup_tunnel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
