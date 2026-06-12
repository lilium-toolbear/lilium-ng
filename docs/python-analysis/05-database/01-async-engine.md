# database/async_engine.py

## 功能
异步 SQLModel 引擎和会话管理。

## 类

### SessionLease
- `lease_id: str` - 租约 ID
- `active: bool` - 是否活跃

### SessionScopeExpiredError(RuntimeError)
- 当会话作用域过期后使用时引发

### GuardedAsyncSession(AsyncSession)
- 在会话作用域过期后使用时快速失败
- 方法: exec, execute, get, commit, rollback, refresh, flush, delete, add

## 函数

```python
def get_async_session() -> AsyncContextManager[AsyncSession]
```
- 获取异步会话上下文管理器

```python
def create_asyncpg_lock_connection(fallback_url: str = None) -> asyncpg.Connection
```
- 创建用于 advisory lock 的 asyncpg 连接

```python
async def get_async_session_dependency() -> AsyncSession
```
- FastAPI 依赖注入

## 常量
```python
APP_NAME = "dzmm_archive"  # 或从 __main__ 推断
```

## 依赖模块
- `asyncpg`
- `sqlalchemy.ext.asyncio.create_async_engine`
- `sqlalchemy.event`
- `sqlalchemy.pool.NullPool`
- `sqlmodel.ext.asyncio.session.AsyncSession`
- `database.database_url.to_asyncpg_database_url`

## Rust 映射
- 位置: `crates/lilium-database/src/pool.rs`
- 状态: ⚠️ 简化实现（缺少 GuardedAsyncSession, 会话租约, 测试安全检查）
