# Claude Brief

Plan work around the repository evidence, not generic folder heuristics. The highest-signal files are `src/main.rs`, `package.json`, `src/main.rs`, `src/api/routes.rs`, `Cargo.toml`.
Runtime shape: The clearest starting point is `src/main.rs`, but the downstream request flow still needs manual confirmation.

## Recommended Sequencing
- Inspect `src/main.rs` first because no downstream import chain was found yet.
- Inspect `src/main.rs` early because it scored highly for repo impact: entrypoint.
- Inspect `package.json` early because it scored highly for repo impact: retained as a critical file because repo metadata still points to it.
- Inspect `src/main.rs` early because it scored highly for repo impact: retained as a critical file because repo metadata still points to it.

## Inspection Targets
- Entrypoints: `src/main.rs`
- Critical files: `src/main.rs`, `package.json`, `src/main.rs`, `src/api/routes.rs`, `Cargo.toml`, `src/admin/auth.rs`
- Important directories: `.gemini`, `.sentry-native`, `migrations`, `Repo`, `src`, `src/api`
- Primary traced path: `src/main.rs`

## Planning Constraints
- Keep changes anchored to the ranked modules: `bin_test.rs`, `src/admin/auth.rs`, `src/api/dto.rs`, `src/api/error.rs`, `src/api/mod.rs`.
- Watch for graph blind spots from unresolved imports: 0.
- Safe command coverage is still limited to 0 detected command(s).

## High-Priority Risks
- Some likely entrypoints have shallow traced execution paths: src/main.rs.
- Configuration files are operationally important here, especially package.json.
- [warning] Safe command story is unclear: Without explicit lint/typecheck/test/build entrypoints, AI loops become slower and riskier.
- [warning] Test and validation story is unclear: Lack of test structure reduces confidence in automated changes.
- [warning] Repository surface area is harder to control than necessary: Large files, deep trees, or a wide root increase the chance of imprecise AI changes.