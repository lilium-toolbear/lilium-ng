# Python → Rust 文件功能映射表

## Spider 子系统

### ws_arbiter.py → lilium-spider/src/arbiter/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebSocketArbiter 类 | Arbiter struct | ✅ |
| scan_and_update_workers() | scan_and_update_workers() | ✅ |
| start_account_worker() | start_worker() | ✅ |
| stop_account_worker() | stop_worker() | ✅ |
| reload_account_worker() | reload_worker() | ✅ |
| handle_command() | handle_command() | ✅ |
| run() 主循环 | run() | ✅ |
| 信号处理 (SIGINT/SIGTERM) | 信号处理 | ✅ |
| 工作进程监控 | monitor_worker_lifecycle() | ✅ |
| Unix socket 控制 | 控制协议 | ✅ |

### ws_worker.py → lilium-spider/src/worker/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| AccountWorker 类 | Worker struct | ✅ |
| WebSocket 连接 | run_websocket() | ✅ |
| 事件入队 | EventIngestor | ✅ |
| 控制命令处理 | handle_control_command() | ✅ |
| 优雅关闭 | 优雅关闭 | ✅ |

### ws_runtime.py → lilium-spider/src/worker/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| Socket.IO 连接 | WebSocket 连接 | ✅ |
| 事件处理 | process_event() | ✅ |
| 心跳 | 心跳 | ✅ |
| 热交换重连 | 重连 | ✅ |

### ws_ingestion.py → lilium-spider/src/ingestion.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| EventEnvelope 数据结构 | EventEnvelope | ✅ |
| EventIngestor 类 | EventIngestor | ✅ |
| EventWriter 类 | EventWriter | ✅ |
| DiskSpillBuffer 类 | DiskSpillBuffer | ✅ |
| accept_event() | accept_event() | ✅ |
| drain_once() | drain_once() | ✅ |
| 并发安全 (asyncio.Lock) | Mutex | ✅ |
| 磁盘溢出逻辑 | 磁盘溢出 | ✅ |
| 批量回放 | read_replay_batch() | ✅ |
| 批量丢弃 | discard_replay_batch() | ✅ |

### ws_control.py → lilium-spider/src/control.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| ControlCommand 数据结构 | ControlCommand | ✅ |
| ControlResponse 数据结构 | ControlResponse | ✅ |
| ControlAction 类型 | ControlAction enum | ✅ |
| validate_account_user_id() | validate_account_user_id() | ✅ |
| to_json() | to_json() | ✅ |
| from_json() | from_json() | ✅ |
| ACCOUNT_ACTIONS 常量 | ACCOUNT_ACTIONS | ✅ |
| ARBITER_ACTIONS 常量 | ARBITER_ACTIONS | ✅ |

### event_processor.py → lilium-spider/src/processor/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| EventProcessor 类 | EventProcessor | ✅ |
| run() 主循环 | run() | ✅ |
| _process_event_list() | process_batch() | ✅ |
| _process_events_individually() | skip_batch() | ✅ |
| _process_event() | process_event() | ✅ |
| message:new 处理 | ✅ | 完整 |
| message:updated 处理 | ✅ | 完整 |
| message:deleted 处理 | ✅ | 完整 |
| message:recalled 处理 | ✅ | 完整 |
| _fetch_collected_users() | user_service.batch_fetch_and_update() | ✅ |
| _download_media_batch() | media_service.download_media_batch() | ✅ |
| 重试逻辑 (指数退避) | process_batch_with_retry() | ✅ |
| 事务批处理 | process_batch_transactional() | ✅ |
| 偏移量追踪 | event_service.save_cursor() | ✅ |

### connection_cleanup.py → 未实现

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| ConnectionCleanupDaemon 类 | 未实现 | ❌ |
| 过期连接清理 | 未实现 | ❌ |

### explore_sync.py → 未实现

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| ExploreFeedSync 类 | 未实现 | ❌ |
| 探索流同步 | 未实现 | ❌ |

## 服务层

### services/message_service.py → lilium-services/src/message.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| MessageService 类 | MessageService | ✅ |
| create_message_if_missing() | create_message() | ✅ |
| update_message() | update_message() | ✅ |
| mark_deleted() | mark_deleted() | ✅ |
| mark_recalled() | mark_recalled() | ✅ |
| get_by_id_at() | get_by_id_at() | ✅ |
| add_to_history() | 在模型中实现 | ✅ |
| 房间成员追踪 | RoomMemberService | ✅ |

### services/user_service.py → lilium-services/src/user.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| UserService 类 | UserService | ✅ |
| batch_fetch_and_update_users() | batch_fetch_and_update() | ✅ |
| get_by_ids() | fetch_user_profile() | ✅ |

### core/media.py → lilium-services/src/media.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| MediaDownloader 类 | MediaService | ✅ |
| download_media_batch() | download_media_batch() | ✅ |
| download_avatar() | download_single_media() | ✅ |
| 并发下载 (Semaphore) | Semaphore 并发 | ✅ |
| 文件存储 | 文件存储 | ✅ |

### services/notification_service.py → lilium-database/src/notifications.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| NotificationService 类 | NotificationManager | ✅ |
| stream_with_polling() | NOTIFY + 轮询 fallback | ✅ |
| wait_for_notification() | subscribe() | ✅ |

## 数据层

### models/wallet/wallet.py → lilium-models/src/wallet/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| Wallet SQLModel | Wallet struct | ✅ |
| snapshot_balance | snapshot_balance | ✅ |
| snapshot_escrow_balance | snapshot_escrow_balance | ✅ |
| snapshot_tx_id | snapshot_tx_id | ✅ |
| allow_negative_balance | allow_negative_balance | ✅ |

### models/wallet/wallet_transaction.py → lilium-models/src/wallet/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WalletTransaction SQLModel | WalletTransaction struct | ✅ |
| TransactionType 枚举 | TransactionType enum | ✅ |
| amount, escrow_delta | amount, escrow_delta | ✅ |
| counterparty_id | counterparty_id | ✅ |
| tx_group_id | tx_group_id | ✅ |

### models/ingestion/websocket_event.py → lilium-models/src/ingestion/mod.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebSocketEvent SQLModel | WebSocketEvent struct | ✅ |
| EventEnvelope | EventEnvelope | ✅ |
| EventProcessorOffset | EventProcessorOffset | ✅ |

### models/dzmm/message.py → lilium-models/src/dzmm/message.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| Message SQLModel | Message struct | ✅ |
| from_websocket() | from_websocket() | ✅ |
| from_api() | 未实现 | ❌ |
| add_to_history() | add_to_history() | ✅ |
| mark_deleted() | 在服务层实现 | ✅ |
| mark_recalled() | 在服务层实现 | ✅ |

### database/async_engine.py → lilium-database/src/pool.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| get_async_session() | DbPool::connect() | ✅ |
| 连接池配置 | PgPoolOptions | ✅ |

### database/notification.py → lilium-database/src/notifications.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| notification_manager | NotificationManager | ✅ |
| subscribe() | subscribe() | ✅ |
| unsubscribe() | unsubscribe() | ✅ |

## 统计

| 模块 | Python 文件 | Rust 文件 | 功能覆盖率 |
|------|-------------|-----------|-----------|
| ws_arbiter.py | 576 行 | arbiter/mod.rs | 95% |
| ws_worker.py | 86 行 | worker/mod.rs | 90% |
| ws_runtime.py | 582 行 | worker/mod.rs | 85% |
| ws_ingestion.py | 251 行 | ingestion.rs | 95% |
| ws_control.py | 293 行 | control.rs | 95% |
| event_processor.py | 761 行 | processor/mod.rs | 90% |
| message_service.py | 1071 行 | message.rs | 85% |
| user_service.py | 500+ 行 | user.rs | 80% |
| media.py | 795 行 | media.rs | 70% |
| notification_service.py | 329 行 | notifications.rs | 85% |
| **总计** | **~5000 行** | **~2000 行** | **~85%** |

## 未实现的文件

| Python 文件 | 说明 | 优先级 |
|-------------|------|--------|
| connection_cleanup.py | 过期连接清理 | 低 |
| explore_sync.py | 探索流同步 | 低 |
| services/turnip_service.py | 大头菜服务 | 中 |
| services/futures_service.py | 期货服务 | 中 |
| services/wallet_service.py | 钱包双重记账 | 高 |
