mod services;
mod platform;

use services::{Reporter, load_config};
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    // Load configuration
    let app_config = load_config();
    tracing::info!("Loaded config: enabled={}, ws_url={}", app_config.reporter.enabled, app_config.reporter.ws_url);

    // Create reporter if enabled
    let reporter = if app_config.reporter.enabled {
        Some(Reporter::new(app_config.reporter.clone()))
    } else {
        tracing::info!("Reporter disabled in config");
        None
    };

    // Spawn background task to report window info
    let reporter_handle = Arc::new(std::sync::Mutex::new(reporter));
    let reporter_clone = reporter_handle.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;

            let reporter_opt = { reporter_clone.lock().unwrap().as_ref().cloned() };

            if let Some(reporter) = reporter_opt {
                #[cfg(target_os = "macos")]
                if let Ok(info) = platform::macos::get_frontmost_window_info_sync() {
                    reporter.send_window_info(&info);
                }

                #[cfg(target_os = "windows")]
                if let Ok(info) = platform::windows::get_frontmost_window() {
                    reporter.send_window_info(&info);
                }
            }
        }
    });

    tracing::info!("ShikenMatrix Reporter started");
    tracing::info!("Press Ctrl+C to exit");

    // Wait for Ctrl+C
    signal::ctrl_c().await?;
    tracing::info!("Received shutdown signal");

    Ok(())
}
