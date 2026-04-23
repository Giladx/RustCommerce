# MEMORY.md

> This is the project's bootstrap memory. It captures what the agent needs to know
> to be effective from the first interaction. Update this as the project evolves.

## Project Identity

- **Type:** rust-cli
- **Stack:** Rust Binary, Rust, managed by cargo
- **Purpose:** e-commerce + admin/dashboard + user authentication + API backend + ML/AI
- **Analyzed:** 2026-03-21T07:22:04.683Z

## Architecture Decisions

- Data layer: migrations
- Auth: src/admin/auth.rs, src/admin/auth.rs

## What Matters Most

These are the files with the highest impact. Changes here ripple everywhere.

1. `src/main.rs` — entrypoint
1. `package.json` — retained as a critical file because repo metadata still points to it
1. `src/main.rs` — retained as a critical file because repo metadata still points to it
1. `src/api/routes.rs` — retained as a critical file because repo metadata still points to it
1. `Cargo.toml` — retained as a critical file because repo metadata still points to it
1. `src/admin/auth.rs` — retained as a critical file because repo metadata still points to it
1. `src/admin/mod.rs` — retained as a critical file because repo metadata still points to it
1. `src/api/dto.rs` — retained as a critical file because repo metadata still points to it

## Known Risks

- Some likely entrypoints have shallow traced execution paths: src/main.rs.
- Configuration files are operationally important here, especially package.json.
- [warning] Safe command story is unclear: Without explicit lint/typecheck/test/build entrypoints, AI loops become slower and riskier.
- [warning] Test and validation story is unclear: Lack of test structure reduces confidence in automated changes.
- [warning] Repository surface area is harder to control than necessary: Large files, deep trees, or a wide root increase the chance of imprecise AI changes.

## Workstream Map

The project breaks down into these independent workstreams:

- **Configuration** (medium risk) — Project-wide configuration and environment
- **Data Model** (high risk) — Data persistence and schema management
- **Authentication** (high risk) — User authentication and session management
- **API Layer** (high risk) — API endpoints, schemas, and data fetching
- **Routing & Endpoints** (high risk) — Request routing, controllers, and page rendering
- **SEO & Indexing** (medium risk) — Search engine optimization and crawlability
- **Build & Deploy** (medium risk) — Build pipeline, CI/CD, and deployment
- **Admin & Dashboard** (medium risk) — Administrative interface and management tools
- **File Storage** (medium risk) — File upload, storage, and media handling
- **Monitoring & Observability** (low risk) — Observability, logging, and health checks
- **Testing** (low risk) — Test infrastructure and coverage

## Readiness Notes

**Blockers to address:**
- [warning] Safe command story is unclear: Without explicit lint/typecheck/test/build entrypoints, AI loops become slower and riskier.
- [warning] Test and validation story is unclear: Lack of test structure reduces confidence in automated changes.
- [warning] Repository surface area is harder to control than necessary: Large files, deep trees, or a wide root increase the chance of imprecise AI changes.

Architecture findings: 2
Code health findings: 7