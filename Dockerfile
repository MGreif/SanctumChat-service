FROM rust:slim-bullseye as build

RUN USER=root cargo new --bin rust-websocket-server

WORKDIR /rust-websocket-server

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN apt clean && apt update && apt install -y pkg-config libssl-dev gcc-multilib libpq-dev libudev-dev

# Build only the dependencies to cache them
RUN cargo build --release

# Copy the source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/rust_websocket_server*
RUN cargo build --release

#RUN cargo install diesel_cli --no-default-features --features postgres
#RUN cargo build --release

FROM debian:bullseye

RUN apt-get update && apt install -y openssl libpq-dev

RUN useradd -u 8877 appuser

USER appuser

COPY --from=build --chown=appuser:appuser /rust-websocket-server/target/release/rust-websocket-server /usr/src/rust-websocket-server
# COPY --from=build /holodeck/target/release/holodeck/target/x86_64-unknown-linux-musl/release/holodeck .

CMD ["/usr/src/rust-websocket-server"]
