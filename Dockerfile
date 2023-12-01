FROM rust:slim-bullseye as build
WORKDIR /app

COPY . .
RUN apt clean && apt update && apt install -y pkg-config libssl-dev gcc-multilib libpq-dev libudev-dev
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --release
CMD ["./target/release/rust-websocket-server"]
