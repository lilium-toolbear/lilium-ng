# Python Recent Commit Port Tracker

Reviewed at: 2026-06-15

Python source: `/Users/bearice/Working/github/dzmm_archive`

Range: `origin/master` commits since `2026-06-13 00:00:00 +0900`

Local Python working tree had uncommitted files during review; this tracker uses
committed `origin/master` history only.

## Immediate Rust Ports

### Network source-address binding

- Python commits:
  - `980bb2e0c12788ddc6575a0d1c51aff0ea543e10`
    `fix(network): bind api and websocket local address`
  - `f3d6061b01cd66ad2a8c512a2c1c45b7ba22567e`
    `fix(api): add http local address env`
- Python files:
  - `dzmm_client/api.py`
  - `spider/ws_runtime.py`
  - `tests/unit/dzmm_client/test_api.py`
  - `tests/unit/spider/test_ws_runtime.py`
- Rust status: ported in `crates/lilium-api-client`.
- Implemented Rust behavior:
  - Read `DZMM_HTTP_LOCAL_ADDRESS`.
  - Treat unset and `auto` as default network behavior.
  - Accept only `0.0.0.0` and `::`.
  - Bind outbound HTTP through `reqwest::ClientBuilder::local_address`.
  - Bind outbound websocket TCP sockets to the configured local address family
    before websocket/TLS handshake.
  - Cover unset, `auto`, IPv4, IPv6, and invalid values with Rust tests.
- Verification:
  - `cargo test -p lilium-api-client`

### Gamefy draw REST image-generation API

- Python commits:
  - `f3eb95fe6285531abe612d9ae708dbdd1f934ed4`
    `fix(gamefy): switch draw client to REST`
  - `266080ea83951a009cc544d53e3fe0cfd049a628`
    `feat(api): update default model to 'iroha' for image generation`
  - `6bd6f129403e9d76bdcd2dd69ed76f181f9be365`
    `fix(dzmm): send flat gamefy draw payload`
- Python files:
  - `dzmm_client/api.py`
  - `tests/repro/test_api_refactor.py`
  - `scripts/generate_raid_monster_art.py`
- Rust status: ported in `crates/lilium-api-client`.
- Implemented Rust behavior:
  - `DzmmApi` stores a configurable `base_url`; `DzmmApi::new` uses
    `ApiClientConfig::default()`, and `DzmmApi::new_with_config` accepts an
    explicit `ApiClientConfig`.
  - All HTTP client methods use the instance `base_url` for direct requests,
    generic `_request`, Origin/Referer headers, and image downloads.
  - `start_image_generation` posts a flat JSON payload to `/api/gamefy/draw`.
  - `poll_generation_task` gets `/api/gamefy/draw/status?taskId=<id>` and
    unwraps a top-level `task` object when present.
  - `generate_image` defaults to model `iroha`.
  - `upload_reference_image` sends
    `{"image":"data:<mime>;base64,<payload>"}` to
    `/api/gamefy/draw/upload-reference`.
  - `start_image_edit` posts a flat JSON payload to `/api/gamefy/draw/edit`.
  - Response validation accepts successful REST responses that omit a
    `success: true` field, and only treats `success: false` as failure.
  - Rust tests use a local HTTP server to verify base URL, endpoint, query, and
    request body compatibility.
- Verification:
  - `cargo test -p lilium-api-client`

## Future Ports When The Pal Domain Is Migrated

### Keep assignments during listing and gifting

- Python commit:
  - `644338701d04fb34a5362c8f9a63132e8bf76e9e`
    `feat(pal): keep assignments during listing and gifting (#991)`
- Python files:
  - `services/gift_service.py`
  - `services/market_service.py`
  - `services/work_tick_service.py`
  - `tests/unit/services/test_work_tick_service.py`
- Rust status:
  - lilium-ng currently has only pure pal/work-efficiency helpers in
    `crates/lilium-core`; it does not yet have the Python pal ownership,
    market, gift, assignment, or work-tick service layer.
- Required future behavior:
  - Listing or gifting a pal must not eagerly release its active assignment.
  - Work tick and dormitory display logic must ignore temporarily unavailable
    workers such as listed or pending-gift pals.
- Priority after pal service migration: high.

### Narrow pal assignment lock scope

- Python commit:
  - `255dd603ea76d11d7802697f9bbdc48c3f81781a`
    `feat(pal): narrow assignment lock scope (#992)`
- Python files:
  - `models/pal/pal_egg.py`
  - `services/pal_assignment_locking.py`
  - `services/gift_service.py`
  - `services/market_service.py`
  - `services/pal_service.py`
  - `services/pal_work_service.py`
  - `services/work_tick_service.py`
- Rust status:
  - No equivalent persistent pal/egg assignment model or service exists yet.
- Required future behavior:
  - Lock assignment rows first only when assignment state is being changed.
  - Lock pal rows separately for availability checks.
  - Release assignments only when ownership changes.
  - Add parent foreign-key constraints for pal breeding eggs.
- Priority after pal service migration: high, because it affects concurrency and
  ownership correctness.

### Confirm bulk release of working pals

- Python commit:
  - `666a8a286b6745b4b5dee790d6cbe4dcf5c53e82`
    `fix(pal): confirm bulk release of working pals (#993)`
- Python files:
  - `toolbear_ui/frontend/src/views/PalsView.vue`
  - `toolbear_ui/frontend/src/views/__tests__/PalsView.bulkActions.test.ts`
- Rust status:
  - No ToolBear frontend or pal UI is present in lilium-ng.
- Required future behavior:
  - Bulk release confirmation must warn when selected pals are working and state
    that release first unassigns them.
- Priority after pal UI migration: medium.

## Not Ported Now

### Xiangqi game design docs

- Python commit:
  - `c5f40df52960a8eb933f3cdab40f28ccbd53fe3f`
    `docs(xiangqi): add game design`
- Python files:
  - `docs/plans/2026-06-14-xiangqi-game-design.md`
  - `docs/plans/2026-06-14-xiangqi-implementation-plan.md`
- Rust status:
  - lilium-ng has no game-worker, ToolBear frontend, or bot command surface for
    Xiangqi.
- Decision:
  - Do not port in the current Rust rewrite scope. Revisit when game-worker or
    bot-game integration is brought into lilium-ng.
