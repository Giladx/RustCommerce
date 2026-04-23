# Claude Code Startup Brief

## Project: rust-cli
Framework: Rust Binary, Rust, managed by cargo
Purpose: e-commerce + admin/dashboard + user authentication + API backend + ML/AI

## Approach: Staged Reasoning
This project has been decomposed into workstreams with dedicated agents.
Work through them in order, planning before acting.

## Agent Queue
1. **Foundation (Configuration Agent)** (config)
   Own project-wide configuration. Ensure environment variables, compiler settings, linter rules, and framework config are consistent and documented.
   Files: Cargo.toml, .env, .gemini/settings.json (+1)
3. **Gatekeeper (Authentication Agent)** (auth)
   Secure the authentication and authorization surface. Verify login flows, session management, protected routes, and permission boundaries.
   Files: src/admin/auth.rs, src/mw/auth.rs
5. **Navigator (Routing & Endpoints Agent)** (routing)
   Own the request routing surface of this rust-cli. Map all routes, verify layouts render correctly, and ensure the routing tree is complete and consist
   Files: src/api/error.rs, src/domain/order/error.rs, src/api/routes
6. **Lighthouse (SEO & Indexing Agent)** (seo-indexing)
   Ensure this e-commerce + admin/dashboard + user authentication + API backend + ML/AI project is properly indexed by search engines. Verify sitemaps, r
   Files: migrations/20260117_extend_product_catalog.sql
7. **Launchpad (Build & Deploy Agent)** (build-deploy)
   Own the build pipeline and deployment configuration. Verify builds succeed, assets are optimized, and deployment targets are correctly configured.
   Files: src/infra/db/mod.rs, src/infra/db/pool.rs, src/infra/db/store.rs (+3)
8. **Operator (Admin & Dashboard Agent)** (admin-dashboard)
   Own the administrative interface. Ensure admin routes are protected, CRUD operations work correctly, and management workflows are complete.
   Files: src/admin/auth.rs, src/admin/mod.rs, src/admin
9. **Agent (File Storage Agent)** (storage)
   Own the file storage workstream. Map all related files, verify correctness, and ensure this area is well-maintained.
   Files: .sentry-native/attachments
10. **Agent (Monitoring & Observability Agent)** (monitoring)
   Own the monitoring & observability workstream. Map all related files, verify correctness, and ensure this area is well-maintained.
   Files: .sentry-native, .sentry-native/attachments, .sentry-native/completed (+2)

## Control Points
- `src/main.rs` (entrypoint): Runtime entry point
- `Cargo.toml` (framework-config): Rust crate manifest

## Risks
- Some likely entrypoints have shallow traced execution paths: src/main.rs.
- Configuration files are operationally important here, especially package.json.
- [warning] Safe command story is unclear: Without explicit lint/typecheck/test/build entrypoints, AI loops become slower and riskier.
- [warning] Test and validation story is unclear: Lack of test structure reduces confidence in automated changes.
- [warning] Repository surface area is harder to control than necessary: Large files, deep trees, or a wide root increase the chance of imprecise AI changes.

## How To Start
1. Read the project model and workstream files
2. Pick the first agent in the queue
3. Read its agent brief in `.agenticlevel7/agents/`
4. Understand scope and dependencies before making changes
5. Validate with the agent's recommended commands