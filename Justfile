default:
    @just --list

# Start an empty Postgres 19RC1 (port 5432); the app creates the schema via SeaORM migrations on startup
db:
    docker run --rm -d --name postgres-temporal-db -e POSTGRES_PASSWORD=password -p 127.0.0.1:5432:5432 postgres:19rc1-bookworm

# Run development server (needs `cargo install cargo-leptos` and `just db`)
serve:
    cargo leptos watch

# Stop TCP listeners on the app port (or a specified port).
kill port="3000":
    #!/usr/bin/env bash
    set -euo pipefail
    export PORT={{quote(port)}}
    port=$PORT
    if [[ ! "$port" =~ ^[0-9]{1,5}$ ]]; then
        printf 'Invalid port: %s (expected 1-65535).\n' "$port" >&2
        exit 2
    fi
    port_number=$((10#$port))
    if (( port_number < 1 || port_number > 65535 )); then
        printf 'Invalid port: %s (expected 1-65535).\n' "$port" >&2
        exit 2
    fi
    if ! command -v lsof >/dev/null 2>&1; then
        printf 'The lsof command is required to find TCP listeners.\n' >&2
        exit 1
    fi
    pids=$(lsof -t -nP -iTCP:"$port_number" -sTCP:LISTEN 2>/dev/null | sort -u || true)
    if [[ -z "$pids" ]]; then
        printf 'No TCP listeners found on port %s.\n' "$port_number"
        exit 0
    fi
    while IFS= read -r pid; do
        kill -TERM "$pid"
        printf 'Sent SIGTERM to PID %s listening on TCP port %s.\n' "$pid" "$port_number"
    done <<< "$pids"

# Build for production
build:
    cargo leptos build --release

# Clean build artifacts
clean:
    rm -rf dist docs
    cargo clean
