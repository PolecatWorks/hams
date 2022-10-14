#ifndef ffi_log2_h
#define ffi_log2_h

/* Generated with cbindgen:0.24.3 */

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * FFI-safe borrowed Rust &str. Can represents `Option<&str>` by setting ptr to null.
 */
typedef struct RustStr {
  /**
   * pointer to c-FFI safe string chars
   */
  const uint8_t *ptr;
  /**
   * length of rust string for C
   */
  uintptr_t len;
} RustStr;

/**
 * FFI-safe Metadata
 */
typedef struct ExternCMetadata {
  /**
   * Log verbosity
   */
  Level level;
  /**
   * Log target
   */
  struct RustStr target;
} ExternCMetadata;

/**
 * FFI-safe owned Rust String.
 */
typedef struct RustString {
  /**
   * pointer to characters
   */
  uint8_t *ptr;
  /**
   * capacity
   */
  uintptr_t cap;
  /**
   * length
   */
  uintptr_t len;
} RustString;

/**
 * FFI-safe Record
 */
typedef struct ExternCRecord {
  /**
   * Extern C Metadata
   */
  struct ExternCMetadata metadata;
  /**
   * fmt::Arguments<'a> are not FFI-safe, so we have no option but to format them beforehand.
   */
  struct RustString message;
  /**
   * module path RustStr
   */
  struct RustStr module_path;
  /**
   * file name RustStr
   */
  struct RustStr file;
  /**
   * Line number of log entry
   */
  int64_t line;
} ExternCRecord;

/**
 * LogParam is LogParam is a struct that transports the necessary objects to enable the configuration of the DLL logger.  * This structure must be FFI-safe. It must be constructured into FFI safe structures from the original structures on teh sending side and reconstruced into the log structures on teh consume size of log functions.
 */
typedef struct LogParam {
  /**
   * function to check if logging is enabled
   */
  bool (*enabled)(struct ExternCMetadata);
  /**
   * Write a log record
   */
  void (*log)(const struct ExternCRecord*);
  /**
   * flush the logs
   */
  void (*flush)(void);
  /**
   * value for the log level
   */
  LevelFilter level;
} LogParam;

#endif /* ffi_log2_h */
