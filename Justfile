default:
    @just --list

# Start an empty Postgres 19RC1 (port 5432); the app creates the schema via SeaORM migrations on startup
db:
    docker run --rm -d --name postgres-temporal-db -e POSTGRES_PASSWORD=password -p 127.0.0.1:5432:5432 postgres:19rc1-bookworm

# Run development server (needs `cargo install cargo-leptos` and `just db`)
serve:
    cargo leptos watch

# Build for production
build:
    cargo leptos build --release

# Clean build artifacts
clean:
    rm -rf dist docs
    cargo clean
