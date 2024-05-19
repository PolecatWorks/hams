# HaMS
Health Monitoring System

[![Rust](https://github.com/PolecatWorks/hams/actions/workflows/rust.yml/badge.svg)](https://github.com/PolecatWorks/hams/actions/workflows/rust.yml)

A library written in rust to implement kubernetes lifecycle interfaces. It is written as a shared object so that it can be utilised by many languages.

Typical usages are:
* Rust
* Python
* C/C++
* Java/Kotlin
* Node


# Test with Miri

Run the command

    cargo watch -x 'miri test'

# Run and test

Run the command

    cargo watch -x "run -- --config sample-rust/test_data/config.yaml start"



# ToDo

List of topics that need work

* [x] Build core library as shared object
  * [x] Logging over FFI
  * [x] Web serving a health endopint
  * [ ] Tests against webservice
  * [x] Alive http check
  * [x] Ready http check
* [x] Validate CAPI calls to protect against stupid CAPI errors (null, etc)
* [x] Wrap CAPI interface with rust interface and use of Result<>
* [ ] Helm sample chart using APIs
* [x] Rust bindings
  * [x] Rust cli program to demonstrate usage
* [x] Python bindings
* [x] NodeJS bindings
* [x] Java/Kotlin bindings
  * [x] Consideration for future: https://openjdk.org/projects/panama/ and https://github.com/openjdk/jextract
* [x] C/C++ bindings
  * [x] Show usage of C logging from Rust SO
* [ ] Support for prometheus
* [x] Shutdown sequences
  * [*] Should Hams include shutdown or should that be provided ONLY by main loop. HaMS provides the signal handling and raises the shutdown_callback BUT it is up to the app on how to respond.
  * [x] How to map a shutdown signal from HaMS to main loop to enable a shutdown API
* [ ] Show an example with header propagation to follow on calls: https://istio.io/latest/docs/tasks/observability/distributed-tracing/overview/
* [*] Create callback for health endpoint to indicate the service is to be shut down
* [ ] Check and update the shutdown hook so that it completes the shutdown process before yeilding the shutdown http API
* [*] Remove shutdown trigger on main thread and use ONLY the health thread as trigger for shutdowns via callback.
  * Health can pickup signals and then make shutdown callback
  * Health does not shutdown based on signals only on command from main thread
  * [*] SIG interrupts are detected by hams but not acted upon.. Except to make call against shutdown callback.
* [ ] Create a rust lib to put nice Ergonomics around the rust FFI interface and expose as a library that is used by sample-rust or sample-rust2

# Useful Reference
List of useful sites to review
* https://rust-unofficial.github.io/patterns/intro.html
* https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65
*

# Check Link Dependencies
Check the link dependencies for a given binary ie dylib on osx or .so on linux

    otool -L <binary>

# Autotools

Build rust and other libraries and system install with autotools.
