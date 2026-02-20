

# Run miri test

    cargo watch -x "miri test"

    MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test

# Run Code
(from top dir)

    cargo watch -x "run -- --config sample-rust-shared/test_data/config.yaml start"
