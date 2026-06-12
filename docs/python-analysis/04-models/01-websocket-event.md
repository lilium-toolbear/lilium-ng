# models/ingestion/websocket_event.py

## 功能
WebSocket 事件模型，用于持久化事件队列。

## 类

### WebSocketEvent(SQLModel, table=True)

**表名**: `websocket_events`

**字段**:
- `id: Optional[int]` - 主键（BIGINT, Identity）
- `timestamp: datetime` - 时间戳（主键的一部分）
- `user_id: str` - DZMM 用户 ID
- `event: str` - 事件类型
- `data: Dict[str, Any]` - 事件负载（JSONB）

**索引**:
- `ix_websocket_events_timestamp_id` - (timestamp, id)
- `ix_websocket_events_user_id_timestamp_id` - (user_id, timestamp, id)
- `ix_websocket_events_event_timestamp_id` - (event, timestamp, id)

**分区**: `RANGE (timestamp)`

**生命周期**:
1. Insert: WebSocket 客户端接收事件 → INSERT
2. Process: 事件处理器轮询 → SELECT ORDER BY id ASC LIMIT 100
3. Delete: 处理成功后 → DELETE WHERE id = ?

## 依赖模块
- `sqlalchemy BIGINT, TIMESTAMP, Column, Identity, Index`
- `sqlalchemy.dialects.postgresql.JSONB`
- `sqlmodel.Field, SQLModel`
- `models.common.base.utc_now`

## Rust 映射
- 位置: `crates/lilium-models/src/ingestion/mod.rs`
- 状态: ✅ 已实现
