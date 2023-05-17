FROM rust:1.69.0 AS chef
RUN cargo install cargo-chef
# FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
# FROM debian:buster-slim AS runtime
FROM ubuntu:lunar as runtime
WORKDIR app
COPY --from=build /app/target/release/sample-rust /usr/local/bin/
COPY --from=build /app/target/release/deps/libhams.so /usr/local/lib/
RUN ldconfig
ENTRYPOINT ["/usr/local/bin/sample-rust"]
CMD ["sample_rust", "--config", "/config/config.yaml", "start"]
