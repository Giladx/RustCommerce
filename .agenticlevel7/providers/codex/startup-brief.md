# Codex Startup Brief

## Project: rust-cli
Framework: Rust Binary, Rust, managed by cargo
Purpose: e-commerce + admin/dashboard + user authentication + API backend + ML/AI

## Approach: Direct Implementation
Workstreams are pre-scoped. Pick an agent, read its files, implement changes.

## Implementation Queue
2. **Schema (Data Model Agent)**
   Files: migrations, src/infra/db/mod.rs, src/infra/db/pool.rs, src/infra/db/store.rs, migrations/20250201120000_fix_store_timestamps.sql
   Run: cargo run
3. **Gatekeeper (Authentication Agent)**
   Files: src/admin/auth.rs, src/mw/auth.rs
   Run: cargo run
4. **Interface (API Layer Agent)**
   Files: src/api/routes.rs, src/api/dto.rs, src/api/error.rs, src/api/mod.rs, src/api/otp_service.rs
   Run: cargo run
7. **Launchpad (Build & Deploy Agent)**
   Files: src/infra/db/mod.rs, src/infra/db/pool.rs, src/infra/db/store.rs, src/infra/mod.rs, src/infra
   Run: cargo build
8. **Operator (Admin & Dashboard Agent)**
   Files: src/admin/auth.rs, src/admin/mod.rs, src/admin
   Run: cargo run
9. **Agent (File Storage Agent)**
   Files: .sentry-native/attachments
   Run: cargo run
10. **Agent (Monitoring & Observability Agent)**
   Files: .sentry-native, .sentry-native/attachments, .sentry-native/completed, .sentry-native/new, .sentry-native/pending
   Run: cargo run
11. **Sentinel (Testing Agent)**
   Files: src/domain/cart/tests.rs, src/domain/money/tests.rs, src/domain/order/tests.rs, src/domain/store/tests.rs, bin_test.rs
   Run: cargo test

## Critical Files
- `src/main.rs`
- `package.json`
- `src/main.rs`
- `src/api/routes.rs`
- `Cargo.toml`
- `src/admin/auth.rs`
- `src/admin/mod.rs`
- `src/api/dto.rs`

## How To Start
1. Open `migrations`
2. Implement the Schema (Data Model Agent) workstream
3. Validate: `cargo run`
4. Move to the next agent in the queue