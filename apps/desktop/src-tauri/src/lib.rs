pub mod ai;
pub mod config;
pub mod error;
pub mod history;
pub mod keychain;
pub mod logging;
pub mod plugins;
pub mod python;
pub mod sftp;
pub mod ssh;
pub mod state;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;

/// Create and configure the Tauri application
pub fn create_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            // Initialize config directories first
            let config_dir = config::get_config_dir()?;
            std::fs::create_dir_all(&config_dir)?;
            std::fs::create_dir_all(config_dir.join("plugins"))?;
            std::fs::create_dir_all(config_dir.join("scripts"))?;
            std::fs::create_dir_all(config_dir.join("themes"))?;
            std::fs::create_dir_all(config_dir.join("history"))?;
            std::fs::create_dir_all(config_dir.join("logs"))?;
            
            tracing::info!("NeonShell config dir: {:?}", config_dir);

            // Initialize log manager
            logging::init_log_manager(config_dir.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize log manager: {}", e))?;

            // Initialize app state
            let state = Arc::new(AppState::new(app.handle().clone())?);
            app.manage(state);

            tracing::info!("NeonShell initialized successfully");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // SSH commands
            ssh::commands::create_session,
            ssh::commands::connect,
            ssh::commands::disconnect,
            ssh::commands::send_data,
            ssh::commands::resize_pty,
            ssh::commands::list_sessions,
            ssh::commands::get_session,
            // New SSH API commands
            ssh::commands::ssh_connect,
            ssh::commands::ssh_disconnect,
            ssh::commands::ssh_write,
            ssh::commands::ssh_resize,
            ssh::commands::ssh_hostkey_decision,
            ssh::commands::connect_profile,
            ssh::commands::ssh_debug_probe,
            ssh::commands::ssh_stress_write,
            // Profile commands
            config::commands::list_profiles,
            config::commands::get_profile,
            config::commands::save_profile,
            config::commands::delete_profile,
            config::commands::import_ssh_config,
            config::commands::export_ssh_config,
            // Settings commands
            config::commands::get_settings,
            config::commands::save_settings,
            // Keychain commands
            keychain::commands::store_secret,
            keychain::commands::get_secret,
            keychain::commands::delete_secret,
            keychain::commands::has_secret,
            keychain::commands::get_keyring_status,
            // Plugin commands
            plugins::commands::list_plugins,
            plugins::commands::get_plugin,
            plugins::commands::enable_plugin,
            plugins::commands::disable_plugin,
            plugins::commands::install_plugin,
            // Python script commands
            python::commands::list_scripts,
            python::commands::run_script,
            python::commands::enable_script,
            python::commands::disable_script,
            // Theme commands
            config::commands::list_themes,
            config::commands::get_theme,
            config::commands::set_theme,
            config::commands::import_theme_zip,
            config::commands::export_pack,
            config::commands::import_pack,
            // History commands
            history::commands::save_terminal_history,
            history::commands::load_terminal_history,
            history::commands::clear_terminal_history,
            history::commands::clear_all_terminal_history,
            // Logging/Debug commands
            logging::commands::get_recent_logs,
            logging::commands::clear_log_view,
            logging::commands::export_debug_bundle,
            logging::commands::get_logs_dir,
            logging::commands::reveal_in_explorer,
            // SFTP commands
            sftp::commands::sftp_list,
            sftp::commands::sftp_stat,
            sftp::commands::sftp_download,
            sftp::commands::sftp_upload,
            sftp::commands::sftp_mkdir,
            sftp::commands::sftp_delete,
            sftp::commands::sftp_rename,
            sftp::commands::sftp_home,
            // AI commands
            ai::commands::get_ai_settings,
            ai::commands::save_ai_settings,
            ai::commands::get_models,
            ai::commands::ai_chat,
            ai::commands::check_ollama,
            ai::commands::store_personal_key,
            ai::commands::delete_personal_key,
            ai::commands::gateway_auth_start,
            ai::commands::gateway_auth_poll,
            ai::commands::gateway_logout,
            ai::commands::is_gateway_authenticated,
        ])
}

