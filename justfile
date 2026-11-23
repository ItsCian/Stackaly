# Load .env into every recipe (DATABASE_URL, etc.)
set dotenv-load

# Show all commands (runs when you type just `just`)
default:
    @just --list

# ── Database ────────────────────────────
# Start Postgres in the background
db-up:
    docker compose up -d

# Stop Postgres (data kept)
db-down:
    docker compose down

# ── Migrations ──────────────────────────
# Apply pending migrations
migrate:
    cd backend && sea-orm-cli migrate up

# Show migration status
migrate-status:
    cd backend && sea-orm-cli migrate status

# Create entities
entities:
    cd backend && sea-orm-cli generate entity -o api/src/entities --with-serde both

# ── Dev ─────────────────────────────────
# Run API + front end together (Ctrl+C stops both)
dev:
    #!/usr/bin/env bash
    # Kill every child process when this script exits, so Ctrl+C
    # doesn't leave the API running in the background
    trap 'kill 0' EXIT
    (cd backend && cargo run -p api) &
    (cd frontend && bun run dev) &
    # Wait for both, otherwise the script would exit immediately
    wait

# Run only the API
api:
    cd backend && cargo run -p api

# Run only the front end (Vite dev server, proxies /api to the API)
web:
    cd frontend && bun run dev

# ── Front end ───────────────────────────
# Install front-end dependencies (after a clone or a package.json change)
web-install:
    cd frontend && bun install

# Production build: type-check + Vite build into frontend/dist
web-build:
    cd frontend && bun run build

# ── Quality ─────────────────────────────
# Format + lint + type-check, same as CI will run
check:
    cd backend && cargo fmt --check
    cd backend && cargo clippy --workspace --all-targets --locked -- -D warnings
    cd frontend && bunx biome ci .
    cd frontend && bunx tsc -b

# Auto-fix formatting and the lints each tool knows how to fix
fix:
    cd backend && cargo fmt
    cd backend && cargo clippy --fix --allow-dirty --allow-staged
    cd frontend && bun run format

# ── Tests ───────────────────────────────
# Run backend tests against the test database
test:
    docker compose up -d --wait db-test
    cd backend && sea-orm-cli migrate up -u $TEST_DATABASE_URL
    cd backend && cargo test -p api
