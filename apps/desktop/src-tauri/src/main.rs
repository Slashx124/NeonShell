// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use neonshell::create_app;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neonshell=info,warn".into()),
        )
        .init();

    create_app()
        .run(tauri::generate_context!())
        .expect("error while running NeonShell");
}

