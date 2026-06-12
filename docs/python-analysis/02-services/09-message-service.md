# services/message_service.py

## 功能
消息业务逻辑服务，处理复杂过滤、搜索、分页、数据丰富、消息生命周期。

## 数据结构

### MessageInsertDict(TypedDict)
用于批量插入消息的字典类型。

### FilterResult
```python
@dataclass
class FilterResult:
    conditions: List[Any]
    needs_gps_join: bool
    needs_user_join: bool
    needs_room_join: bool
```

## 类

### MessageServiceError(DomainServiceError)
- `code = "MESSAGE_INVALID_CURSOR"`

### MessageService(AsyncBaseService)

**方法**:

#### 查询方法
- `async get_messages(filters, pagination) -> PaginatedResult`
  - 复杂过滤和搜索
  - 游标分页
  - 数据丰富（连接 users, rooms）

- `async get_by_id(message_id, enrich=True) -> Optional[EnrichedMessage]`
  - 根据 ID 获取消息

- `async get_by_id_at(message_id, sent_at, enrich=True) -> Optional[EnrichedMessage]`
  - 根据复合键获取消息（分区表优化）

- `async get_context(message_id, before_count=10, after_count=10) -> Optional[MessageContextResult]`
  - 获取消息上下文（前后消息）

- `async count_messages(filters) -> int`
  - 计算消息数量

#### 生命周期方法
- `async create_message_if_missing(message) -> bool`
  - 创建消息（如果不存在）

- `async update_message(message) -> None`
  - 更新消息

- `async mark_deleted(message_id, deleted_by=None) -> None`
  - 标记消息为已删除

- `async mark_recalled(message_id) -> None`
  - 标记消息为已撤回

#### 辅助方法
- `async _enrich_single(message) -> EnrichedMessage`
  - 丰富单个消息（连接用户和房间信息）

- `async _load_message_by_key(message_id, sent_at) -> Optional[Message]`
  - 根据复合键加载消息

## 依赖模块
- `sqlalchemy.text`
- `sqlalchemy.dialects.postgresql.insert`
- `sqlmodel.select, func, or_, and_, col, Integer`
- `models.Message, User, Room, ImageGPS`
- `models.common.base.parse_datetime`
- `services.base.AsyncBaseService`
- `services.types.MessageFilters, PaginationParams, EnrichedMessage, MessageStats, MessageContextResult`
- `services.errors.DomainServiceError`

## Rust 映射
- 位置: `crates/lilium-services/src/message.rs`
- 状态: ⚠️ 基本实现（缺少复杂过滤、分页、数据丰富）
