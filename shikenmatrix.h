#ifndef SHIKENMATRIX_H
#define SHIKENMATRIX_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Log level for callback
 */
enum SmLogLevel {
  Info = 0,
  Warning = 1,
  Error = 2,
};
typedef uint8_t SmLogLevel;

/**
 * Configuration for the reporter
 */
typedef struct SmConfig {
  /**
   * Whether the reporter is enabled
   */
  bool enabled;
  /**
   * WebSocket URL (null-terminated string, owned by Rust)
   */
  char *ws_url;
  /**
   * Authentication token (null-terminated string, owned by Rust)
   */
  char *token;
  /**
   * Whether to enable media reporting
   */
  bool enable_media_reporting;
} SmConfig;

/**
 * Opaque handle for Reporter instance
 */
typedef struct SmReporter {

} SmReporter;

/**
 * Status of the reporter
 */
typedef struct SmStatus {
  /**
   * Whether the reporter is running
   */
  bool is_running;
  /**
   * Whether the WebSocket is connected
   */
  bool is_connected;
  /**
   * Last error message (null-terminated string, owned by Rust, null if no error)
   */
  char *last_error;
} SmStatus;

/**
 * Callback function type for logs
 */
typedef void (*SmLogCallback)(SmLogLevel level, const char *message, uintptr_t user_data);

/**
 * Callback function type for window data (with icon)
 */
typedef void (*SmWindowDataCallback)(const char *title,
                                     const char *process_name,
                                     uint32_t pid,
                                     const uint8_t *icon_data,
                                     uintptr_t icon_size,
                                     uintptr_t user_data);

/**
 * Callback function type for media data (with artwork)
 */
typedef void (*SmMediaDataCallback)(const char *title,
                                    const char *artist,
                                    const char *album,
                                    double duration,
                                    double elapsed_time,
                                    bool playing,
                                    const uint8_t *artwork_data,
                                    uintptr_t artwork_size,
                                    uintptr_t user_data);

/**
 * Check if accessibility permission is granted
 *
 * # Returns
 * * `true` - Permission granted
 * * `false` - Permission not granted
 */
bool sm_check_accessibility_permission(void);

/**
 * Request accessibility permission
 *
 * This will show the system permission dialog if not already granted
 *
 * # Returns
 * * `true` - Permission already granted or request succeeded
 * * `false` - Permission not granted (user needs to manually enable in System Settings)
 */
bool sm_request_accessibility_permission(void);

/**
 * Check if media API is available
 *
 * This checks if the media API can be called without being blocked by Gatekeeper.
 * Uses a timeout to detect if the library is blocked (blocked calls may hang).
 *
 * # Returns
 * * `true` - Media API is available
 * * `false` - Media API is not available (library blocked by Gatekeeper)
 */
bool sm_check_media_permission(void);

/**
 * Reset media permission check (removes the blocked marker)
 * Call this after user has allowed the library in System Settings
 */
void sm_reset_media_permission_check(void);

/**
 * Load configuration from file
 *
 * Returns a pointer to SmConfig that must be freed with sm_config_free
 */
struct SmConfig *sm_config_load(void);

/**
 * Save configuration to file
 *
 * # Arguments
 * * `config` - Pointer to SmConfig struct (will not be modified or freed)
 *
 * # Returns
 * * `true` - Configuration saved successfully
 * * `false` - Failed to save (config was null or save failed)
 */
bool sm_config_save(const struct SmConfig *config);

/**
 * Free a SmConfig struct created by sm_config_load
 *
 * # Arguments
 * * `config` - Pointer to SmConfig to free (must not be null)
 */
void sm_config_free(struct SmConfig *config);

/**
 * Free a string allocated by Rust
 *
 * This should be used for any *mut c_char returned from other FFI functions
 * when the caller is finished with it.
 *
 * # Arguments
 * * `s` - Pointer to string to free (safe if null)
 */
void sm_string_free(char *s);

/**
 * Start the reporter with the given configuration
 *
 * # Arguments
 * * `config` - Pointer to SmConfig struct (will not be modified or freed)
 *
 * # Returns
 * * Non-null pointer - Handle to the running reporter (opaque)
 * * Null pointer - Failed to start reporter (config was null or reporter already running)
 *
 * # Safety
 * The returned pointer must be passed to sm_reporter_stop to clean up resources
 */
struct SmReporter *sm_reporter_start(const struct SmConfig *config);

/**
 * Stop the running reporter
 *
 * # Arguments
 * * `handle` - Handle returned by sm_reporter_start
 *
 * # Returns
 * * `true` - Reporter stopped successfully
 * * `false` - Failed to stop (invalid handle or reporter not running)
 */
bool sm_reporter_stop(struct SmReporter *_handle);

/**
 * Get the current status of the reporter
 *
 * # Arguments
 * * `handle` - Handle returned by sm_reporter_start (ignored but kept for API consistency)
 *
 * # Returns
 * * SmStatus struct containing the current status
 */
struct SmStatus sm_reporter_get_status(const struct SmReporter *_handle);

/**
 * Check if the reporter is currently running
 *
 * # Returns
 * * `true` - Reporter is running
 * * `false` - Reporter is not running
 */
bool sm_reporter_is_running(void);

/**
 * Set log callback for receiving formatted logs from backend
 *
 * # Arguments
 * * `callback` - Function pointer to log callback
 * * `user_data` - User data value to pass to callback
 */
void sm_reporter_set_log_callback(SmLogCallback callback, uintptr_t user_data);

/**
 * Set window data callback for receiving window information
 *
 * # Arguments
 * * `callback` - Function pointer to window data callback
 * * `user_data` - User data value to pass to callback
 */
void sm_reporter_set_window_callback(SmWindowDataCallback callback, uintptr_t user_data);

/**
 * Set media data callback for receiving media playback information
 *
 * # Arguments
 * * `callback` - Function pointer to media data callback
 * * `user_data` - User data value to pass to callback
 */
void sm_reporter_set_media_callback(SmMediaDataCallback callback, uintptr_t user_data);

extern bool AXIsProcessTrusted(void);

extern bool AXIsProcessTrustedWithOptions(const __CFDictionary *options);

extern void *AXUIElementCreateApplication(int32_t pid);

extern int32_t AXUIElementCopyAttributeValue(void *element, const void *attribute, void **value);

extern void CFRelease(void *cf);

#endif  /* SHIKENMATRIX_H */
