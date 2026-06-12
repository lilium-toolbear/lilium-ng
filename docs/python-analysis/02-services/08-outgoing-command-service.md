# services/outgoing_command_service.py

## 功能
出站命令队列服务，处理 WebSocket 消息发送。

## 类

### OutgoingCommandService(AsyncBaseService)

**类级常量**:
- `_TERMINAL_STATUSES` - 终态状态
- `_STANDARD_MAX_ATTEMPTS = 3` - 标准最大尝试次数
- `_MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS = 6` - 消息发送限流最大尝试次数
- `_RATE_LIMIT_ERROR_MARKERS` - 限流错误标记

**方法**:
- `async create_command(account_user_id, event, data, require_ack=True, max_attempts=None) -> OutgoingCommand`
  - 创建新的出站命令

- `async get_pending_commands(account_user_id, limit=100) -> List[OutgoingCommand]`
  - 获取待处理命令（FIFO）

- `async mark_processing(command_id) -> None`
  - 标记命令为处理中

- `async mark_success(command_id, response=None) -> None`
  - 标记命令为成功

- `async mark_failed(command_id, error_message) -> None`
  - 标记命令为失败

- `async mark_timeout(command_id) -> None`
  - 标记命令为超时

- `async retry_or_fail(command_id, error_message) -> None`
  - 重试或标记为失败

## 依赖模块
- `sqlalchemy.delete`
- `sqlmodel.select, col`
- `models.ingestion.outgoing_command.OutgoingCommand, OutgoingCommandStatus`
- `services.base.AsyncBaseService`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
