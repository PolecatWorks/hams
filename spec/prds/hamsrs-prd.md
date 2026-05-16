# hamsrs/ (Rust Bindings) PRD

Overview: hamsrs/ exposes hams/ functionality to Rust consumers with ergonomic idiomatic wrappers and safe FFI boundaries.

Goals:
- Provide idiomatic Rust API with safety wrappers around FFI.
- Keep parity with core features and maintain tests.
- Publish a crates.io package for v1.0.

User Stories & Acceptance:
- As a Rust Integrator, install via crates.io and run sample in <10 minutes.
- FFI boundaries must be covered by integration tests and fuzzing where applicable.

Security:
- Enforce memory-safety checks at FFI boundaries and CI fuzzing for inputs.

PRD version: 0.1
