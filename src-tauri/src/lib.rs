mod commands;
mod core;
mod platform;
mod servers;
mod state;

use state::AppState;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize app state
            let app_state = AppState::new()?;
            app.manage(app_state);

            // Build system tray menu
            let start_all = MenuItemBuilder::with_id("start_all", "Start All Services")
                .build(app)?;
            let stop_all = MenuItemBuilder::with_id("stop_all", "Stop All Services")
                .build(app)?;
            let open_dashboard = MenuItemBuilder::with_id("open_dashboard", "Open Dashboard")
                .build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit phpHerd")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&open_dashboard)
                .separator()
                .item(&start_all)
                .item(&stop_all)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("phpHerd")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open_dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "start_all" => {
                        tracing::info!("Starting all services...");
                    }
                    "stop_all" => {
                        tracing::info!("Stopping all services...");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            tracing::info!("phpHerd started successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::php::get_php_versions,
            commands::php::get_current_php_version,
            commands::php::install_php_version,
            commands::php::uninstall_php_version,
            commands::php::switch_php_version,
            commands::php::get_php_extensions,
            commands::php::toggle_php_extension,
            commands::sites::get_sites,
            commands::sites::get_parked_paths,
            commands::sites::park_directory,
            commands::sites::unpark_directory,
            commands::sites::link_site,
            commands::sites::unlink_site,
            commands::sites::isolate_site_php,
            commands::sites::secure_site,
            commands::sites::unsecure_site,
            commands::sites::install_phpmyadmin,
            commands::nginx::get_nginx_status,
            commands::nginx::install_nginx,
            commands::nginx::start_nginx,
            commands::nginx::stop_nginx,
            commands::nginx::restart_nginx,
            commands::services::get_services,
            commands::services::get_available_services,
            commands::services::create_service,
            commands::services::download_service_binary,
            commands::services::start_service,
            commands::services::stop_service,
            commands::services::delete_service,
            commands::node::get_node_versions,
            commands::node::get_current_node_version,
            commands::node::install_node_version,
            commands::node::switch_node_version,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::dumps::get_dumps,
            commands::dumps::clear_dumps,
            commands::logs::get_log_files,
            commands::logs::get_log_entries,
            commands::mail::get_emails,
            commands::mail::delete_email,
            commands::mail::clear_all_emails,
        ])
        .run(tauri::generate_context!())
        .expect("error while running phpHerd");
}
