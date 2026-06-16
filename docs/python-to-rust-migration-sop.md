# Python To Rust Migration SOP

This SOP defines the required workflow for migrating behavior from the Python
codebase into this Rust workspace. Its purpose is to keep parity work grounded
in real source code and prevent stale analysis documents from becoming the
source of truth.

## Source Of Truth

The Python source code is the source of truth.

Markdown notes, old audits, handoff files, issue comments, and previous agent
summaries are only search indexes. They can point to files and topics, but they
do not prove behavior. Every parity claim must be verified against the current
Python implementation and the current Rust implementation.

When Python behavior depends on database schema, JSON shape, HTTP payload,
Socket.IO payload, config loading, environment variables, or observability, use
the real source code and runtime fixtures that define those surfaces.

## Workflow

1. Start from an executable boundary.

   Use the Rust binary entry point, service public function, client method, or
   test case being migrated as the boundary. Trace downward through Rust code
   and then map the same boundary to the Python source.

2. Read the Python implementation directly.

   Read the Python entry point, called services, models, fixtures, config
   loaders, and tests that define the behavior. Do not infer behavior from file
   names, stale Markdown, or previous migration counts.

   After completing a Python equivalence scan, record the exact Python source
   file path and Git commit hash in a Rust code comment next to the migrated
   model, function, or module. Use the current Python repository commit from
   `git -C ../dzmm_archive rev-parse HEAD`. The
   comment must identify the source that was read, for example:

   ```rust
   // Python parity source: dzmm_archive@<commit> models/dzmm/message.py
   ```

   When one Rust module covers multiple Python files, list each Python file in
   separate comments. When Rust intentionally diverges from Python, add a short
   comment naming the Python field or behavior and the reason for the
   divergence.

3. Record the behavior contract before changing Rust.

   Identify the exact externally visible contract:

   - database tables, columns, constraints, partitions, and reset behavior
   - JSON request and response bodies
   - HTTP method, path, query, headers, cookies, and base URL handling
   - Socket.IO event names, transport choice, reconnect policy, and headers
   - environment variables and `.env` loading
   - Sentry, tracing, metrics, and log context
   - domain errors and status semantics
   - concurrency and isolation rules

4. Compare scenarios, not file counts.

   Build a scenario matrix from Python behavior. Each scenario must state the
   input state, action, expected output, side effects, and failure mode. Matching
   the number of Python tests is not a parity result.

5. Implement the Rust behavior at the owning layer.

   Put database runtime concerns in `lilium-database`, test isolation in
   `lilium-test-fixtures`, pure calculations in `lilium-core`, API transport in
   `lilium-api-client`, and orchestration in `lilium-services` and binaries.
   Do not patch one caller when the missing abstraction belongs to a shared
   crate.

6. Preserve external shapes.

   Rust may use different internal structure, but database schema, JSON
   payloads, HTTP endpoints, config names, error meaning, and observable runtime
   behavior must stay compatible with Python unless the migration explicitly
   changes the contract.

7. Port tests as behavior checks.

   Rust tests must verify behavior and compatibility surfaces. DB-backed tests
   use `crates/lilium-test-fixtures`, load `.env`, require `TEST_DATABASE_URL`,
   and never use `#[ignore]` as a fixture gate. A missing required environment
   variable must fail clearly.

8. Verify with targeted commands.

   Run the smallest meaningful Rust test set first, then broader checks when the
   changed surface crosses crate boundaries. Do not rerun unrelated tests just to
   add noise.

9. Update docs during the scan, not after memory fades.

   Every functionality or logic scan must update the durable project record
   while the source evidence is still fresh:

   - Add Rust code comments next to migrated logic that name the Python file and
     commit hash.
   - Update architecture docs when the scan changes ownership, runtime
     boundaries, connection lifetimes, config rules, or cross-crate APIs.
   - Update `docs/python-to-rust-migration-progress.md` with the scanned
     boundary, Python sources read, Rust files touched, verified scenarios,
     remaining gaps, and commands run.
   - Keep progress notes scenario-based. Do not track parity by raw file counts
     or test counts.

10. Update durable docs only.

   Keep stable architecture specs and this SOP current. Delete stale audits,
   handoff files, and one-off investigation notes after their work has landed.
   Long-lived progress tracking belongs in
   `docs/python-to-rust-migration-progress.md`. README must describe the current
   workspace, not maintain a historical document index.

## Required Evidence

Every completed migration slice must be backed by:

- Python source files read for the behavior.
- Python source file paths and commit hashes recorded in Rust comments for the
  migrated slice.
- The matching entry in `docs/python-to-rust-migration-progress.md` updated in
  the same change set.
- Architecture docs updated when the scan changes runtime ownership, config,
  connection lifetimes, database access, notification behavior, or public APIs.
- Rust files changed for the implementation.
- Tests or commands run.
- The compatibility surface that was verified.

Do not claim parity from successful compilation alone.

## Red Flags

Stop and re-check the Python source when any of these appear:

- placeholder comments, empty functions, `todo!()`, `unimplemented!()`, dead
  public symbols, or unused service dependencies
- broad `anyhow::Error` in service code where the caller needs domain semantics
- `#[instrument(level = "debug" skip_all)]` added without checking field sensitivity
- copied SQL that duplicates ORM responsibilities without a clear PostgreSQL
  reason
- tests gated by `#[ignore]` for required infrastructure
- config files invented for Rust when Python uses environment variables
- docs used as proof instead of source code

## Current Project Decisions

- Production config is environment-driven and loaded by binaries.
- Database access enters through `Database`, `DatabaseConfig`,
  `transaction!`, `DbSession`, and `RawDbConnection`.
- SeaORM is the ORM layer. Raw SQL remains for PostgreSQL-specific behavior.
- Test database isolation uses pooled temporary databases plus truncate/reset,
  not shared DB locking and not transaction rollback isolation.
- Observability is day-0 migration scope. Tracing and Sentry behavior must be
  present when the Python feature depends on runtime diagnostics.
