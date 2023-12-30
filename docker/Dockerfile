FROM rust:slim-bullseye as build

RUN USER=root cargo new --bin anonchat-backend

WORKDIR /anonchat-backend

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN apt clean && apt update && apt install -y pkg-config libssl-dev gcc-multilib libpq-dev libudev-dev

# Build only the dependencies to cache them
RUN cargo build --release

# Copy the source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/anonchat_backend*
RUN cargo build --release


FROM debian:bullseye

RUN apt-get update && apt install -y openssl libpq-dev

RUN useradd -u 8877 appuser

USER appuser

COPY --from=build --chown=appuser:appuser /anonchat-backend/target/release/anonchat-backend /usr/src/anonchat-backend

CMD ["/usr/src/anonchat-backend"]
