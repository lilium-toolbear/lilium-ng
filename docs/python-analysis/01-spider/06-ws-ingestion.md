# spider/ws_ingestion.py

## 功能
有界事件摄取，支持磁盘溢出。

## 类

### EventEnvelope
```python
@dataclass(slots=True)
class EventEnvelope:
    account_user_id: str
    event_type: str
    payload: dict[str, Any]
    received_at: datetime
    source: Literal["socket", "disk_replay"] = "socket"
```

### DiskSpillBuffer
```python
class DiskSpillBuffer:
    def __init__(self, path: Path) -> None
    async def append(self, event: EventEnvelope) -> None
    async def read_replay_batch(self, limit: int) -> list[EventEnvelope]
    async def discard_replay_batch(self, count: int) -> None
    async def has_pending(self) -> bool
```
- 使用 asyncio.Lock 保证并发安全
- schema_version: 2
- JSONL 格式存储

### EventIngestor
```python
class EventIngestor:
    def __init__(self, *, account_user_id: str, max_queue_size: int, spill: DiskSpillBuffer) -> None
    async def accept_event(self, event: EventEnvelope) -> bool
    def stop_accepting(self) -> None
    @property
    def queue_depth(self) -> int
    @property
    def is_accepting(self) -> bool
```
- asyncio.Queue 有界队列
- 队列满时 spill 到磁盘

### EventWriter
```python
class EventWriter:
    def __init__(self, *, ingestor: EventIngestor, insert_batch: InsertBatch, batch_size: int = 100, batch_max_wait: float = 0.25, idle_sleep: float = 0.05) -> None
    async def run(self, stop_event: asyncio.Event) -> None
    async def drain_once(self) -> int
    async def _spill_memory_queue(self) -> None
    async def _insert_replay_batch(self, batch: list[EventEnvelope]) -> int
    def _to_insert(self, event: EventEnvelope) -> WebSocketEventInsert
```
- 优先 drain 磁盘，再 drain 内存队列
- 失败时重新 spill 到磁盘

## 依赖模块
- `services.websocket_event_service.WebSocketEventInsert`

## Rust 映射
- 位置: `binaries/lilium-spider/src/ingestion.rs`
- 状态: ✅ 基本实现（EventWriter.drain_once 内存队列部分未完成）
