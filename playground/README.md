# Run build on test

Based on code here: https://adventures.michaelfbryan.com/posts/ffi-safe-polymorphism-in-rust/

This is my run for automated test on save

    cargo watch -x "miri test -- --nocapture"


# Plan

We have implemented HealthWrapper to allow Arc<Mutex<>> of the actual HealthCheck implementation.

We have HealthCheck to implement the FFI safe abstract base class

We have OwnedHealthCheck to handle the Rust impl across the FFI boundaries to control/manage the HealthCheck.

How do we merge all above into a single usage pattern.
