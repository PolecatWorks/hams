# Run build on test

Based on code here: https://adventures.michaelfbryan.com/posts/ffi-safe-polymorphism-in-rust/

This is my run for automated test on save

    cargo watch -x "miri test -- --nocapture"
