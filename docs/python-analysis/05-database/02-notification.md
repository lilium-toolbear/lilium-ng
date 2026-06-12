# database/notification.py

## 功能
PostgreSQL LISTEN/NOTIFY 基础设施，支持自动重连。

## 类

### Subscription
- `id: str` - 订阅 ID
- `channel: str` - 通道名称
- `handler: Callable` - 处理函数

### _SafeListenerConnectionProxy
- 使 LISTEN 连接关闭对损坏的 socket 具有弹性

### NotificationManager (单例)
- 维护单个共享连接
- 支持每通道多个处理程序（分发器模式）
- 连接丢失时自动重连
- 健康检查防止静默失败

**方法**:
- `async subscribe(channel, handler) -> str` - 订阅通道
- `async unsubscribe(sub_id)` - 取消订阅
- `async publish(channel, payload)` - 发布通知

## 依赖模块
- `asyncpg`
- `asyncpg_listen.NotificationListener`
- `database.async_engine.create_asyncpg_connection`

## Rust 映射
- 位置: `crates/lilium-database/src/notifications.rs`
- 状态: ⚠️ 简化实现（缺少单例模式、自动重连、健康检查）
