# Stackaly

Stackaly is a modular, multi-tenant web platform that helps small businesses
reconcile their daily sales. It sits on top of the tools a shop already uses
(POS systems, card terminal, cash drawers), compares their numbers at the end
of each day, and flags any discrepancy before it becomes a mystery.

The first module, **daily reconciliation**, was built for a real client: an
Italian concept store with a shop, a small market, a takeaway counter and a
trattoria, each using a different checkout system.

> Final-year project (TFE), EPHEC Louvain-la-Neuve, 2026–2027.

[![CI](https://github.com/ItsCian/Stackaly/actions/workflows/ci.yml/badge.svg)](https://github.com/ItsCian/Stackaly/actions/workflows/ci.yml)

## Tech stack

| Layer      | Technology                                   |
| ---------- | -------------------------------------------- |
| Back-end   | Rust, Axum, SeaORM                           |
| Database   | PostgreSQL 18                                |
| Front-end  | Svelte, Bun                                  |
| CI/CD      | GitHub Actions                               |
| Hosting    | VPS in the EU                                |

## Repository layout

```
stackaly/
├── backend/               # Rust Cargo workspace
│   ├── api/               # Axum API (src/ + integration tests in tests/)
│   └── migration/         # SeaORM database migrations
├── frontend/              # Svelte app
├── .github/               # PR template, CI workflows
├── docker-compose.yml     # PostgreSQL for development and tests
└── justfile               # Project commands (run `just` to list them)
```

## Getting started

### Prerequisites

- [Rust](https://rustup.rs): TODO (pin the version, e.g. with a `rust-toolchain.toml`)
- [Docker](https://www.docker.com) with Docker Compose: runs PostgreSQL, no local install needed
- [just](https://github.com/casey/just): `brew install just` or `cargo install just`
- [sea-orm-cli](https://www.sea-ql.org/SeaORM/) 2.x: `cargo install sea-orm-cli@2.0.4 --locked`
- [Bun](https://bun.sh): for the front-end

### Environment variables

Copy the example file and fill in your local values:

```bash
cp .env.example .env
```

| Variable             | Used by        | Description                                              |
| -------------------- | -------------- | -------------------------------------------------------- |
| `POSTGRES_USER`      | Docker Compose | Database user                                            |
| `POSTGRES_PASSWORD`  | Docker Compose | Database password                                        |
| `POSTGRES_DB`        | Docker Compose | Name of the development database                         |
| `POSTGRES_PORT`      | Docker Compose | Host port of the development database (default `5432`)   |
| `POSTGRES_TEST_PORT` | Docker Compose | Host port of the test database (default `5433`)          |
| `DATABASE_URL`       | API, migrations| Connection string of the development database            |
| `TEST_DATABASE_URL`  | Tests          | Connection string of the test database (port `5433`)     |
| `HOST`               | API            | Address the API listens on (default `127.0.0.1`)         |
| `PORT`               | API            | Port the API listens on (default `3000`)                 |
| `RUST_LOG`           | API            | Log level, e.g. `info` or `info,sqlx=warn`               |

The connection strings must match the `POSTGRES_*` values above.
`.env` is ignored by Git; only `.env.example` is committed.

### Run the back-end

```bash
just db-up      # start PostgreSQL (development database)
just migrate    # apply the database migrations
just dev        # start the API on http://127.0.0.1:3000
```

Check that it works:

```bash
curl http://127.0.0.1:3000/health
# {"status":"ok","database":"ok"}
```

### Run the tests

```bash
just test
```

This starts a separate, in-memory PostgreSQL container (`db-test`), applies the
migrations to it and runs the integration tests. Your development data is never
touched.

### Run the front-end

<!-- TODO (STA-46) -->

## Commands

All project commands are in the `justfile`. Run `just` to list them.

| Command               | What it does                                         |
| --------------------- | ---------------------------------------------------- |
| `just db-up`          | Start the development database                       |
| `just db-down`        | Stop it (data is kept)                               |
| `just migrate`        | Apply pending migrations                             |
| `just migrate-status` | Show which migrations are applied                    |
| `just dev`            | Run the API                                          |
| `just test`           | Run the integration tests on the test database       |
| `just check`          | Check formatting and lints (same as CI)              |
| `just fix`            | Auto-fix formatting and the lints Clippy can fix     |

## API

| Method   | Route             | Description                           |
| -------- | ----------------- | ------------------------------------- |
| `GET`    | `/health`         | API and database status               |
| `POST`   | `/companies`      | Create a company                      |
| `GET`    | `/companies`      | List companies                        |
| `GET`    | `/companies/{id}` | Get one company                       |
| `PATCH`  | `/companies/{id}` | Update the fields sent                |
| `DELETE` | `/companies/{id}` | Delete a company                      |

Every error uses the same JSON format:

```json
{ "code": "not_found", "message": "Resource not found" }
```

| Code               | Status          | Meaning                                         |
| ------------------ | --------------- | ----------------------------------------------- |
| `invalid_request`  | 400, 415, 422   | Malformed request (bad JSON, missing field…)    |
| `validation_error` | 400             | A business rule was not met                     |
| `not_found`        | 404             | The resource or route does not exist            |
| `conflict`         | 409             | Conflicts with existing data (e.g. duplicate VAT) |
| `internal_error`   | 500             | Server error (details are logged, not returned) |

## Workflow

Work is tracked in Linear. Each issue gets its own branch, using the name Linear
generates (e.g. `cianjonesth/sta-40-...`). Changes reach `main` only through a
pull request, merged with **squash and merge**.

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org)
(`feat(api): ...`, `fix(db): ...`, `test(api): ...`) and mention the Linear issue
