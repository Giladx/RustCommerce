# Next Actions

## Safe Immediate Actions
- Confirm that `src/main.rs` is the real runtime starting point before broad edits.
- Review `src/main.rs` first because it ranked highest for repo impact.
- Add explicit safe scripts for lint, typecheck, and test discovery in package metadata.
- Decide which validation commands are the canonical read-only checks for this repo.

## Manual Human-Review Actions
- Decide which commands should be supported as the official lint/typecheck/test/build interface.

## AI-Friendly Implementation Tasks
- Inspect `src/main.rs` first and identify its first local dependency.
- Turn the top critical files `src/main.rs`, `package.json`, `src/main.rs`, `src/api/routes.rs` into an ordered implementation map.
- Inspect package metadata and propose a minimal safe script set for lint, typecheck, and test discovery.
- Convert the top readiness blockers into a short, ordered implementation plan tied to concrete files.
- Summarize how `src/main.rs` depends on `bin_test.rs`, `src/admin/auth.rs`, `src/api/dto.rs`.