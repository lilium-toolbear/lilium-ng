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

9. Update durable docs only.

   Keep stable architecture specs and this SOP current. Delete stale audits,
   migration trackers, and handoff files after their work has landed. README
   must describe the current workspace, not maintain a historical document
   index.

## Required Evidence

Every completed migration slice must be backed by:

- Python source files read for the behavior.
- Rust files changed for the implementation.
- Tests or commands run.
- The compatibility surface that was verified.

Do not claim parity from successful compilation alone.

## Red Flags

Stop and re-check the Python source when any of these appear:

- placeholder comments, empty functions, `todo!()`, `unimplemented!()`, dead
  public symbols, or unused service dependencies
- broad `anyhow::Error` in service code where the caller needs domain semantics
- `#[instrument(skip_all)]` added without checking field sensitivity
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
