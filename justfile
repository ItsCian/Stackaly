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

# ── API ─────────────────────────────────
# Run the API
dev:
    cd backend && cargo run -p api

# Format + lint, same as CI will run
check:
    cd backend && cargo fmt --check
    cd backend && cargo clippy -- -D warnings

# Auto-fix formatting and the lints Clippy knows how to fix
fix:
    cd backend && cargo fmt
    cd backend && cargo clippy --fix --allow-dirty --allow-staged

# Tests commands
test:
    docker compose up -d --wait db-test
    cd backend && sea-orm-cli migrate up -u $TEST_DATABASE_URL
    cd backend && cargo test -p api
