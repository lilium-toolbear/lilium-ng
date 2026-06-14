# Lilium NG - 交接文档

## 测试迁移状态更新（2026-06-13）

当前主线不应再以“测试数量接近”判断完成度。新的测试补齐目标和模块矩阵见 `docs/TEST_MIGRATION_GOAL.md`。

已完成第一批 no-DB parity slice：
- 从 Python `tests/unit/models/test_message_parsing.py` 迁移 `Message.from_websocket` 的核心解析行为：wrapped event、snake_case 字段、image attachment、video metadata、sticker、reference。
- Rust 实现同步补齐 `crates/lilium-models/src/dzmm/message.rs` 的解析逻辑。
- 验证：`cargo test -p lilium-models` 通过，`cargo test -p lilium-event-processor` 通过。

仍需继续：
- `Message.from_websocket` 剩余 Python 行为：invalid-event errors、system/recalled/edited/all-sample fixture coverage。
- `Message.from_websocket` 的 remaining 行为测试已继续补齐：新增 invalid-event、system、recalled、edited 及 10 条样本循环覆盖（text/image/sticker/reference）。
- Models 侧 account/websocket_connection/wallet 的 Python 测试分类和迁移。
- Event/offset service、Account service 目前 Rust 测试为 0，需要先建立 DB harness 再补齐服务级 parity。

## 测试迁移状态更新（2026-06-14）

当前 workspace 的 Rust 测试已经全绿，数据库测试统一走 `lilium-database` 的事务会话夹具，并且只读取 `TEST_DATABASE_URL`：

- `cargo test --workspace --no-fail-fast` 通过
- `#[ignore]` 的 DB 服务测试已清空
- `lilium-services` 的测试入口已统一到 `lilium-database::test_fixtures::with_db_session(...)`
- 需要 `DbPool` 的场景统一走 `with_db_session_and_pool(...)`

## 会话背景

**目标**: 用 Rust 重写 DZMM Python 系统
**仓库**: `kuma-dzmm/lilium-ng`（本地 `~/Working/github/lilium-ng/`）
**Python 源码**: `/Users/bearice/Working/github/dzmm_archive/`
**Git**: branch `master`, author `kuma <kuma@kuma.homes>`

## 当前状态（2026-06-14）

### 编译：零错误 ✅
```
cargo check          -> 0 errors
```

### 测试：workspace 全绿 ✅
```
cargo test --workspace --no-fail-fast -> 通过
api-client:      121 passed / 0 failed / 0 ignored
core:             38 passed / 0 failed / 0 ignored
models:           17 passed / 0 failed / 0 ignored
services:        201 passed / 0 failed / 0 ignored
spider:           20 passed / 0 failed / 0 ignored
```

### Rust 实现（全部 Python 服务已翻译）
```
crates/
├── lilium-common/       # 错误类型, 常量, 工具函数
├── lilium-models/       # 数据模型 (message 24字段, user 18字段, room 22字段,
│   dzmm/                #   room_member, account, websocket_connection, 
│   wallet/              #   wallet, wallet_transaction, ingestion)
│   ingestion/
├── lilium-database/     # PgPool + queries (messages, events, wallet, accounts)
├── lilium-core/         # 纯业务逻辑 (pal_work, work_efficiency, land_bonus)
├── lilium-services/     # 全部 9 个服务
│   message_service,
│   event (WebSocketEventService + EventProcessorOffsetService),
│   room_member, user, media,
│   account_service, websocket_connection_service,
│   outgoing_command_service, notification_service
└── lilium-api-client/   # DZMMApi 完整翻译 (2381行) + WebSocketEventDecoder

binaries/
├── lilium-spider/       # arbiter + worker (20 tests)
└── lilium-event-processor/  # event processor (20 tests)
```

### 代码 vs Python 对齐状态

| 模块 | Python 文件 | Rust 文件 | 状态 |
|------|------------|-----------|------|
| API client | `dzmm_client/api.py` (2349行) | `api-client/src/http.rs` (2393行) | ✅ 完整翻译 |
| API client | `dzmm_client/websocket.py` (188行) | `api-client/src/websocket.rs` (360行) | ✅ 完整翻译 |
| Message Service | `services/message_service.py` (1071行) | `services/message.rs` (1372行) | ✅ 完整翻译 |
| User Service | `services/user_service.py` (423行) | `services/user.rs` (294行) | ✅ 完整翻译 |
| Room Member | `services/room_member_service.py` (486行) | `services/room_member.rs` (151行) | ✅ 完整翻译 |
| Event Service | `services/websocket_event_service.py` (316行) | `services/event.rs` (364行) | ✅ 完整翻译 |
| Offset Service | `services/event_processor_offset_service.py` (125行) | `services/event.rs` | ✅ 合并到 event.rs |
| Account Service | `services/account_service.py` (497行) | `services/account_service.rs` (289行) | ✅ 完整翻译 |
| WS Connection | `services/websocket_connection_service.py` (427行) | `services/websocket_connection_service.rs` (398行) | ✅ 完整翻译 |
| Outgoing Command | `services/outgoing_command_service.py` (309行) | `services/outgoing_command_service.rs` (737行) | ✅ 完整翻译 |
| Notification | `services/notification_service.py` (329行) | `services/notification_service.rs` (281行) | ✅ 完整翻译 |
| Core | `core/pal_work.py` 等 | `lilium-core/src/*.rs` | ✅ 完整翻译 |
| Models | 全部 model 文件 | `lilium-models/src/**/*.rs` | ✅ 对齐 DDL |

## 关键规则（来自历史 session）

1. **禁止编造 SQL**: 用 `docs/python-analysis/04-models/MODEL_DDL.sql`（2892行）作为唯一参考。
2. **禁止用迁移脚本**: 项目不使用 sqlx migrate，使用 MODEL_DDL.sql 管理 schema。
3. **手动验收**: 每个 Rust 文件必须与 Python 对比功能一致性。
4. **Python→Rust 翻译**: 必须读真实 Python 源码 `/Users/bearice/Working/github/dzmm_archive/`，不能用分析文档 `docs/python-analysis/`。
5. **Agent 不要自己修代码**: 编译错误 / 测试失败交给 agent 处理，不要手动修。

## 待修复

### 当前已清空
- 之前记录的 8 个失败测试已被修复。
- 之前记录的 97 个 `#[ignore]` DB 测试已收敛为可执行测试。
- 当前仍需继续做的是 Python 侧剩余测试的逐条语义对齐审核，而不是继续堆叠忽略测试。

## Agent 使用经验

### 成功的模式
- 单个文件翻译：agent 读 Python 源码 → 写 Rust，成功率高
- 代码 review + 修复：agent 读两边的代码然后修
- 编译错误修复：agent 能做

### 失败的模式
- agent 需要读多个大文件（>1000行）时容易卡在 turnCount=0
- 测试翻译比代码翻译难：测试依赖 fixtures/factories/DB，agent 写的是填充测试
- 部分 agent 回报 "orphaned: process restarted"

## Cargo 运行
```bash
# cargo 在 ~/.cargo/bin/cargo 是 broken symlink
# 需要用这个：
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo check
cargo test --workspace --no-fail-fast
```

## 关键文件位置

- 项目根目录: `~/Working/github/lilium-ng/`
- Rust 源码: `crates/` 和 `binaries/`
- Python 源码（读）: `/Users/bearice/Working/github/dzmm_archive/`
- Python 分析（参考）: `docs/python-analysis/`
- 模型 DDL: `docs/python-analysis/04-models/MODEL_DDL.sql`
- Project memory: `/Users/bearice/.local/share/mimocode/memory/projects/0ded274f-899b-4ae0-9083-dbed79d598f2/MEMORY.md`
