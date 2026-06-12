# models/ingestion/outgoing_command.py

## 功能
出站命令模型，用于 WebSocket 消息队列。

## 类

### OutgoingCommandStatus(StrEnum)
- `PENDING = "pending"`
- `PROCESSING = "processing"`
- `SUCCESS = "success"`
- `FAILED = "failed"`
- `TIMEOUT = "timeout"`

### OutgoingCommand(SQLModel, table=True)

**表名**: `outgoing_commands`

**字段**:
- `id: Optional[int]` - 自增主键（FIFO）
- `created_at: datetime` - 创建时间
- `account_user_id: str` - 目标账户 ID
- `event: str` - 事件类型
- `data: Dict[str, Any]` - 命令数据（JSONB）
- `require_ack: bool` - 是否需要确认
- `status: str` - 处理状态
- `processed_at: Optional[datetime]` - 处理时间
- `ack_response: Optional[Dict[str, Any]]` - 确认响应
- `error_message: Optional[str]` - 错误消息
- `attempt_count: int` - 尝试次数
- `max_attempts: int` - 最大尝试次数

**生命周期**:
1. Insert: CLI/API 创建命令 → status=pending
2. Pickup: ws_client 轮询 → status=processing
3. Send: ws_client 通过 sio.call()/sio.emit() 发送
4. Complete: 更新 status=success/failed/timeout

## 依赖模块
- `sqlalchemy.TIMESTAMP`
- `sqlalchemy.dialects.postgresql.JSONB`
- `sqlmodel.Column, Field, SQLModel`
- `models.common.base.utc_now`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
