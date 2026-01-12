//
//  RustBridge.swift
//  ShikenMatrix
//
//  FFI bridge to Rust library
//

import Foundation
import AppKit

// MARK: - Image Cache (防止反复解码)
class ImageCache {
    static let shared = ImageCache()
    private let cache = NSCache<NSString, NSImage>()
    
    init() {
        cache.countLimit = 50 // Limit to 50 items
    }
    
    /// Get image from cache or decode from data
    func image(for key: String, from data: Data?) -> NSImage? {
        let nsKey = key as NSString
        if let cached = cache.object(forKey: nsKey) {
            return cached
        }
        
        guard let data = data, let image = NSImage(data: data) else {
            return nil
        }
        
        cache.setObject(image, forKey: nsKey)
        return image
    }
    
    /// Get window icon from cache or System API
    func windowIcon(for pid: UInt32, key: String) -> NSImage? {
        let nsKey = key as NSString
        if let cached = cache.object(forKey: nsKey) {
            return cached
        }
        
        // Fetch from system API
        if let app = NSRunningApplication(processIdentifier: pid_t(pid)), let icon = app.icon {
            cache.setObject(icon, forKey: nsKey)
            return icon
        }
        
        return nil
    }
    
    func clearCache() {
        cache.removeAllObjects()
    }
}

// MARK: - FFI Declarations

@_silgen_name("sm_config_load")
func sm_config_load() -> UnsafeMutableRawPointer

@_silgen_name("sm_config_save")
func sm_config_save(_ config: UnsafeRawPointer) -> Bool

@_silgen_name("sm_config_free")
func sm_config_free(_ config: UnsafeMutableRawPointer)

@_silgen_name("sm_string_free")
func sm_string_free(_ s: UnsafeMutableRawPointer)

@_silgen_name("sm_reporter_start")
func sm_reporter_start(_ config: UnsafeRawPointer) -> UnsafeMutableRawPointer

@_silgen_name("sm_reporter_stop")
func sm_reporter_stop(_ handle: UnsafeMutableRawPointer) -> Bool

@_silgen_name("sm_reporter_get_status")
func sm_reporter_get_status(_ handle: UnsafeRawPointer) -> SmStatus

@_silgen_name("sm_reporter_is_running")
func sm_reporter_is_running() -> Bool

@_silgen_name("sm_reporter_set_log_callback")
func sm_reporter_set_log_callback(_ callback: @convention(c) (UInt8, UnsafePointer<CChar>, UInt) -> Void, _ userData: UInt)

@_silgen_name("sm_reporter_set_window_callback")
func sm_reporter_set_window_callback(_ callback: @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>, UInt32, UnsafePointer<UInt8>?, Int, UInt) -> Void, _ userData: UInt)

@_silgen_name("sm_reporter_set_media_callback")
func sm_reporter_set_media_callback(_ callback: @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>, UnsafePointer<CChar>, Double, Double, Bool, UnsafePointer<UInt8>?, Int, UInt) -> Void, _ userData: UInt)

@_silgen_name("sm_check_accessibility_permission")
func sm_check_accessibility_permission() -> Bool

@_silgen_name("sm_request_accessibility_permission")
func sm_request_accessibility_permission() -> Bool

@_silgen_name("sm_check_media_permission")
func sm_check_media_permission() -> Bool

@_silgen_name("sm_reset_media_permission_check")
func sm_reset_media_permission_check()

// MARK: - FFI Structs

/// Log level enum matching Rust
enum SmLogLevel: UInt8 {
    case info = 0
    case warning = 1
    case error = 2
}

/// C-compatible struct for Config
struct SmConfig {
    var enabled: Bool
    var wsUrl: UnsafeMutablePointer<CChar>
    var token: UnsafeMutablePointer<CChar>
    var enableMediaReporting: Bool
}

/// C-compatible struct for Status
struct SmStatus {
    var isRunning: Bool
    var isConnected: Bool
    var lastError: UnsafeMutablePointer<CChar>
}

// MARK: - Swift Models

/// Swift model for Reporter Config
struct ReporterConfig: Equatable {
    var enabled: Bool
    var wsUrl: String
    var token: String
    var enableMediaReporting: Bool
}

/// Swift model for Reporter Status
struct ReporterStatus {
    var isRunning: Bool
    var isConnected: Bool
    var lastError: String?
}

/// Window data from backend
struct WindowData {
    var title: String
    var processName: String
    var pid: UInt32
    var icon: NSImage?
}

/// Media data from backend
struct MediaData {
    var title: String
    var artist: String
    var album: String
    var duration: Double
    var elapsedTime: Double
    var playing: Bool
    var artworkData: Data?
}

// MARK: - Rust Bridge

/// Bridge to Rust library
class RustBridge {
    // Callback storage (fileprivate so wrapper functions can access)
    fileprivate static var logCallback: ((SmLogLevel, String) -> Void)?
    fileprivate static var windowCallback: ((WindowData) -> Void)?
    fileprivate static var mediaCallback: ((MediaData) -> Void)?
    
    /// Set log callback to receive formatted logs from backend
    static func setLogCallback(_ callback: @escaping (SmLogLevel, String) -> Void) {
        print("🔧 RustBridge: Setting log callback")
        logCallback = callback
        sm_reporter_set_log_callback(logCallbackWrapper, 0)
        print("✅ RustBridge: Log callback set")
    }
    
    /// Set window data callback to receive window information
    static func setWindowCallback(_ callback: @escaping (WindowData) -> Void) {
        print("🔧 RustBridge: Setting window callback")
        windowCallback = callback
        sm_reporter_set_window_callback(windowCallbackWrapper, 0)
        print("✅ RustBridge: Window callback set")
    }
    
    /// Set media data callback to receive media playback information
    static func setMediaCallback(_ callback: @escaping (MediaData) -> Void) {
        print("🔧 RustBridge: Setting media callback")
        mediaCallback = callback
        sm_reporter_set_media_callback(mediaCallbackWrapper, 0)
        print("✅ RustBridge: Media callback set")
    }

    /// Clear all callbacks to prevent memory leaks
    static func clearCallbacks() {
        print("🧹 RustBridge: Clearing all callbacks...")
        logCallback = nil
        windowCallback = nil
        mediaCallback = nil
        // Set dummy C callbacks to prevent crashes from dangling pointers
        sm_reporter_set_log_callback({ _, _, _ in }, 0)
        sm_reporter_set_window_callback({ _, _, _, _, _, _ in }, 0)
        sm_reporter_set_media_callback({ _, _, _, _, _, _, _, _, _ in }, 0)
        print("✅ RustBridge: All callbacks cleared")
    }

    /// Load configuration from Rust
    static func loadConfig() -> ReporterConfig? {
        let ptr = sm_config_load()
        guard ptr != UnsafeMutableRawPointer(bitPattern: 0) else { return nil }
        defer { sm_config_free(ptr) }

        let config = ptr.bindMemory(to: SmConfig.self, capacity: 1).pointee

        let wsUrl = String(cString: config.wsUrl)
        let token = String(cString: config.token)

        return ReporterConfig(
            enabled: config.enabled,
            wsUrl: wsUrl,
            token: token,
            enableMediaReporting: config.enableMediaReporting
        )
    }

    /// Save configuration to Rust
    static func saveConfig(_ config: ReporterConfig) -> Bool {
        guard let wsUrlPtr = strdup(config.wsUrl),
              let tokenPtr = strdup(config.token) else {
            return false
        }
        defer {
            free(wsUrlPtr)
            free(tokenPtr)
        }

        var smConfig = SmConfig(
            enabled: config.enabled,
            wsUrl: wsUrlPtr,
            token: tokenPtr,
            enableMediaReporting: config.enableMediaReporting
        )

        return withUnsafePointer(to: &smConfig) { ptr in
            sm_config_save(UnsafeRawPointer(ptr))
        }
    }

    /// Start the reporter
    static func startReporter(config: ReporterConfig) -> UnsafeMutableRawPointer? {
        guard let wsUrlPtr = strdup(config.wsUrl),
              let tokenPtr = strdup(config.token) else {
            return nil
        }
        defer {
            free(wsUrlPtr)
            free(tokenPtr)
        }

        var smConfig = SmConfig(
            enabled: config.enabled,
            wsUrl: wsUrlPtr,
            token: tokenPtr,
            enableMediaReporting: config.enableMediaReporting
        )

        // Set environment variable for media reporting
        if config.enableMediaReporting {
            setenv("ENABLE_MEDIA_REPORTING", "1", 1)
        } else {
            unsetenv("ENABLE_MEDIA_REPORTING")
        }

        let handle = withUnsafePointer(to: &smConfig) { ptr in
            sm_reporter_start(UnsafeRawPointer(ptr))
        }

        return handle == UnsafeMutableRawPointer(bitPattern: 0) ? nil : handle
    }

    /// Stop the reporter
    static func stopReporter(_ handle: UnsafeMutableRawPointer) -> Bool {
        return sm_reporter_stop(handle)
    }

    /// Get reporter status
    static func getStatus(_ handle: UnsafeMutableRawPointer) -> ReporterStatus {
        let status = sm_reporter_get_status(handle)

        let lastError: String?
        // Check if pointer is null by comparing integer address
        let isErrorNull = Int(bitPattern: status.lastError) == 0
        if isErrorNull {
            lastError = nil
        } else {
            lastError = String(cString: status.lastError)
        }

        return ReporterStatus(
            isRunning: status.isRunning,
            isConnected: status.isConnected,
            lastError: lastError
        )
    }

    /// Check if reporter is running
    static func isRunning() -> Bool {
        return sm_reporter_is_running()
    }
    
    /// Check if accessibility permission is granted
    static func checkAccessibilityPermission() -> Bool {
        return sm_check_accessibility_permission()
    }
    
    /// Request accessibility permission
    static func requestAccessibilityPermission() -> Bool {
        return sm_request_accessibility_permission()
    }
    
    /// Check if media API is available (libmediaremote_rs.dylib loaded)
    static func checkMediaPermission() -> Bool {
        return sm_check_media_permission()
    }
    
    /// Reset media permission check (removes blocked marker)
    /// Call this after user has allowed the library in System Settings
    static func resetMediaPermissionCheck() {
        sm_reset_media_permission_check()
    }
}

// MARK: - C Callback Wrappers

/// C callback wrapper for log messages
private func logCallbackWrapper(levelRaw: UInt8, message: UnsafePointer<CChar>, _: UInt) {
    let level = SmLogLevel(rawValue: levelRaw) ?? .info
    let msg = String(cString: message)
    DispatchQueue.main.async {
        RustBridge.logCallback?(level, msg)
    }
}

/// C callback wrapper for window data
private func windowCallbackWrapper(title: UnsafePointer<CChar>, processName: UnsafePointer<CChar>, pid: UInt32, iconData: UnsafePointer<UInt8>?, iconSize: Int, _: UInt) {
    // We ignore iconData from Rust because it is now null/empty to save bandwidth
    // Instead we fetch the icon natively using the PID
    
    let icon = ImageCache.shared.windowIcon(for: pid, key: "window-\(pid)")

    let data = WindowData(
        title: String(cString: title),
        processName: String(cString: processName),
        pid: pid,
        icon: icon
    )

    DispatchQueue.main.async {
        RustBridge.windowCallback?(data)
    }
}

/// C callback wrapper for media data
private func mediaCallbackWrapper(title: UnsafePointer<CChar>, artist: UnsafePointer<CChar>, album: UnsafePointer<CChar>, duration: Double, elapsed: Double, playing: Bool, artworkData: UnsafePointer<UInt8>?, artworkSize: Int, _: UInt) {
    // Only copy artwork if present and reasonable size (< 2MB)
    let artwork: Data?
    if let artworkData = artworkData, artworkSize > 0, artworkSize < 2_000_000 {
        artwork = Data(bytes: artworkData, count: artworkSize)
    } else {
        artwork = nil
    }

    let data = MediaData(
        title: String(cString: title),
        artist: String(cString: artist),
        album: String(cString: album),
        duration: duration,
        elapsedTime: elapsed,
        playing: playing,
        artworkData: artwork
    )

    DispatchQueue.main.async {
        RustBridge.mediaCallback?(data)
    }
}
