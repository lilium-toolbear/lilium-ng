# services/event_processor_offset_service.py

## 功能
事件处理器偏移量追踪服务，实现 Kafka 风格的偏移量追踪。

## 类

### EventProcessorOffsetService(AsyncBaseService)

**方法**:
- `async get_cursor(processor_id: str) -> EventProcessorOffset | None`
  - 获取处理器的游标记录

- `async get_offset(processor_id: str) -> int`
  - 获取最后处理的事件 ID
  - 返回 0（如果从未处理）

- `async update_offset(processor_id, last_processed_id, last_processed_timestamp=None, last_processed_at=None) -> EventProcessorOffset`
  - 更新处理器偏移量
  - 幂等操作 - 重复调用是安全的
  - UPSERT 偏移量记录

- `async delete_offset(processor_id: str) -> None`
  - 删除处理器偏移量记录
  - 用于重置处理器位置

## 依赖模块
- `sqlmodel.select`
- `models.ingestion.event_processor_offset.EventProcessorOffset`
- `services.base.AsyncBaseService`

## Rust 映射
- 位置: `crates/lilium-services/src/event.rs`
- 状态: ⚠️ 部分实现（只有 load_cursor 和 save_cursor）
