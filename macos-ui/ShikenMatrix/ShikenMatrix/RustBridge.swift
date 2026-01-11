//
//  RustBridge.swift
//  ShikenMatrix
//
//  FFI bridge to Rust library
//

import Foundation

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

// MARK: - FFI Structs

/// C-compatible struct for Config
struct SmConfig {
    var enabled: Bool
    var wsUrl: UnsafeMutablePointer<CChar>
    var token: UnsafeMutablePointer<CChar>
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
}

/// Swift model for Reporter Status
struct ReporterStatus {
    var isRunning: Bool
    var isConnected: Bool
    var lastError: String?
}

// MARK: - Rust Bridge

/// Bridge to Rust library
class RustBridge {
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
            token: token
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
            token: tokenPtr
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
            token: tokenPtr
        )

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
}
