# services/notification_service.py

## 功能
PostgreSQL LISTEN/NOTIFY 服务，支持轮询回退。

## 类

### NotificationService

**方法**:
- `async wait_for_notification(channel, timeout=None, poll_callback=None) -> bool`
  - 等待指定通道的通知
  - 支持超时和轮询回退

- `async wait_for_multiple(channels, timeout=None) -> Optional[str]`
  - 等待多个通道中的任意一个通知

- `async stream_with_polling(channel, state, poll_callback, polling_interval=30.0, stop_event=None, initial_poll=False) -> AsyncIterator[List[T]]`
  - NOTIFY 作为唤醒信号 + 轮询获取数据
  - 状态管理：poll_callback 接受和返回状态
  - 初始轮询防止竞态条件

## 依赖模块
- `database.notification.notification_manager`
- `utils.setup_logging`
- `utils.sentry.reset_sentry_propagation_context`

## Rust 映射
- 位置: `crates/lilium-database/src/notifications.rs`
- 状态: ⚠️ 基本实现（缺少 stream_with_polling 完整逻辑）
