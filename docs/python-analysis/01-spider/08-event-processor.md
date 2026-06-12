# spider/event_processor.py

## 功能
事件处理器守护进程，从数据库队列消费 WebSocket 事件。

## 类

### EventProcessor

**构造参数**:
- `processor_id: str` - 处理器 ID（默认 "event_processor_main"）
- `polling_interval: float` - 轮询间隔（默认 5.0 秒）
- `batch_size: int` - 批量大小（默认 100）
- `queue_depth_warning: int` - 队列深度警告阈值（默认 1000）
- `max_retries: int` - 最大重试次数（默认 3）
- `initial_retry_delay: float` - 初始重试延迟（默认 1.0 秒）
- `max_retry_delay: float` - 最大重试延迟（默认 60.0 秒）
- `retry_backoff_factor: float` - 指数退避因子（默认 2.0）

**关键状态**:
- `processor_id: str` - 处理器唯一标识
- `processed_count: int` - 已处理事件计数
- `queue_offset: int` - 当前队列偏移量
- `_stop_event: asyncio.Event` - 关闭信号

**方法**:

```python
async def _fetch_collected_users(
    self,
    account_service: AccountService,
    session: AsyncSession,
    user_fetch_collector: List[UserFetchRequest],
) -> None
```
- 按账户分组用户获取请求
- 为每个账户调用 batch_fetch_and_update_users()
- 依赖: `services.account_service.AccountService`, `core.user_sync.batch_fetch_and_update_users`

```python
async def _process_event_list(
    self, events: List[WebSocketEvent], session: AsyncSession
) -> List[str]
```
- 在单个事务中处理事件列表
- 收集用户获取请求
- 收集需要媒体下载的消息 ID
- 更新偏移量
- 返回需要媒体下载的消息 ID 列表
- 依赖: `services.MessageService`, `services.AccountService`, `services.EventProcessorOffsetService`, `services.RoomMemberService`

```python
async def _process_events_individually(
    self, events: List[WebSocketEvent]
) -> List[str]
```
- 逐个处理事件（当批量处理失败时）
- 每个事件独立事务
- 跳过毒事件并更新偏移量
- 依赖: 同 _process_event_list

```python
async def _download_media_batch(self, message_ids: List[str])
```
- 在后台下载媒体（非阻塞）
- 使用 Semaphore 限制并发（最大 10）
- 使用共享 MediaDownloader
- 依赖: `core.media.process_message_media`, `core.media.MediaDownloader`

```python
async def _process_event(
    self,
    event: WebSocketEvent,
    session: AsyncSession,
    message_service: MessageService,
    account_service: AccountService,
    room_member_service: RoomMemberService,
    user_fetch_collector: List[UserFetchRequest],
) -> Optional[str]
```
- 处理单个事件
- 返回消息 ID（如果需要媒体下载）
- 事件类型分发:
  - message:new → 创建消息 + 收集用户 + 检测系统消息
  - message:updated → 更新消息（验证存在性 + 保留历史）
  - message:deleted → 标记删除
  - message:recalled → 标记撤回
  - presence:user-online → 收集用户
  - group:member-joined → 更新房间成员
  - group:member-left → 更新房间成员
- 依赖: `services.MessageService`, `services.RoomMemberService`, `models.dzmm.message.Message`

```python
async def run(self)
```
- 主运行循环
- 加载初始游标
- 使用 NotificationService.stream_with_polling()
- 处理事件批次
- 重试逻辑（指数退避 + 抖动）
- 失败后降级为逐事件处理
- 依赖: `services.NotificationService`, `services.WebSocketEventService`, `services.EventProcessorOffsetService`

```python
def shutdown(self)
```
- 信号优雅关闭

## 依赖模块

### 服务层
- `database.app_name.override_process_app_name`
- `database.async_engine.get_async_session`
- `services.AccountService`
- `services.MessageService`
- `services.WebSocketEventService`
- `services.EventProcessorOffsetService`
- `services.NotificationService`
- `services.RoomMemberService`

### 核心层
- `core.media.process_message_media`
- `core.media.MediaDownloader`

### 模型层
- `models.dzmm.message.Message`
- `models.ingestion.websocket_event.WebSocketEvent`
- `models.common.base.parse_datetime`

### 工具
- `utils.setup_logging`
- `utils.sentry.init_backend_sentry`

## 数据流
```
main()
  → EventProcessor(processor_id)
  → processor.run()
    → 加载初始游标
    → NotificationService.stream_with_polling()
      → poll_new_events()  [查询新事件]
      → _process_event_list()  [批量处理]
        → _process_event()  [单事件处理]
          → message_service.create_message_if_missing()
          → room_member_service.upsert_member_simple()
          → user_fetch_collector.append()
        → _fetch_collected_users()  [批量获取用户]
        → offset_service.update_offset()  [更新偏移量]
      → _download_media_batch()  [后台媒体下载]
```

## Rust 映射
- 位置: `binaries/lilium-spider/src/processor/mod.rs` 和 `binaries/lilium-event-processor/src/processor.rs`
- 状态: ⚠️ 基本实现（缺少完整的事件类型处理、用户获取逻辑、媒体下载逻辑）
