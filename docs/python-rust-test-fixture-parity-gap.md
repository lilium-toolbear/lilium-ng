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

1. `crates/lilium-database/src/test_fixtures.rs` loads `.env` lazily for helpers and requires `TEST_DATABASE_URL`.
2. `crates/lilium-database/src/test_fixtures.rs` provisions a process-scoped temporary PostgreSQL database from the live schema bootstrap in `crates/lilium-database/testdata/live_schema_bootstrap/0001_live_schema.sql`.
3. The bootstrap file is now pure SQL and recreates the live table / trigger layout without depending on a local TOML config or the unavailable `zhparser` extension.
4. Per-test reset now deletes rows from the live tables and recreates the partitions exercised by tests.
5. The temporary test database is cleaned up at process exit, and startup sweeps stale `lilium_test_*` databases that are no longer in use.
6. The event processor offset model now matches the live schema `integer` column instead of assuming `bigint`.

## Remaining Gaps

1. Rust still uses a service-fixture reset path rather than one metadata-driven canonical schema reset helper.
2. No CI-only `UNLOGGED` table optimization.
3. Rust does not mirror Python's worker naming or lifecycle exactly; it uses one temp DB per process.

## Executable Repair List

### Done

- [x] Process-scoped temporary test database provisioning.
- [x] Live schema bootstrap from exported SQL.
- [x] Pure SQL bootstrap file without `pg_dump` control commands.
- [x] Test-time `zhparser` compatibility layer that does not require the extension.
- [x] Per-test table reset and partition repair.
- [x] Automatic cleanup for the temporary test database at process exit.
- [x] Startup sweep for stale `lilium_test_*` databases with no active connections.
- [x] `event_processor_offsets.last_processed_id` schema/type alignment.

### Still Open

- [ ] If strict parity is desired, replace the current fixture-specific reset with a metadata-driven schema reset layer.
- [ ] Add optional CI-only `UNLOGGED` mode if performance becomes a bottleneck.

## Current Recommendation

If the goal is Python parity rather than just green tests, the next step is optional teardown plus a metadata-driven reset layer. The current implementation is already closer to the Python model than the prior shared-lock fixture approach.
