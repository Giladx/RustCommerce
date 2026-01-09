# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` wires the Axum HTTP server, pulling request handlers from `src/api`. Those handlers lean on DTOs (`src/api/dto.rs`), repositories (`src/api/repository`) and shared error types. Core domain logic lives under `src/domain` with bounded contexts such as `product`, `cart`, `order`, and supporting invariants in `money` and `discount`. Infrastructure concerns stay inside `src/infra/db`, which exposes SQLx pools used throughout the API. Database migrations belong to `/migrations` (timestamped SQL files); run them before starting the server. Generated binaries land in `target/`, so never edit anything there.

## Build, Test & Development Commands
- `cargo check` – fast compile-pass to catch typing and dependency issues.
- `cargo fmt && cargo clippy --all-targets` – enforce formatting and linting before pushing.
- `cargo test` – executes unit and integration tests; add `-- --nocapture` while debugging assertions.
- `cargo run` – boots the HTTP server using `.env` values loaded by `dotenvy`.
- `sqlx migrate run` – applies SQL migrations against the `DATABASE_URL`; use `sqlx migrate revert` to roll back the last step.

## Coding Style & Naming Conventions
Stick to the default rustfmt style (4-space indentation, trailing commas, grouped imports). Modules, files, functions, and variables use `snake_case`; types and traits stay in `PascalCase`. Repositories, DTOs, and route handlers should be suffixed accordingly (e.g., `CartRepository`, `CreateProductRequest`). Keep public APIs minimal, re-exporting only through each `mod.rs` to preserve clean layering between `api`, `domain`, and `infra`.

## Testing Guidelines
Prefer focused unit tests colocated in each module via `#[cfg(test)] mod tests`. Integration scenarios that touch HTTP layers or the database belong under a future `tests/` directory using `#[tokio::test]`. Name tests descriptively (`test_updates_discount_when_threshold_met`) and set up isolated data with SQLx transactions so runs stay hermetic. Aim for coverage on every domain rule and serialization boundary.

## Commit & Pull Request Guidelines
Follow the existing imperative, scope-first style (`Add cart repository`, `Fix order total rounding`). Each pull request should explain the problem, outline the solution, list testing evidence (`cargo test`, migration output), and link any related issue. Include screenshots or sample JSON payloads when touching HTTP contracts, and flag schema changes so reviewers can plan database migrations.

## Configuration & Security Notes
Provide a `.env` with `DATABASE_URL` and any feature flags; never commit secrets. Keep migrations idempotent and validated with `sqlx migrate run --dry-run` before deployment. If pulling sample data, store it under `Repo/` or a developer-only fixture folder to avoid shipping PII.
