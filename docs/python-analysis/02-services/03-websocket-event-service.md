# services/websocket_event_service.py

## 功能
WebSocket 事件队列服务，提供事件队列操作。

## 数据结构

### WebSocketEventInsert(TypedDict)
```python
class WebSocketEventInsert(TypedDict):
    user_id: str
    event: str
    data: Dict[str, Any]
    timestamp: datetime
```

## 类

### WebSocketEventService(AsyncBaseService)

**方法**:
- `async insert_event(user_id, event, data, timestamp=None) -> WebSocketEvent`
  - 插入单个事件
  - 返回创建的事件对象

- `async insert_events(events: Sequence[WebSocketEventInsert]) -> int`
  - 批量插入事件
  - 返回插入的行数

- `async get_pending_events(limit=100, user_id=None, event_type=None) -> List[WebSocketEvent]`
  - 获取待处理事件（FIFO 顺序）
  - 支持 user_id 和 event_type 过滤

- `async get_events_after_offset(last_processed_id=0, last_processed_timestamp=None, limit=100, user_id=None, event_type=None) -> List[WebSocketEvent]`
  - Kafka 风格偏移量消费
  - 使用 (timestamp, id) 元组游标

- `async delete_event(event_id: int) -> None`
  - 删除已处理的事件

- `async get_queue_depth() -> int`
  - 获取队列深度

- `async get_oldest_event_age() -> Optional[timedelta]`
  - 获取最旧事件的年龄

- `async get_max_event_id() -> Optional[int]`
  - 获取最大事件 ID

- `async get_latest_event_cursor() -> tuple[Optional[datetime], int]`
  - 获取最新游标

- `async get_latest_event(user_id=None, event_type=None) -> Optional[WebSocketEvent]`
  - 获取最新事件

- `async get_latest_timestamp_for_id(event_id: int) -> Optional[datetime]`
  - 获取事件 ID 的时间戳

## 依赖模块
- `sqlmodel.select, func, col, delete`
- `sqlalchemy.insert, tuple_`
- `models.ingestion.websocket_event.WebSocketEvent`
- `services.base.AsyncBaseService`

## Rust 映射
- 位置: `crates/lilium-services/src/event.rs`
- 状态: ⚠️ 部分实现（只有 get_events_after_offset 和 load_cursor/save_cursor）
