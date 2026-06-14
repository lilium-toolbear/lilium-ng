# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace for the Lilium NG rewrite. Core code lives under `crates/` and binaries under `binaries/`.

- `crates/lilium-common`: shared utilities, constants, and error types
- `crates/lilium-models`: data models and row mappings
- `crates/lilium-database`: pool/session management and SQL helpers
- `crates/lilium-core`: pure business logic
- `crates/lilium-services`: service-layer orchestration
- `binaries/lilium-spider` and `binaries/lilium-event-processor`: executable entry points
- `docs/`: parity audits, migration notes, and handoff documents

Keep tests close to the code they verify, especially module-level unit tests and DB fixture tests in `crates/lilium-database`.

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

## Commit & Pull Request Guidelines
Commit history uses conventional prefixes such as `feat:`, `fix:`, `docs:`, `refactor:`, and `chore:`. Keep commit messages short and imperative.

Pull requests should explain the change, reference any related issue or audit note, and list the commands you ran. Include screenshots only for UI-facing changes.

## Agent Notes
Before changing behavior, confirm the root cause with code or runtime evidence. Avoid speculative fixes, avoid noisy reruns, and resolve PR review threads after addressing them.
For Python parity work, read the real Python source first and treat markdown analysis as index or hints only, not as the source of truth.
