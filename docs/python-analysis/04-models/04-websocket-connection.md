# models/ingestion/websocket_connection.py

## 功能
WebSocket 连接跟踪模型，使用 advisory locks 实现崩溃安全。

## 类

### WebSocketConnection(SQLModel, table=True)

**表名**: `websocket_connections`

**字段**:
- `lock_id: int` - PostgreSQL advisory lock ID（主键，BIGINT）
- `account_user_id: str` - 账户用户 ID（外键）
- `connected_at: datetime` - 连接时间
- `last_heartbeat: datetime` - 最后心跳时间

**Advisory Lock 集成**:
- lock_id 对应 pg_advisory_lock(lock_id)
- 进程崩溃时锁自动释放
- 每个凭据一个连接（pg_try_advisory_lock）

**监控**:
- 查询 pg_locks 验证锁状态
- 检测崩溃进程

**生命周期**:
1. Connect: 获取 advisory lock → INSERT
2. Heartbeat: 周期性更新 → UPDATE last_heartbeat
3. Disconnect (graceful): 释放锁 → DELETE
4. Disconnect (crash): 锁自动释放（无 DELETE）

## 依赖模块
- `sqlmodel.Field, SQLModel, Column`
- `sqlalchemy.TIMESTAMP, BigInteger`
- `models.common.base.utc_now`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
