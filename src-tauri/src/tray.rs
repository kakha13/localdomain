use crate::commands::domains::sync_state_to_daemon;
use crate::db::models;
use crate::state::AppState;
use tauri::menu::{CheckMenuItem, MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

fn build_tray_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let state = app.state::<AppState>();

    let (daemon_running, caddy_running) = {
        let client = state.daemon_client.lock().unwrap();
        match client.status() {
            Ok(s) => (s.daemon_running, s.caddy_running),
            Err(_) => (false, false),
        }
    };

    let domains = {
        let conn = state.db.lock().unwrap();
        models::list_domains(&conn).unwrap_or_default()
    };

    let daemon_text = if daemon_running {
        "Daemon: Running"
    } else {
        "Daemon: Stopped"
    };
    let caddy_text = if caddy_running {
        "Caddy: Running"
    } else {
        "Caddy: Stopped"
    };

    let mut builder = MenuBuilder::new(app)
        .item(&MenuItem::with_id(
            app,
            "daemon-status",
            daemon_text,
            true,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "caddy-status",
            caddy_text,
            true,
            None::<&str>,
        )?)
        .separator();

    for domain in &domains {
        let id = format!("domain-{}", domain.id);
        let check = CheckMenuItem::with_id(
            app,
            &id,
            &domain.name,
            true,
            domain.enabled,
            None::<&str>,
        )?;
        builder = builder.item(&check);
    }

    builder
        .separator()
        .item(&MenuItem::with_id(
            app,
            "start-services",
            "Start Services",
            true,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "stop-services",
            "Stop Services",
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "open",
            "Open LocalDomain",
            true,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "quit",
            "Quit",
            true,
            None::<&str>,
        )?)
        .build()
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_tray_menu(app)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(tauri::image::Image::from_bytes(include_bytes!(
            "../icons/32x32.png"
        ))?)
        .icon_as_template(false)
        .tooltip("LocalDomain")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();

            if let Some(domain_id) = id.strip_prefix("domain-") {
                handle_domain_toggle(app, domain_id);
                refresh_tray_menu(app);
                notify_frontend(app);
                return;
            }

            match id {
                "start-services" => {
                    let state = app.state::<AppState>();
                    sync_state_to_daemon(&state).ok();
                    let client = state.daemon_client.lock().unwrap();
                    client.start_caddy().ok();
                    drop(client);
                    refresh_tray_menu(app);
                    notify_frontend(app);
                }
                "stop-services" => {
                    let state = app.state::<AppState>();
                    let client = state.daemon_client.lock().unwrap();
                    client.stop_caddy().ok();
                    drop(client);
                    refresh_tray_menu(app);
                    notify_frontend(app);
                }
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

pub fn refresh_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn notify_frontend(app: &AppHandle) {
    let _ = app.emit("state-changed", ());
}

fn handle_domain_toggle(app: &AppHandle, domain_id: &str) {
    let state = app.state::<AppState>();

    let new_enabled = {
        let conn = state.db.lock().unwrap();
        match models::get_domain(&conn, domain_id) {
            Ok(Some(domain)) => !domain.enabled,
            _ => return,
        }
    };

    {
        let conn = state.db.lock().unwrap();
        if let Ok(Some(domain)) = models::toggle_domain(&conn, domain_id, new_enabled) {
            models::insert_audit_log(
                &conn,
                if new_enabled {
                    "domain_enabled"
                } else {
                    "domain_disabled"
                },
                Some(domain_id),
                Some(&domain.name),
            )
            .ok();
        }
    }

    sync_state_to_daemon(&state).ok();
}
