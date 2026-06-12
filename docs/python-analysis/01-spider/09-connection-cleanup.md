# spider/connection_cleanup.py

## 功能
连接清理守护进程，定期清理陈旧的 WebSocket 连接记录。

## 类

### ConnectionCleanupDaemon

**构造参数**:
- `cleanup_interval: float` - 清理间隔（默认 60.0 秒）
- `timeout_seconds: int` - 超时阈值（默认 300 秒 = 5 分钟）

**关键状态**:
- `_shutdown: bool` - 关闭标志
- `total_cleaned: int` - 总清理数
- `cleanup_runs: int` - 清理运行次数

**方法**:
- `async run_cleanup()` - 运行单次清理
  - 调用 `connection_service.cleanup_stale_connections()`
  - 更新统计信息

- `async run()` - 主运行循环
  - 周期性调用 run_cleanup()
  - 每 60 秒执行一次

- `shutdown()` - 信号优雅关闭

## 清理逻辑
1. 查询心跳时间超过阈值的连接记录
2. 检查 PostgreSQL advisory lock 是否仍被持有
3. 删除锁已释放的连接记录
4. 记录清理操作摘要

## 依赖模块
- `database.async_engine.get_async_session`
- `services.websocket_connection_service.WebsocketConnectionService`
- `utils.setup_logging`
- `utils.sentry.init_backend_sentry`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
