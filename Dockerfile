FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef

WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS buildcache
COPY --from=planner /app/recipe.json recipe.json

FROM buildcache as dev
# Install dev tools
RUN cargo install cargo-watch
# Grab dependencies into target
RUN cargo chef cook --recipe-path recipe.json
# This enables us ot keep /app/target inside the container when we mount /app from host
VOLUME /app/target
# So we hold the daemon mode of docker run
CMD sleep infinity

FROM buildcache AS buildcacherelease
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application

FROM buildcacherelease AS build
COPY . .
RUN cargo build --release -p hams

# We do not need the Rust toolchain to run the binary!
# FROM debian:bookworm-slim as runtime
# https://github.com/GoogleContainerTools/distroless
FROM scratch AS runtime
COPY --from=build /app/target/release/libhams.so /usr/local/lib/
