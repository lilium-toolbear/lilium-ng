# Python vs Rust Test Fixture Parity Gap

Source of truth for the Python side:
- `tests/conftest.py`
- `database/async_engine.py`
- `tests/helpers/session_harness.py`

This document records the current Rust fixture state relative to that Python model.

## What Python Does

1. Loads the project `.env` during pytest bootstrap, then removes production-facing DB and Sentry env vars from the test process.
2. Uses one temporary PostgreSQL database per pytest worker/run, not a shared mutable test database.
3. Rebuilds or resets test state per test by clearing tables and restoring required partitions.
4. Keeps a separate helper for multi-session tests, but the default isolation model is worker DB plus per-test reset.
5. Uses test-specific setup for partitioned tables and CI-specific `UNLOGGED` table behavior.

## What Rust Has Today

1. `crates/lilium-test-fixtures` loads `.env` lazily for helpers and requires `TEST_DATABASE_URL`.
2. `crates/lilium-test-fixtures` provisions pooled temporary PostgreSQL databases from the live schema bootstrap in `crates/lilium-database/testdata/live_schema_bootstrap/0001_live_schema.sql`.
3. The bootstrap file is now pure SQL and recreates the live table / trigger layout without depending on a local TOML config or the unavailable `zhparser` extension.
4. Per-test reset truncates live tables, restarts identities, unlocks advisory locks, and recreates the partitions exercised by tests.
5. Leased test databases are returned to an in-process pool after each test unit, and startup sweeps stale `lilium_test_*` databases that are no longer in use.
6. The event processor offset model now matches the live schema `integer` column instead of assuming `bigint`.

## Remaining Gaps

1. No CI-only `UNLOGGED` table optimization.
2. Rust does not mirror Python's worker naming or lifecycle exactly; it uses a process-local database pool rather than pytest worker names.

## Executable Repair List

### Done

- [x] Process-scoped temporary test database provisioning.
- [x] Live schema bootstrap from exported SQL.
- [x] Pure SQL bootstrap file without `pg_dump` control commands.
- [x] Test-time `zhparser` compatibility layer that does not require the extension.
- [x] Per-test table reset and partition repair.
- [x] Pooled test database lease / return lifecycle.
- [x] Startup sweep for stale `lilium_test_*` databases with no active connections.
- [x] `event_processor_offsets.last_processed_id` schema/type alignment.
- [x] Fixture responsibilities split into database lease, reset, seed, and profile layers.

### Still Open

- [ ] Add optional CI-only `UNLOGGED` mode if performance becomes a bottleneck.

## Current Recommendation

If the goal is Python parity rather than just green tests, the next step is optional CI-only performance tuning. The current implementation follows the important isolation rule: concurrent test units lease separate databases, and reuse relies on truncate/reset rather than drop/recreate.
