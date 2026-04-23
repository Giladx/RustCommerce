# FILES.md

> The file system is the agent's workspace. This map shows what matters and why.

## Entry Points

The runtime starts here. Read these first to understand the application flow.

- `src/main.rs` — entrypoint (importance: 2)

## Directory Map

- `.gemini/` — unknown: unclassified directory
- `.sentry-native/` — unknown: unclassified directory
- `migrations/` — source: database migrations
- `Repo/` — unknown: unclassified directory
- `src/` — source: common source directory
- `src/api/` — unknown: unclassified directory

## Critical Files

Ranked by impact. Changes to these files affect the most other files.

### `src/main.rs`
- Category: entrypoint
- Impact score: 5
- entrypoint

### `package.json`
- Category: manifest
- Impact score: 40
- retained as a critical file because repo metadata still points to it

### `src/main.rs`
- Category: entrypoint
- Impact score: 39
- retained as a critical file because repo metadata still points to it

### `src/api/routes.rs`
- Category: entrypoint
- Impact score: 38
- retained as a critical file because repo metadata still points to it

### `Cargo.toml`
- Category: entrypoint
- Impact score: 37
- retained as a critical file because repo metadata still points to it

### `src/admin/auth.rs`
- Category: entrypoint
- Impact score: 36
- retained as a critical file because repo metadata still points to it

### `src/admin/mod.rs`
- Category: entrypoint
- Impact score: 35
- retained as a critical file because repo metadata still points to it

### `src/api/dto.rs`
- Category: entrypoint
- Impact score: 34
- retained as a critical file because repo metadata still points to it

### `src/api/error.rs`
- Category: entrypoint
- Impact score: 33
- retained as a critical file because repo metadata still points to it

### `src/api/mod.rs`
- Category: entrypoint
- Impact score: 32
- retained as a critical file because repo metadata still points to it

### `src/api/otp_service.rs`
- Category: entrypoint
- Impact score: 31
- retained as a critical file because repo metadata still points to it

### `src/api/repository/category_repository.rs`
- Category: entrypoint
- Impact score: 30
- retained as a critical file because repo metadata still points to it

### `src/api/repository/checkout_repository.rs`
- Category: entrypoint
- Impact score: 29
- retained as a critical file because repo metadata still points to it

### `src/api/repository/customer_repository.rs`
- Category: entrypoint
- Impact score: 28
- retained as a critical file because repo metadata still points to it

### `src/api/repository/customer_service_repository.rs`
- Category: entrypoint
- Impact score: 27
- retained as a critical file because repo metadata still points to it

## Key Modules

**`bin_test.rs`** — rust test (13 lines, 0 imports)
- Lines: 13 | Imports: 0 | Imported by: 0

**`src/admin/auth.rs`** — rust script (122 lines, 0 imports)
- Lines: 122 | Imports: 0 | Imported by: 0

**`src/api/dto.rs`** — rust script (109 lines, 0 imports)
- Lines: 109 | Imports: 0 | Imported by: 0

**`src/api/error.rs`** — rust script (56 lines, 0 imports)
- Lines: 56 | Imports: 0 | Imported by: 0

**`src/api/mod.rs`** — rust script (6 lines, 0 imports)
- Lines: 6 | Imports: 0 | Imported by: 0

**`src/api/otp_service.rs`** — rust script (258 lines, 1 imports)
- Lines: 258 | Imports: 1 | Imported by: 0

**`src/api/repository/category_repository.rs`** — rust script (88 lines, 1 imports)
- Lines: 88 | Imports: 1 | Imported by: 0

**`src/api/repository/checkout_repository.rs`** — rust script (607 lines, 1 imports)
- Lines: 607 | Imports: 1 | Imported by: 0

**`src/api/repository/customer_repository.rs`** — rust script (80 lines, 1 imports)
- Lines: 80 | Imports: 1 | Imported by: 0

**`src/api/repository/customer_service_repository.rs`** — rust script (152 lines, 1 imports)
- Lines: 152 | Imports: 1 | Imported by: 0

**`src/api/repository/mod.rs`** — rust script (10 lines, 0 imports)
- Lines: 10 | Imports: 0 | Imported by: 0

**`src/api/repository/product_attribute_repository.rs`** — rust script (90 lines, 1 imports)
- Lines: 90 | Imports: 1 | Imported by: 0

## Configuration Files


## Execution Paths

Traced runtime flows through the codebase:

**src/main.rs** (low confidence)
`src/main.rs`
Reason: No downstream local import chain was found from src/main.rs.
