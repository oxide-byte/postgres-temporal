default:
    @just --list

# Start Postgres 19RC1 with the temporal schema (port 5432)
db:
    docker build -t postgres-temporal-db infrastructure
    docker run --rm -d --name postgres-temporal-db -p 127.0.0.1:5432:5432 postgres-temporal-db

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
