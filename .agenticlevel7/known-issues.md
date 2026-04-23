# Known Issues

## Architecture And Code Health
- [high] Oversized files detected: Large files make navigation and AI-assisted editing slower and less reliable.
  Evidence: src/api/routes.rs (35727 bytes), src/admin/mod.rs (403304 bytes)
- [warning] Very large top-level surface area: A crowded repository root increases onboarding and navigation cost.
  Evidence: .env, .gemini, .gitignore, .sentry-native, 20251230170754_init_core.sql, AGENTS.md, bin_test, bin_test.rs, Cargo.lock, Cargo.toml, demo_categories_products.sql, DEMO_DATA_README.md
- [warning] No test directories detected in a code-heavy repository: An unclear test story makes safe AI-assisted changes harder.
  Evidence: code-files=63
- [warning] Generated or build output folders appear committed: Committed build artifacts can add noise and hide the real source surface.
  Evidence: target
- [warning] Environment setup is ambiguous: Multiple env-like files or a missing .env.example can make setup unclear for humans and AI tools.
  Evidence: .env
- [warning] Excessive root-level clutter: A crowded root makes repository orientation harder than necessary.
  Evidence: .env, .gemini, .gitignore, .sentry-native, 20251230170754_init_core.sql, AGENTS.md, bin_test, bin_test.rs, Cargo.lock, Cargo.toml, demo_categories_products.sql, DEMO_DATA_README.md
- [high] Large files likely hard for AI and humans to work with: Very large files usually increase change risk and reduce prompt efficiency.
  Evidence: src/api/routes.rs (35727 bytes), src/admin/mod.rs (403304 bytes)
- [warning] Several top-level folders have unclear boundaries: Unclassified top-level directories suggest inconsistent folder conventions.
  Evidence: .gemini, .sentry-native, Repo, src/api
- [warning] No safe automation scripts were detected: Missing explicit lint/typecheck/test/build entrypoints slows reliable AI execution loops.
  Evidence: safe-commands-detected=0

## Repo-Intelligence Risks
- Some likely entrypoints have shallow traced execution paths: src/main.rs.
- Configuration files are operationally important here, especially package.json.

## Readiness Blockers
- [warning] Safe command story is unclear: Without explicit lint/typecheck/test/build entrypoints, AI loops become slower and riskier.
- [warning] Test and validation story is unclear: Lack of test structure reduces confidence in automated changes.
- [warning] Repository surface area is harder to control than necessary: Large files, deep trees, or a wide root increase the chance of imprecise AI changes.

## Command And Setup Gaps
- lint: no explicit safe command detected
- typecheck: no explicit safe command detected
- test-discovery: no explicit safe command detected
- build-discovery: no explicit safe command detected