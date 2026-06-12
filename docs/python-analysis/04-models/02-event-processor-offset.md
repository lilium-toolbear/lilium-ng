# models/ingestion/event_processor_offset.py

## 功能
事件处理器偏移量追踪模型，用于 Kafka 风格队列管理。

## 类

### EventProcessorOffset(SQLModel, table=True)

**表名**: `event_processor_offsets`

**字段**:
- `processor_id: str` - 处理器 ID（主键）
- `last_processed_id: int` - 最后处理的事件 ID
- `last_processed_timestamp: Optional[datetime]` - 最后处理的事件时间戳
- `last_processed_at: Optional[datetime]` - 最后处理的时间
- `updated_at: datetime` - 更新时间

**设计**:
- Kafka 风格偏移量追踪
- 事件保留在 websocket_events 表中（不删除）
- 每个处理器通过 (timestamp, id) 跟踪位置

## 依赖模块
- `sqlalchemy.TIMESTAMP, Column`
- `sqlmodel.Field, SQLModel`
- `models.common.base.utc_now`

## Rust 映射
- 位置: `crates/lilium-models/src/ingestion/mod.rs`
- 状态: ✅ 已实现
