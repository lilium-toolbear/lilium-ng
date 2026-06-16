# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace for the Lilium NG rewrite. Core code lives under `crates/` and binaries under `binaries/`.

- `crates/lilium-common`: shared utilities, constants, and error types
- `crates/lilium-models`: data models and row mappings
- `crates/lilium-database`: Database runtime, SeaORM entities, and raw SQL sessions
- `crates/lilium-test-fixtures`: test-only database leasing, reset, and fixture profiles
- `crates/lilium-core`: pure business logic
- `crates/lilium-services`: service-layer orchestration
- `binaries/lilium-spider` and `binaries/lilium-event-processor`: executable entry points
- `docs/database-layer-plan.md`: current database layer architecture spec
- `docs/python-to-rust-migration-sop.md`: required Python-to-Rust parity workflow
- `docs/python-to-rust-migration-progress.md`: current scenario-based migration progress tracker

Keep tests close to the code they verify. Use `crates/lilium-test-fixtures` for DB-backed integration tests.

## Build, Test, and Development Commands

- `cargo build`: compile the full workspace
- `cargo test`: run all workspace tests
- `cargo test -p lilium-database`: run database-layer tests only
- `cargo run --bin lilium-spider`: start the spider binary locally
- `cargo run --bin lilium-event-processor`: start the event processor locally
- `cargo fmt --all`: format code with standard Rust formatting
- `cargo clippy --all-targets --all-features`: run lint checks


## Coding Style & Naming Conventions
Use standard Rust style and `rustfmt` defaults. Prefer small, explicit modules and preserve the current layer boundaries: database access in `lilium-database`, domain logic in `lilium-core`, and orchestration in `lilium-services`.

- files and modules: `snake_case`
- types, traits, enums: `CamelCase`
- functions, variables, and fields: `snake_case`
- binaries and workspace crates: descriptive, feature-specific names

## Testing Guidelines
Write tests that verify behavior, not constants or placeholder assertions. Prefer targeted tests for the affected crate or module, and use the shared DB fixture helpers for transactional database cases.

Observability should be treated as day-0 infrastructure for migrated binaries. Keep tracing in place, and carry over the Python-side Sentry/telemetry surface when a feature depends on runtime diagnostics.

## Observability Guidelines
Instrument service-layer boundaries that perform external I/O or orchestrate external I/O: database calls, dedicated PostgreSQL sessions, API/network requests, filesystem reads or writes, notification listeners, and long-running polling loops.

Do not instrument pure helpers, constructors, deterministic mappers, parsers, normalizers, path builders, cache predicates, retry classifiers, hash/id calculators, or other functions that only transform in-memory values. These create noisy spans without improving runtime diagnosis.

When adding `#[instrument]`, skip large or sensitive values such as database handles, API clients, request bodies, credentials, cookies, raw payloads, URLs, file contents, and callback closures. Record compact fields such as IDs, counts, booleans, operation names, and durations. Keep SQL visibility in the database observability layer instead of duplicating SQL-like spans in services.

Sentry queries made during debugging should default to `statsPeriod=1h`; only expand the window when explicitly requested.

## Commit & Pull Request Guidelines
Commit history uses conventional prefixes such as `feat:`, `fix:`, `docs:`, `refactor:`, and `chore:`. Keep commit messages short and imperative.

Pull requests should explain the change, reference any related issue or audit note, and list the commands you ran. Include screenshots only for UI-facing changes.

## Agent Notes
Before changing behavior, confirm the root cause with code or runtime evidence. Avoid speculative fixes, avoid noisy reruns, and resolve PR review threads after addressing them.
For Python parity work, read the real Python source first and treat markdown analysis as index or hints only, not as the source of truth.
During Python parity scans, update code comments, architecture docs, and `docs/python-to-rust-migration-progress.md` in the same change set as the code.
Always uses rust 2024 edition
Do not add test backdoors to production code. Avoid `#[cfg(test)]` / `#[cfg(not(test))]` behavior branches, magic `__test` values, noop implementations, or test-only constructors in production modules. When tests need to isolate network, workers, time, or external services, define an explicit dependency boundary and use a real mock library or fixture in the test module.
