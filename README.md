# SanctumChat Service

A rust based backend providing a RESTful api used to manage:
- Users
- FriendRequests
- Friends
- Messages

It also contains a WebSocket endpoint to establish a websocket connection.


# Setup

- Install rust: https://www.rust-lang.org/tools/install
- Install libraries required for dependencies:
    - `sudo apt install libpq-dev` # Required to build diesel ORM
- Install diesel ORM CLI tools: https://diesel.rs/guides/getting-started


# First Start

- Start dependencies: `make start-dependencies`
- Run migrations: `make run-migrations` # This requires a set-up `pg_hba.conf` that allows password-less localhost login.
- Rename `.env.sample` to `.env` (`mv .env.sample .env`)
- Run cargo `cargo run`

# Environment Variables

|Name|Description|
|-----|-----|
|DATABASE_URL|url to database|
|HASHING_KEY|Hashing salt|
|CORS_ORIGIN|Cors origin|
|RUST_LOG|Log level|


# Tests

This project contains a selfmade python service test framework to test the websocket connection.

Execute unit tests with `make run-unit-tests`
Execute service tests with `make run-service-tests`
