# Stackaly

Stackaly is a modular, multi-tenant web platform that helps small businesses
reconcile their daily sales. It sits on top of the tools a shop already uses
(POS systems, card terminal, cash drawers), compares their numbers at the end
of each day, and flags any discrepancy before it becomes a mystery.

The first module, **daily reconciliation**, was built for a real client: an
Italian concept store with a shop, a small market, a takeaway counter and a
trattoria, each using a different checkout system.

> Final-year project (TFE), EPHEC Louvain-la-Neuve, 2026–2027.

<!-- TODO (STA-41): add the CI status badge here once the workflow exists -->

## Tech stack

| Layer      | Technology                                   |
| ---------- | -------------------------------------------- |
| Back-end   | Rust, Axum, SeaORM                           |
| Database   | PostgreSQL                                   |
| Front-end  | Svelte                                       |
| CI/CD      | GitHub Actions                               |
| Hosting    | VPS in the EU                                |

## Repository layout

```
stackaly/
├── backend/     # Rust Cargo workspace (API, entities, migrations)
├── frontend/    # Svelte app
└── .github/     # PR template, CI workflows
```

## Getting started

### Prerequisites

- Rust: TODO (pin the version, e.g. with a `rust-toolchain.toml`)
- Node.js: TODO
- PostgreSQL: TODO

### Environment variables

Copy the example file and fill in your local values:

```bash
cp .env.example .env
```

<!-- TODO: list each variable and what it does -->

### Run the back-end

<!-- TODO (STA-44) -->

### Run the front-end

<!-- TODO (STA-46) -->

## Workflow

Work is tracked in Linear. Each issue gets its own branch, using the name Linear
generates (e.g. `cianjonesth/sta-40-...`). Changes reach `main` only through a
pull request.