# HaMS: Health and Monitoring System

[![Rust](https://github.com/PolecatWorks/hams/actions/workflows/rust.yml/badge.svg)](https://github.com/PolecatWorks/hams/actions/workflows/rust.yml)

HaMS is a health and monitoring library written in Rust. It is designed to implement Kubernetes lifecycle interfaces (Liveness/Readiness probes) and expose metrics.

It is built as a shared object (`libhams.dylib` / `libhams.so`) so it can be utilized by multiple languages (C/C++, Python, Node.js, Java/Kotlin) via FFI.

## Project Structure

This repository consists of several key components:

-   **`hams`**: The core library. It implements the health checks (alive/ready), web server, and FFI interface.
-   **`hamsrs`**: A safe Rust wrapper around the `hams` FFI. Use this if you are integrating HaMS into a Rust application.
-   **`ffi-log2`**: A utility library that enables the shared object to log via the host application's logger, ensuring unified logging.
-   **`sample-rust`**: An example Rust application demonstrating how to use `hamsrs` and `ffi-log2`.

## Usage (Rust)

To run the Rust sample application, which demonstrates a fully integrated HaMS service:

```bash
cargo watch -x "run -- --config sample-rust/test_data/config.yaml start"
```

This starts the service with a configuration file that sets the web server prefix to `api`.

## API Reference

The HaMS web server exposes the following endpoints (default port 8080, configurable):

### Health Checks

*   **Liveness Probe**: `GET /hams/alive`
    *   **Returns**: `200 OK` (if healthy) or `503 Service Unavailable` (if unhealthy).
    *   **Response Body**:
        ```json
        {
          "name": "HamName",
          "valid": true
        }
        ```

*   **Liveness Probe (Verbose)**: `GET /hams/alive_verbose`
    *   **Returns**: Same status codes as above, but includes details for each registered probe.
    *   **Response Body**:
        ```json
        {
          "name": "HamName",
          "valid": true,
          "details": [
            { "name": "probe1", "valid": true },
            { "name": "probe2", "valid": false }
          ]
        }
        ```

*   **Readiness Probe**: `GET /hams/ready`
    *   Similar to `/hams/alive`.

*   **Readiness Probe (Verbose)**: `GET /hams/ready_verbose`
    *   Similar to `/hams/alive_verbose`.

### Lifecycle & Metrics

*   **Version Info**: `GET /hams/version`
    *   Returns JSON with service and library version details.

*   **Shutdown**: `POST /hams/shutdown`
    *   Triggers the registered shutdown callback in the host application.

*   **Metrics**: `GET /hams/metrics`
    *   Returns the output of the registered Prometheus callback (text/plain).

## Integration Details

### `hamsrs` (Rust Integration)

`hamsrs` provides a high-level, safe Rust API.

```rust
use hamsrs::{Hams, ProbeManual};
use hamsrs::hams::config::HamsConfig;

// Initialize logging (bridges hams internal logs to your app's logger)
hamsrs::hams_logger_init(ffi_log2::log_param()).unwrap();

// Create probes
let manual_probe = ProbeManual::new("manual_check", true).unwrap();

// Create and start HaMS
let config = HamsConfig::default();
let hams = Hams::new(cancellation_token, &config).unwrap();

// Register probes
hams.alive_insert(manual_probe.clone()).expect("Failed to insert probe");

hams.start().unwrap();
```

### Manual Build / C-API Usage

If you are building the shared object directly for use in C/C++ or other languages without using Cargo's linking, you may need to adjust the `rpath` on macOS to ensure the library can be found relative to your binary.

**macOS `rpath` Adjustment:**

```bash
# Update the ID to include an rpath
install_name_tool -id @rpath/../lib/libhams.dylib target/debug/libhams.dylib

# Verification
otool -L target/debug/libhams.dylib
```

**Check Link Dependencies:**

```bash
otool -L <binary>
```

## Testing with Miri

To run tests with Miri (Undefined Behavior detector):

```bash
cargo watch -x 'miri test'
```

## Useful References

*   [Rust FFI Patterns](https://rust-unofficial.github.io/patterns/intro.html)
*   [Understanding RPATH](https://itwenty.me/posts/01-understanding-rpath/)
*   [Wrapping Unsafe C Libraries in Rust](https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65)
