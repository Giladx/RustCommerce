# Codex Brief

This repository is best treated as a git-based codebase with concrete starting points in `src/main.rs`, `package.json`, `src/api/routes.rs`, `Cargo.toml`.

Runtime model: The clearest starting point is `src/main.rs`, but the downstream request flow still needs manual confirmation.

## Where To Start
- Open `src/main.rs` first.
- Inspect `src/main.rs` because entrypoint.
- Inspect `package.json` because retained as a critical file because repo metadata still points to it.
- Inspect `src/main.rs` because retained as a critical file because repo metadata still points to it.
- Inspect `src/api/routes.rs` because retained as a critical file because repo metadata still points to it.

## Likely Execution Path
- `src/main.rs`
- Confidence: low

## Real Repo Risks
- Some likely entrypoints have shallow traced execution paths: src/main.rs.
- Configuration files are operationally important here, especially package.json.
- Without explicit lint/typecheck/test/build entrypoints, AI loops become slower and riskier.
- Lack of test structure reduces confidence in automated changes.
- Large files, deep trees, or a wide root increase the chance of imprecise AI changes.

## Concrete First Tasks
- Inspect `src/main.rs` first and identify its first local dependency.
- Turn the top critical files `src/main.rs`, `package.json`, `src/main.rs`, `src/api/routes.rs` into an ordered implementation map.
- Inspect package metadata and propose a minimal safe script set for lint, typecheck, and test discovery.
- Convert the top readiness blockers into a short, ordered implementation plan tied to concrete files.
- Summarize how `src/main.rs` depends on `bin_test.rs`, `src/admin/auth.rs`, `src/api/dto.rs`.

## Operating Notes
- Prefer edits in `src/main.rs`, `package.json`, `src/main.rs`, `src/api/routes.rs` before wandering into lower-ranked files.
- Re-run safe validation commands only if they exist: detected=0.
- Treat unresolved imports as possible graph blind spots: 0.