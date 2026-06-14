# Goal Document: Python Test Parity Recovery

## Current Status

- The original count-based parity snapshot is stale.
- Current parity work should be tracked through the per-module Python analysis docs plus `docs/HANDOFF.md`, `docs/rust-code-audit.md`, and `docs/python-parity-entrypoint-audit.md`.
- Behavior parity still matters more than raw test counts; new tests should always point to the exact Python test file they mirror.
- There are currently no `#[ignore]`, `todo!()`, `unimplemented!()`, or other obvious placeholder test markers in the Rust workspace.
- DB-backed tests are wired through `crates/lilium-test-fixtures` and require `TEST_DATABASE_URL`; there is no fallback path in the harness.
- A small number of tests already carry explicit Python-mirror comments, but most migrated tests still rely on Rust-only names and assertions instead of a source Python file reference.

## Current Checklist

- [ ] When a Rust test is added or changed, record the source Python test file and the concrete behavior slice it covers.
- [ ] Expand event-processor coverage for the remaining Python event branches, especially `message:updated`, `message:recalled`, `presence:user-online`, `group:member-joined`, and unknown-event handling.
- [ ] Keep DB-backed tests env-only and do not add fallback paths in the harness.
- [ ] Refresh the handoff and audit docs after each finished batch so they stay current.
