#ifndef SHIKENMATRIX_H
#define SHIKENMATRIX_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

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

#endif /* SHIKENMATRIX_H */
