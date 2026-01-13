mod services;
mod platform;

use services::{Reporter, load_config};
use std::sync::Arc;
use tokio::signal;
use base64::{Engine as _, engine::general_purpose};

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

    // Spawn background task to report window and media info
    let reporter_handle = Arc::new(std::sync::Mutex::new(reporter));
    let reporter_clone = reporter_handle.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;

            let reporter_opt = { reporter_clone.lock().unwrap().as_ref().cloned() };

            if let Some(reporter) = reporter_opt {
                // Report window info
                #[cfg(target_os = "macos")]
                if let Ok(info) = platform::macos::get_frontmost_window_info_sync() {
                    reporter.send_window_info(&info);
                }

                #[cfg(target_os = "windows")]
                if let Ok(info) = platform::windows::get_frontmost_window() {
                    reporter.send_window_info(&info);
                }

                // Report media playback info (macOS only)
                // TEMPORARILY DISABLED - causing crashes
                // Set ENABLE_MEDIA_REPORTING=1 to enable this feature
                #[cfg(target_os = "macos")]
                if std::env::var("ENABLE_MEDIA_REPORTING").unwrap_or_default() == "1" {
                    // Wrap in catch_unwind to prevent panics from crashing the app
                    let media_result = std::panic::catch_unwind(|| {
                        match platform::macos::get_media_metadata() {
                            Ok(Some(metadata)) => {
                                match platform::macos::get_playback_state() {
                                    Ok(Some(state)) => {
                                        reporter.send_media_playback(&metadata, &state);
                                        
                                        // Upload artwork if available and not cached
                                        if let (Some(artwork_data), Some(mime_type), Some(content_id)) = 
                                            (metadata.artwork_data.as_ref(), metadata.artwork_mime_type.as_ref(), metadata.content_item_identifier.as_ref()) {
                                            // Artwork data is now binary (Arc<Vec<u8>>), no need to decode
                                            reporter.upload_artwork(content_id.clone(), artwork_data.to_vec(), mime_type.clone());
                                        }
                                    }
                                    Ok(None) => {
                                        // No media playing, skip
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to get playback state: {}", e);
                                    }
                                }
                            }
                            Ok(None) => {
                                // No media metadata available
                            }
                            Err(e) => {
                                tracing::warn!("Failed to get media metadata: {}", e);
                            }
                        }
                    });
                    
                    if let Err(e) = media_result {
                        tracing::error!("Media reporting panicked: {:?}", e);
                    }
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
