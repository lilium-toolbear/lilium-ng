# services/websocket_connection_service.py

## 功能
WebSocket 连接跟踪服务，使用 PostgreSQL advisory locks 实现崩溃安全的连接跟踪。

## 类

### WebSocketConnectionServiceError(DomainServiceError)
- `code = "WEBSOCKET_CONNECTION_INVALID_REQUEST"`

### WebSocketConnectionLockConflictError(ServiceError, ConnectionError)
- 当另一个 WebSocket 客户端已持有账户锁时引发

### WebsocketConnectionService(AsyncBaseService)

**构造参数**:
- `session: AsyncSession`
- `lock_connection: Optional[asyncpg.Connection]` - 已持有 advisory lock 的连接

**方法**:
- `async acquire_connection_lock(user_id: str) -> int`
  - 获取 advisory lock
  - 返回 lock_id
  - 如果锁已被持有，引发 WebSocketConnectionLockConflictError

- `async ensure_connection_lock(user_id, *, expected_lock_id=None) -> int`
  - 确保当前连接仍持有 advisory lock
  - 如果锁连接丢失，重新获取

- `async release_connection_lock(lock_id: int) -> None`
  - 释放 advisory lock

- `async update_heartbeat(lock_id: int) -> None`
  - 更新连接心跳时间

- `async cleanup_stale_connections(timeout_seconds: int = 300) -> int`
  - 清理陈旧连接
  - 返回清理的连接数

- `_calculate_lock_id(account_user_id: str) -> int`
  - 使用 MD5 哈希计算确定性 advisory lock ID

## 依赖模块
- `asyncpg`
- `hashlib`
- `database.async_engine.create_asyncpg_lock_connection`
- `models.ingestion.websocket_connection.WebSocketConnection`
- `services.base.AsyncBaseService`
- `services.errors.DomainServiceError, ServiceError`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
