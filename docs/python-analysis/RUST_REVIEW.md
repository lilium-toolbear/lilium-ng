# Rust 实现复查报告

## 概述

对比 Python 代码库和 Rust 实现，检查功能一致性、接口对齐、测试覆盖。

---

## 1. Crate 结构 vs Python 模块

| Rust Crate | Python 模块 | 状态 |
|------------|-------------|------|
| lilium-common | models/common/base.py, utils/ | ⚠️ 简化 |
| lilium-models | models/dzmm/, models/wallet/, models/ingestion/ | ⚠️ 部分实现 |
| lilium-database | database/async_engine.py, database/notification.py | ⚠️ 简化 |
| lilium-core | core/pal_work.py, core/work_efficiency_bonus.py, core/land_bonus.py | ✅ 完整 |
| lilium-services | services/message_service.py, services/event_service.py 等 | ⚠️ 部分实现 |
| lilium-api-client | dzmm_client/api.py, dzmm_client/websocket.py | ⚠️ 简化 |

---

## 2. 关键差距

### 2.1 lilium-common
**Python**: `models/common/base.py` 提供 FixedDecimal2, BigNumber, utc_now, parse_datetime
**Rust**: 只有 utils.rs 提供 utc_now, decimal_to_f64, i64_to_decimal
**差距**: 缺少 FixedDecimal 类型（DECIMAL(38,2)）

### 2.2 lilium-models
**Python**: 159 个模型文件
**Rust**: 11 个模型文件
**差距**: 
- 缺少 room_member.rs
- 缺少 dzmm/types.rs（枚举定义）
- 缺少 dzmm/user_history.rs
- 缺少探索流模型（book, card, chapter 等）
- 缺少 wallet_ids.rs（系统钱包 ID）

### 2.3 lilium-database
**Python**: async_engine.py (761行) + notification.py (452行)
**Rust**: pool.rs (简化) + notifications.rs (简化)
**差距**:
- 缺少 GuardedAsyncSession（会话租约）
- 缺少测试安全检查
- 缺少单例 NotificationManager
- 缺少自动重连

### 2.4 lilium-services
**Python**: 124 个服务文件
**Rust**: 5 个服务文件
**差距**:
- 缺少 account_service.py
- 缺少 websocket_connection_service.py
- 缺少 outgoing_command_service.py
- 缺少 notification_service.py（包装器）
- MessageService 缺少复杂过滤、分页、FTS

### 2.5 lilium-api-client
**Python**: api.py (2349行) + websocket.py (188行)
**Rust**: 简化实现
**差距**:
- 缺少完整的认证逻辑（token refresh, QR code）
- 缺少 tRPC 批量请求
- 缺少反检测 headers
- 缺少 rate limiter

---

## 3. SQL 问题

**严重问题**: Rust 代码中的 SQL 是编造的，不是从 model 生成的。

**正确方法**: 
```python
from sqlalchemy.schema import CreateTable
from sqlalchemy.dialects.postgresql import dialect as pg_dialect

for table_name in sorted(metadata.tables.keys()):
    table = metadata.tables[table_name]
    ddl = CreateTable(table).compile(dialect=pg_dialect())
    print(str(ddl))
```

**已生成**: MODEL_DDL.sql (2892行) 从 model metadata 生成

---

## 4. 测试覆盖

| 模块 | Python 测试 | Rust 测试 | 状态 |
|------|-------------|-----------|------|
| spider | ~30 | 12 | ⚠️ 缺少集成测试 |
| services | ~50 | 0 | ❌ 未测试 |
| core | ~20 | 11 | ✅ 覆盖良好 |
| models | ~10 | 2 | ⚠️ 缺少测试 |

---

## 5. 下一步

1. **修复 SQL**: 使用 MODEL_DDL.sql 作为参考，重写 Rust 中的 SQL
2. **补全 models**: 添加缺失的模型文件
3. **实现完整 services**: 按 Python 代码实现所有服务
4. **添加测试**: 对齐 Python 测试覆盖
5. **集成测试**: 验证 Rust 和 Python 的行为一致性
