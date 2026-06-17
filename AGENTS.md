# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace for the Lilium NG rewrite. Core code lives under `crates/` and binaries under `binaries/`.

- `crates/lilium-common`: shared utilities, constants, and error types
- `crates/lilium-models`: data models and row mappings
- `crates/lilium-database`: Database runtime, connection owners, transactions, and raw SQL sessions
- `crates/lilium-test-fixtures`: test-only database leasing, reset, and fixture profiles
- `crates/lilium-core`: pure business logic
- `crates/lilium-api-client`: external HTTP and Socket.IO transport
- `crates/lilium-services`: service-layer orchestration
- `binaries/lilium-spider`, `binaries/lilium-event-processor`, and `binaries/lilium-cli`: executable entry points
- `docs/database-layer-plan.md`: current database layer architecture spec
- `docs/python-to-rust-migration-sop.md`: required Python-to-Rust parity workflow
- `docs/python-to-rust-migration-progress.md`: current scenario-based migration progress tracker

Keep tests close to the code they verify. Use `crates/lilium-test-fixtures` for DB-backed integration tests.

## Architecture

```
Binaries (lilium-spider, lilium-event-processor, lilium-cli)
    → Service Layer (lilium-services)
        → Core Layer (lilium-core) — pure business logic, no async/DB
        → Domain Layer (lilium-models) — SeaORM entities and row mappings
        → Data Access (lilium-database) — connection pool, sessions, transactions
        → API Client (lilium-api-client) — external HTTP and Socket.IO calls
```

**Rule:** Never skip layers. Presentation code (binaries) must not use raw `sea_orm::Entity` queries directly — go through services.
Data access and API clients are sibling dependencies of services and binaries;
`lilium-database` must not depend on `lilium-api-client`.

## Build, Test, and Development Commands

- `cargo build`: compile the full workspace
- `cargo test`: run all workspace tests
- `cargo test -p lilium-database`: run database-layer tests only
- `cargo run --bin lilium-spider`: start the spider binary locally
- `cargo run --bin lilium-event-processor`: start the event processor locally
- `cargo run --bin lilium-cli`: run the operational CLI locally
- `cargo fmt --all`: format code with standard Rust formatting
- `cargo clippy --all-targets --all-features`: run lint checks


## Coding Style & Naming Conventions
Use standard Rust style and `rustfmt` defaults. Prefer small, explicit modules and preserve the current layer boundaries: database access in `lilium-database`, domain logic in `lilium-core`, and orchestration in `lilium-services`.

- files and modules: `snake_case`
- types, traits, enums: `CamelCase`
- functions, variables, and fields: `snake_case`
- binaries and workspace crates: descriptive, feature-specific names

## Critical Rules

### Database Migrations — Never Run Without Permission

**NEVER execute `sea-orm-cli migrate` or any migration apply command without explicit user approval.** This includes test databases that may share production credentials via `.env`.

- Generating migration files with `sea-orm-cli migrate generate` is OK.
- Applying migrations is FORBIDDEN unless the user explicitly says to run it.
- Subagents and background agents must also follow this rule.
- When generating migration scripts, verify model/migration consistency manually before treating the migration as complete.

### Message Lifecycle

- **New**: Insert with deduplication (`message_id` as PK)
- **Updated**: Preserve old content in `history` JSON array, update `updated_at`
- **Deleted/Recalled**: Set flag only. **DO NOT change content.**

### Wallet Balance — Never Direct UPDATE

All balance changes must go through `WalletService` methods (`credit`, `debit`, `freeze`, `unfreeze`, `release`, `transfer`) which create `wallet_transaction` records. Direct updates break ledger reconciliation.

### Enum Handling

- Use SeaORM `DeriveActiveEnum` for database enums.
- JSON serialization should use `snake_case` string values, not repr strings like `"Type::Variant"`.
- Compare with enum members directly, never with string matching.

### Constant And Content Data Changes

Do not add or update tests that merely pin constant values, static content data, tuning numbers, spawn rates, loot rates, or other balance constants. Changing constants does not require tests by itself.

## Testing Guidelines
Write tests that verify behavior, not constants or placeholder assertions. Prefer targeted tests for the affected crate or module, and use the shared DB fixture helpers for transactional database cases.

Observability should be treated as day-0 infrastructure for migrated binaries. Keep tracing in place, and carry over the Python-side Sentry/telemetry surface when a feature depends on runtime diagnostics.

### Agent Test Rule (Narrowest Verification)

After code changes, run verification that matches the surface area you touched:

- Single-crate changes: `cargo test -p <crate>`
- Single-module changes: `cargo test -p <crate> <module>`
- Prefer `cargo test -p <crate> --all-targets` over full `cargo test`
- Run full `cargo test` only when the user explicitly asks, or when changes are broad enough that targeted verification is not credible
- Always run `cargo fmt --all` and `cargo clippy --all-targets --all-features` before committing

If tests cannot run (missing database, external service), report explicitly and continue.

### Do Not Pin Constant Values in Tests

Do not add tests that assert constant values, tuning numbers, or static content data. These create maintenance burden without verifying behavior. Test the logic that *uses* constants, not the constants themselves.

Protocol and storage contract values are behavior, not tuning constants: event
names, JSON field names, environment variable names, database table/column names,
and exit-code semantics may be asserted when they prove Python parity or public
compatibility.

## Observability Guidelines
Instrument service-layer boundaries that perform external I/O or orchestrate external I/O: database calls, dedicated PostgreSQL sessions, API/network requests, filesystem reads or writes, notification listeners, and long-running polling loops.

Do not instrument pure helpers, constructors, deterministic mappers, parsers, normalizers, path builders, cache predicates, retry classifiers, hash/id calculators, or other functions that only transform in-memory values. These create noisy spans without improving runtime diagnosis.

When adding `#[instrument]`, skip large or sensitive values such as database handles, API clients, request bodies, credentials, cookies, raw payloads, URLs, file contents, and callback closures. Record compact fields such as IDs, counts, booleans, operation names, and durations. Keep SQL visibility in the database observability layer instead of duplicating SQL-like spans in services.

Sentry queries made during debugging should default to `statsPeriod=1h`; only expand the window when explicitly requested.

When investigating production/runtime issues or checking logs, query Sentry first. Do not inspect local log files by default — only if Sentry does not contain the needed evidence or the user explicitly asks for local log analysis.

## Before Committing

1. Only commit files you modified
2. `cargo fmt --all`
3. `cargo clippy --all-targets --all-features`
4. Run the narrowest credible test verification for the code you touched
5. If parity work: update `docs/python-to-rust-migration-progress.md` and parity comments
6. If models changed: create or update migration files
7. DO NOT change upstream remote URLs — if SSH is not working, ask the user

## Commit & Pull Request Guidelines

### Commit Message Format

All commits must follow Conventional Commits:

- Preferred format: `type(scope): short summary`
- Allowed when no scope adds value: `type: short summary`
- Lowercase type, imperative mood, no trailing period
- Keep one intent per commit when practical
- Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`, `build`, `revert`
- Use `!` or a `BREAKING CHANGE:` footer for breaking changes
- For larger changes, add a body that explains why and a `Tests:` section

### Pull Request Format

Every PR description should be structured so reviewers can understand the change quickly:

- **Why / Motivation**: what problem this PR solves, why it matters now, and any issue/spec/plan references
- **Design / Approach**: the core implementation idea, important tradeoffs, non-obvious decisions
- **Scope**: what is included and what is explicitly out of scope
- **Validation**: commands run, manual verification, screenshots/logs when relevant
- **Post-merge Follow-ups**: required migrations, config changes, monitoring, cleanup, or explicit `None`

Additional rules:

- Do not open PRs with empty descriptions or one-line summaries like "misc fixes"
- If rollout or operational risk exists, call out rollback/mitigation notes
- If CI failures are out of scope, say so explicitly in the PR body instead of silently fixing unrelated code
- ALWAYS use squash merge, no exceptions!

### PR CI Scope Rule

When a pull request has failing CI or checks:

- First determine whether the failure is caused by the branch diff, by the PR merge against the latest base branch, or by unrelated/pre-existing breakage elsewhere.
- Do **not** expand the PR scope to fix unrelated CI failures by default.
- If the failure is unrelated, document it with concrete evidence and treat it as out of scope unless the user explicitly asks to include it.
- If the failure is due to base-branch drift, merging/rebasing the latest base branch is acceptable, but do not add unrelated code changes just to make the check pass.

### Design-First Rule

For multi-step feature work or system behavior changes that require a durable
design document, the design document must be completed under `docs/plans/` and
committed before implementation begins. Agent execution plans under
`docs/superpowers/plans/` are not design documents.

- Do not start implementation code changes until the design doc exists and is committed.
- If the design changes materially during review, update and commit the design doc again before continuing.
- Reference the committed design doc in tracking issues/PRs when applicable.

## Per-Crate Conventions

### Service Layer (`lilium-services`)

- **Result types over panics** for expected failures (insufficient funds, limit exceeded, lock conflict). Services return `Result<T, LiliumError>` with domain-specific error variants, not `anyhow::Error` for business logic failures.
- **Typed DTOs over `serde_json::Value`** inside the service layer. Use named structs for stable data structures and cross-service return values. Keep `serde_json::Value` at system boundaries (external APIs, raw JSONB columns, message queues).
- **API boundary lives above services**. Service methods return typed domain models/DTOs, not ad-hoc JSON-ready values. Serialization into API response shapes belongs in the binary/presentation layer.
- **Expected failures must be semantic**. Service-layer errors should carry stable context (error kind, message, retryable flag). Do not return bare `String` errors for business failures.
- **Cursor-based pagination** for large result sets.
- **No raw SQL in presentation code** — always go through services.

### Database Layer (`lilium-database`)

- **Pool health**: Set `pool_pre_ping` equivalent and connection recycle timeout for long-running processes.
- **Timezone**: Enforce UTC on all connections.
- **Dedicated connections**: Advisory lock connections and LISTEN/NOTIFY connections must use separate physical connections from the application pool.
- **Migration consistency**: When generating or editing migration scripts, verify that model metadata and migration are consistent. Do not ship migrations that only "make the current database happy" while leaving model/migration drift.

### Model Layer (`lilium-models`)

- **One entity per module** when the entity is substantial. Shared domain types go in domain-local modules (e.g., `ingestion/mod.rs`).
- **No ORM relationships in entities**. SeaORM relations are defined but joins are resolved in the service layer, not via lazy loading.
- **Timestamps always timezone-aware** — use `DateTime<Utc>` (chrono) for all timestamp fields.
- **Factory methods for derived fields** — use `impl` constructors that calculate derived state (expiry, settlement) rather than requiring callers to compute them.

### Core Layer (`lilium-core`)

- **Pure functions only** — no async, no DB access, no I/O. Input → output, no side effects.
- **Testable without database** — all core logic must be unit-testable with mock/fixture data.
- **Frozen by default** — prefer immutable data structures. Use `Clone`/`Copy` semantics, avoid interior mutability unless necessary.

### API Client (`lilium-api-client`)

- **Auth retry**: HTTP requests should automatically retry on 401/403 after refreshing credentials.
- **Cookie management**: Merge cookies from jar and response, handle deduplication.
- **Rate limiting**: Enforce request throttling to avoid detection.
- **Anti-detection**: Realistic browser headers, randomized delays, no `Authorization` headers (cookies only).

### Test Fixtures (`lilium-test-fixtures`)

- **Database safety**: Tests must never connect to the production database. Use `TEST_DATABASE_URL` only.
- **Per-process isolation**: Each test process gets a unique test database (PID + sequence).
- **Truncate between tests**: Clear table rows between tests, don't drop/recreate tables.
- **Fixture profiles**: Use composable fixture profiles (Empty, Shared, User, Message) rather than monolithic seed data.

## Agent Notes
Before changing behavior, confirm the root cause with code or runtime evidence. Avoid speculative fixes, avoid noisy reruns, and resolve PR review threads after addressing them.
For Python parity work, read the real Python source first and treat markdown analysis as index or hints only, not as the source of truth.
During Python parity scans, update code comments and
`docs/python-to-rust-migration-progress.md` in the same change set as the code.
Update architecture docs in the same change set when the scan changes ownership,
runtime boundaries, connection lifetimes, config rules, public APIs, or
cross-crate boundaries.
Always use Rust 2024 edition.
Do not add test backdoors to production code. Avoid `#[cfg(test)]` / `#[cfg(not(test))]` behavior branches, magic `__test` values, noop implementations, or test-only constructors in production modules. When tests need to isolate network, workers, time, or external services, define an explicit dependency boundary and use a real mock library or fixture in the test module.
