//! FFI functions for accessibility permission management

#[cfg(target_os = "macos")]
use crate::platform::macos::{check_accessibility_permission, request_accessibility_permission};

/// Check if accessibility permission is granted
///
/// # Returns
/// * `true` - Permission granted
/// * `false` - Permission not granted
#[no_mangle]
pub extern "C" fn sm_check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        check_accessibility_permission()
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        true // Always return true on non-macOS platforms
    }
}

/// Request accessibility permission
///
/// This will show the system permission dialog if not already granted
///
/// # Returns
/// * `true` - Permission already granted or request succeeded
/// * `false` - Permission not granted (user needs to manually enable in System Settings)
#[no_mangle]
pub extern "C" fn sm_request_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        match request_accessibility_permission() {
            Ok(granted) => granted,
            Err(_) => false,
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Check if media API is available
///
/// This checks if the media API can be called without being blocked by Gatekeeper.
/// Uses a timeout to detect if the library is blocked (blocked calls may hang).
///
/// # Returns
/// * `true` - Media API is available
/// * `false` - Media API is not available (library blocked by Gatekeeper)
#[no_mangle]
pub extern "C" fn sm_check_media_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        use tracing::{info, error, warn};
        use std::sync::atomic::{AtomicU8, Ordering};
        
        // 0 = not checked, 1 = available, 2 = blocked
        static MEDIA_STATUS: AtomicU8 = AtomicU8::new(0);
        
        // Return cached result if already checked
        let status = MEDIA_STATUS.load(Ordering::SeqCst);
        if status == 1 {
            return true;
        } else if status == 2 {
            return false;
        }
        
        info!("Checking media API availability...");
        
        // Check if there's a marker file indicating the library was blocked
        let marker_path = dirs::home_dir()
            .map(|h| h.join(".shikenmatrix").join(".media_blocked"))
            .unwrap_or_default();
        
        if marker_path.exists() {
            warn!("Found media blocked marker file, assuming blocked");
            MEDIA_STATUS.store(2, Ordering::SeqCst);
            return false;
        }
        
        // Try to call the media API with a timeout
        // If Gatekeeper blocks it, the call might hang or fail
        let (tx, rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            // Try to call is_playing - this should return quickly if the library is loaded
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                mediaremote_rs::is_playing()
            }));
            let _ = tx.send(result);
        });
        
        // Wait for result with timeout (5 seconds - more generous)
        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(Ok(_)) => {
                // Call succeeded
                info!("✅ Media API is available");
                MEDIA_STATUS.store(1, Ordering::SeqCst);
                true
            }
            Ok(Err(e)) => {
                // Call panicked - definitely blocked
                error!("❌ Media API panicked: {:?}", e);
                // Create marker file
                if let Some(parent) = marker_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&marker_path, "blocked");
                MEDIA_STATUS.store(2, Ordering::SeqCst);
                false
            }
            Err(_) => {
                // Timeout - might be blocked, but could also be slow
                // Don't mark as blocked permanently, just warn
                warn!("⚠️ Media API call timed out (might be blocked or slow)");
                // Don't create marker file - let it retry next time
                // But cache as blocked for this session
                MEDIA_STATUS.store(2, Ordering::SeqCst);
                false
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Reset media permission check (removes the blocked marker)
/// Call this after user has allowed the library in System Settings
#[no_mangle]
pub extern "C" fn sm_reset_media_permission_check() {
    #[cfg(target_os = "macos")]
    {
        use std::sync::atomic::{AtomicU8, Ordering};
        use tracing::info;
        
        // Reset the static status
        static MEDIA_STATUS: AtomicU8 = AtomicU8::new(0);
        MEDIA_STATUS.store(0, Ordering::SeqCst);
        
        // Remove the marker file
        let marker_path = dirs::home_dir()
            .map(|h| h.join(".shikenmatrix").join(".media_blocked"))
            .unwrap_or_default();
        
        if marker_path.exists() {
            let _ = std::fs::remove_file(&marker_path);
            info!("Removed media blocked marker file");
        }
        
        info!("Media permission check reset");
    }
}
