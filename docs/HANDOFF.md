# Lilium NG - 交接文档

## 会话背景

**目标**: 用 Rust 重写 DZMM Python 系统，从 spider 开始
**代码库**: `kuma-dzmm/lilium-ng` (本地 `~/Working/github/dzmm-spider-rust/`)
**Git**: branch `master`, author `kuma <kuma@kuma.homes>`

## 当前状态

### Rust 实现 (46 测试通过)
```
crates/
├── lilium-common/      # 错误类型, 常量, 工具函数
├── lilium-models/      # 数据模型 (message, user, room, wallet, ingestion)
├── lilium-database/    # 数据库连接, 查询
├── lilium-core/        # 纯业务逻辑 (pal_work, work_efficiency, land_bonus)
└── lilium-services/    # 服务层 (message, event, room_member, user, media)

binaries/
├── lilium-spider/      # arbiter + worker
└── lilium-event-processor/  # 事件处理器
```

### Python 分析 (27+ 文件已分析)
```
docs/python-analysis/
├── 01-spider/        # 11 files ✅
├── 02-services/      # 11 files ✅
├── 03-core/          # 2 files ✅
├── 04-models/        # 17 files ✅
├── 05-database/      # 2 files ✅
├── 06-dzmm-client/   # 2 files ✅
├── RUST_REVIEW.md    # 对比报告
└── MODEL_DDL.sql     # 从 model 生成的真实 DDL (2892行)
```

## 关键规则 (必须遵守)

1. **SQL 生成**: 只能用 `CreateTable(table).compile(dialect=pg_dialect())` 从 model metadata 生成。禁止用迁移脚本，禁止编造 SQL。
2. **文件分析**: 每个 Python 文件必须逐行分析，写入单独文档，全部完成后才能写 Rust。
3. **全仓库迁移**: 不只是 spider，所有 Python 服务都要重写。
4. **手动验收**: 每个 Rust 文件必须与 Python 对比，确保功能一致、接口统一、测试完整。
5. **垃圾文件清理**: SCHEMA.md, REAL_SCHEMA.sql, ALEMBIC_SQL.sql 已删除，只有 MODEL_DDL.sql 保留。

## 待修复的 Rust 差距

1. **SQL 是编造的** — 必须使用 MODEL_DDL.sql 作为参考
2. **lilium-common** — 缺少 FixedDecimal 类型 (DECIMAL(38,2))
3. **lilium-models** — 缺少 room_member.rs, types.rs
4. **lilium-database** — 缺少 GuardedAsyncSession, 测试安全检查
5. **lilium-services** — 缺少 account_service, websocket_connection_service
6. **lilium-api-client** — 缺少完整认证逻辑, tRPC 批量请求

## 下一步

1. 修复 Rust 代码中的 SQL (使用 MODEL_DDL.sql)
2. 补全缺失的模块
3. 确保测试逻辑与 Python 一致
4. 手动验收每个 Rust 文件 vs Python 对应文件

## 关键文件位置

- 项目根目录: `~/Working/github/dzmm-spider-rust/`
- Rust 源码: `crates/` 和 `binaries/`
- Python 分析: `docs/python-analysis/`
- 模型 DDL: `docs/python-analysis/04-models/MODEL_DDL.sql`
- 对比报告: `docs/python-analysis/RUST_REVIEW.md`
- 项目 memory: `/Users/bearice/.local/share/mimocode/memory/projects/89e3498e-a67b-4a4b-aad7-50544854d004/MEMORY.md`
