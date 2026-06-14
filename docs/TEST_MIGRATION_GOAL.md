# Goal Document: Python Test Parity Recovery

## Go / No-Go
- **Judgment**: Go
- **Reason**: The workspace compiles, but current Rust test count is not a parity signal. Several translated modules have zero or superficial Rust tests despite Python behavior tests existing.

## Target Outcome
Translated Rust modules have behavior tests ported from the real Python test suite, with a traceable matrix showing which Python tests are covered, deferred, or blocked by missing DB/test harness.

## Goal Definition
- **Type**: quality / migration parity
- **Boundary**: `lilium-ng` modules already translated from Python: API client, websocket decoding, spider runtime pieces, event processor, services, models, core/database contracts.
- **Non-goals**:
  - Port tests for Python domains that do not exist in this Rust repo yet.
  - Add tests that only assert constants or object construction without migrated behavior value.
  - Hide missing DB coverage behind more `#[ignore]` tests without a harness plan.
- **Deferred work**:
  - Full PostgreSQL integration harness for service tests.
  - End-to-end runtime tests that need real DZMM credentials or network.
- **Verification rule**: Each migrated slice names the source Python tests and has focused Rust test commands that fail before the implementation change and pass after.
- **Evidence source**: Python test files under `/Users/bearice/Working/github/dzmm_archive/tests`, Rust tests under `crates/` and `binaries/`, and focused `cargo test` runs.
- **Pass criteria**: For every included module, the matrix no longer has unclassified Python tests; each covered behavior has a passing Rust test or an explicit blocked/deferred reason.
- **Confidence note**: Function counts are only triage. Completion requires file-by-file comparison against Python test bodies.

## Current State
| Area | Python tests | Rust tests | Rust ignored | Initial finding |
|------|--------------|------------|--------------|-----------------|
| API client | 120 | 121 | 0 | Counts are similar, but behavior parity still needs body-level review. |
| Spider arbiter/runtime/worker/control | 37 | 12 | 0 | Major runtime/control behavior missing. |
| Spider event processor/ingestion | 30 | 28 | 0 | Count is close; body-level parity still needed. |
| Message service | 76 | 88 | 34 | Many Rust tests are query-builder or ignored DB tests; Python cursor/enrichment/batch behaviors are not proven. |
| User service | 41 | 30 | 18 | Search/history/top-users/batch behavior gaps remain. |
| Room member service | 48 | 23 | 18 | Search, batch, stale join/leave, admin count gaps remain. |
| Event/offset service | 30 | 0 | 0 | No Rust tests despite translated service code. |
| Account service | 31 | 0 | 0 | No Rust tests despite translated service code. |
| Outgoing command service | 43 | 40 | 17 | Count close, but DB behavior is mostly ignored. |
| Notification service | 1 | 8 | 0 | Rust tests do not match the single Python Sentry propagation behavior. |
| Models | 69 | 2 | 0 | Largest no-DB gap; message parsing/account/wallet/connection tests mostly absent. |

## Priority Rationale
- Start with no-DB model and parser tests because they expose real translation defects without waiting for infrastructure.
- Then cover spider/control runtime behavior, which also has a low external dependency footprint.
- Build a DB harness before claiming service parity; ignored tests are not proof.

## Assumptions and Open Decisions
| Item | Status | Impact | Owner / Next step |
|------|--------|--------|-------------------|
| Python source of truth | confirmed | Tests must be ported from real Python files, not summaries. | Read source test files before each slice. |
| DB service tests | unresolved | Many Python service tests need PostgreSQL fixtures. | Define Rust test DB harness before unignoring/porting DB-heavy tests. |
| Public Rust constructors for models | unresolved | Python model tests assume default constructors; Rust structs currently require explicit fields. | Decide per model whether to add constructors/defaults or test serialization/parse behavior only. |

## Phases

### Phase 1: No-DB Model Behavior
- **Purpose**: Recover high-signal model/parser coverage without infrastructure.
- **Phase rules**:
  - Read the Python model test file and source model before adding Rust tests.
  - Do not assert meaningless constants just to increase count.
  - Use red/green for each behavior slice.
- **Todos**:
  - [x] Port the first `tests/unit/models/test_message_parsing.py` behavior slice to `crates/lilium-models/src/dzmm/message.rs`: wrapped event, image attachment, video metadata, sticker, and reference parsing.
- [x] Continue `tests/unit/models/test_message_parsing.py`: invalid-event errors, recalled/edited/system message semantics, and all-sample fixture coverage.
  - [ ] Classify account, websocket connection, wallet model tests as covered, requires constructors/defaults, or schema-only.
- **Exit proof**: Focused `cargo test -p lilium-models`.
- **Stop condition**: Rust model API decision is needed before adding constructors/defaults.

### Phase 2: Spider Runtime/Control Parity
- **Purpose**: Fill the biggest non-DB binary gaps.
- **Phase rules**:
  - Port Python runtime/control tests by behavior, not by line count.
  - Prefer existing socket/control APIs; do not reshape production APIs only for tests.
- **Todos**:
  - [ ] Compare `tests/unit/spider/test_ws_control.py` against `binaries/lilium-spider/src/control.rs`.
  - [ ] Compare arbiter/runtime/worker tests and classify missing runtime surfaces.
- **Exit proof**: Focused `cargo test -p lilium-spider`.
- **Stop condition**: Missing production architecture makes a Python behavior impossible to express.

### Phase 3: DB Harness Before Service Parity
- **Purpose**: Stop accumulating ignored service tests.
- **Phase rules**:
  - No new ignored DB tests unless the doc records why the harness cannot support them yet.
  - Derive schema from `docs/python-analysis/04-models/MODEL_DDL.sql`.
- **Todos**:
  - [ ] Define a Rust PostgreSQL test harness and cleanup discipline.
  - [ ] Unblock event/offset and account service tests first because they currently have zero coverage.
- **Exit proof**: At least one service module has non-ignored DB parity tests passing locally.
- **Stop condition**: DB schema or fixture setup cannot be derived from current metadata.

### Phase 4: Body-Level Coverage Matrix
- **Purpose**: Replace count-based status with evidence.
- **Phase rules**:
  - Every Python test gets one status: covered, intentionally different, deferred, blocked, or out of scope.
  - Each covered row names the Rust test.
- **Todos**:
  - [ ] Create module-by-module mapping for included Python files.
  - [ ] Update `docs/HANDOFF.md` with true parity status after each batch.
- **Exit proof**: Matrix has no unclassified rows for included modules.
- **Stop condition**: A Python test covers a module not translated into Rust.

## Final Validation
- `cargo test -p lilium-models`
- `cargo test -p lilium-spider`
- `cargo test -p lilium-event-processor`
- Service-specific cargo commands after DB harness exists.

## First Execution Step
Port the first no-DB behavior slice from `tests/unit/models/test_message_parsing.py` into `crates/lilium-models/src/dzmm/message.rs`, starting with RED tests for image/video/sticker/reference parsing.
