# Python → Rust 完整功能映射表

## 1. Spider 核心模块

### spider/ws_arbiter.py → binaries/lilium-spider/src/arbiter/mod.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebSocketArbiter.__init__() | Arbiter::new() | ✅ |
| scan_and_update_workers() | scan_and_update_workers() | ✅ |
| start_account_worker() | start_worker() | ✅ |
| stop_account_worker() | stop_worker() | ✅ |
| reload_account_worker() | reload_worker() | ✅ |
| handle_command() | handle_command() | ✅ |
| run() | run() | ✅ |
| monitor_worker_lifecycle() | monitor_worker_lifecycle() | ✅ |
| 信号处理 | 信号处理 | ✅ |
| Unix socket 控制 | 控制协议 | ✅ |

### spider/ws_worker.py → binaries/lilium-spider/src/worker/mod.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| AccountWorker.__init__() | Worker::new() | ✅ |
| run() | run() | ✅ |
| run_writer() | run_writer() | ✅ |
| run_websocket() | run_websocket() | ✅ |

### spider/ws_runtime.py → binaries/lilium-spider/src/worker/mod.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| SocketRuntime.__init__() | 内嵌在 Worker | ✅ |
| run() | run_websocket() | ✅ |
| process_event() | process_event() | ✅ |
| hot_swap_connection() | 重连逻辑 | ✅ |

### spider/ws_ingestion.py → binaries/lilium-spider/src/ingestion.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| EventEnvelope | EventEnvelope | ✅ |
| EventIngestor.__init__() | EventIngestor::new() | ✅ |
| EventIngestor.accept_event() | accept_event() | ✅ |
| EventIngestor.stop_accepting() | stop_accepting() | ✅ |
| EventWriter.__init__() | EventWriter::new() | ✅ |
| EventWriter.run() | run() | ✅ |
| EventWriter.drain_once() | drain_once() | ✅ |
| DiskSpillBuffer.__init__() | DiskSpillBuffer::new() | ✅ |
| DiskSpillBuffer.append() | append() | ✅ |
| DiskSpillBuffer.read_replay_batch() | read_replay_batch() | ✅ |
| DiskSpillBuffer.discard_replay_batch() | discard_replay_batch() | ✅ |
| DiskSpillBuffer.has_pending() | has_pending() | ✅ |

### spider/ws_control.py → binaries/lilium-spider/src/control.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| ControlCommand | ControlCommand | ✅ |
| ControlResponse | ControlResponse | ✅ |
| ControlAction | ControlAction enum | ✅ |
| validate_account_user_id() | validate_account_user_id() | ✅ |
| to_json() | to_json() | ✅ |
| from_json() | from_json() | ✅ |
| write_message() | write_message() | ✅ |
| read_message() | read_message() | ✅ |

### spider/event_processor.py → binaries/lilium-spider/src/processor/mod.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| EventProcessor.__init__() | EventProcessor::new() | ✅ |
| run() | run() | ✅ |
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

### spider/ws_exit_codes.py → crates/lilium-common/src/constants.rs

| Python 常量 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WORKER_LOCK_CONFLICT_EXIT_CODE | WORKER_LOCK_CONFLICT_EXIT_CODE | ✅ |

### spider/ws_client.py → 已弃用

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| 旧版入口点 | 已弃用，委托给 ws_arbiter | N/A |

---

## 2. 服务层依赖

### services/websocket_event_service.py → crates/lilium-services/src/event.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebSocketEventService 类 | EventService | ✅ |
| insert_event() | 未实现 | ❌ |
| insert_events() | 未实现 | ❌ |
| get_pending_events() | 未实现 | ❌ |
| get_events_after_offset() | poll_events() | ✅ |
| delete_event() | 未实现 | ❌ |
| get_queue_depth() | 未实现 | ❌ |
| get_oldest_event_age() | 未实现 | ❌ |
| get_max_event_id() | 未实现 | ❌ |
| get_latest_event_cursor() | 未实现 | ❌ |
| get_latest_event() | 未实现 | ❌ |
| get_latest_timestamp_for_id() | 未实现 | ❌ |

### services/event_processor_offset_service.py → crates/lilium-services/src/event.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| EventProcessorOffsetService 类 | EventService | ✅ |
| get_cursor() | load_cursor() | ✅ |
| get_offset() | 未实现 | ❌ |
| update_offset() | save_cursor() | ✅ |
| delete_offset() | 未实现 | ❌ |

### services/notification_service.py → crates/lilium-database/src/notifications.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| NotificationService 类 | NotificationManager | ✅ |
| wait_for_notification() | subscribe() | ✅ |
| wait_for_multiple() | 未实现 | ❌ |
| stream_with_polling() | NOTIFY + 轮询 fallback | ✅ |

### services/account_service.py → 未实现

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| AccountService 类 | 未实现 | ❌ |
| create_account() | 未实现 | ❌ |
| get_account() | 未实现 | ❌ |
| list_accounts() | 未实现 | ❌ |
| create_auth_client() | 未实现 | ❌ |
| update_password() | 未实现 | ❌ |

### services/websocket_connection_service.py → 未实现

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebsocketConnectionService 类 | 未实现 | ❌ |
| acquire_connection_lock() | 未实现 | ❌ |
| ensure_connection_lock() | 未实现 | ❌ |
| release_connection_lock() | 未实现 | ❌ |
| update_heartbeat() | 未实现 | ❌ |
| cleanup_stale_connections() | 未实现 | ❌ |

### services/outgoing_command_service.py → 未实现

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| OutgoingCommandService 类 | 未实现 | ❌ |
| get_pending_commands() | 未实现 | ❌ |
| mark_command_executed() | 未实现 | ❌ |

### services/message_service.py → crates/lilium-services/src/message.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| MessageService 类 | MessageService | ✅ |
| create_message_if_missing() | create_message() | ✅ |
| update_message() | update_message() | ✅ |
| mark_deleted() | mark_deleted() | ✅ |
| mark_recalled() | mark_recalled() | ✅ |
| get_by_id_at() | get_by_id_at() | ✅ |
| add_to_history() | 在模型中实现 | ✅ |

### services/room_member_service.py → crates/lilium-services/src/room_member.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| RoomMemberService 类 | RoomMemberService | ✅ |
| upsert_member_simple() | upsert_member() | ✅ |
| mark_member_left() | mark_member_left() | ✅ |

### services/user_service.py → crates/lilium-services/src/user.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| UserService 类 | UserService | ✅ |
| batch_fetch_and_update_users() | batch_fetch_and_update() | ✅ |
| get_by_ids() | fetch_user_profile() | ✅ |

---

## 3. Core 层依赖

### core/media.py → crates/lilium-services/src/media.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| MediaDownloader 类 | MediaService | ✅ |
| download_media_batch() | download_media_batch() | ✅ |
| download_avatar() | download_single_media() | ✅ |
| process_message_media() | 未实现 | ❌ |

### core/user_sync.py → crates/lilium-services/src/user.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| batch_fetch_and_update_users() | batch_fetch_and_update() | ✅ |
| fetch_and_update_user() | fetch_user_profile() | ✅ |
| _download_avatars_background() | 未实现 | ❌ |

### core/pal_work.py → crates/lilium-core/src/pal_work.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| calculate_efficiency() | calculate_efficiency() | ✅ |
| calculate_exp_needed() | calculate_exp_needed() | ✅ |
| calculate_exp_gain_per_hour() | calculate_exp_gain_per_hour() | ✅ |
| calculate_turnip_consumption_per_hour() | calculate_turnip_consumption_per_hour() | ✅ |
| get_work_score() | get_work_score() | ✅ |
| is_role_matched() | is_role_matched() | ✅ |

### core/work_efficiency_bonus.py → crates/lilium-core/src/work_efficiency.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| calculate_work_efficiency_multiplier() | calculate_work_efficiency_multiplier() | ✅ |
| calculate_gender_balance() | calculate_gender_balance() | ✅ |

### core/land_bonus.py → crates/lilium-core/src/land_bonus.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| calculate_farm_time_bonus() | calculate_farm_time_bonus() | ✅ |
| calculate_farm_capacity_bonus() | calculate_farm_capacity_bonus() | ✅ |
| calculate_farm_harvest_bonus() | calculate_farm_harvest_bonus() | ✅ |
| calculate_warehouse_capacity_bonus() | calculate_warehouse_capacity_bonus() | ✅ |
| calculate_credit_income_per_hour() | calculate_credit_income_per_hour() | ✅ |
| calculate_resource_level_multiplier() | calculate_resource_level_multiplier() | ✅ |
| calculate_resource_cache_hours() | calculate_resource_cache_hours() | ✅ |
| calculate_resource_worker_multiplier() | calculate_resource_worker_multiplier() | ✅ |
| calculate_resource_income_per_hour() | calculate_resource_income_per_hour() | ✅ |
| calculate_resource_cache_cap() | calculate_resource_cache_cap() | ✅ |
| calculate_dormitory_hatch_speed_bonus() | calculate_dormitory_hatch_speed_bonus() | ✅ |
| calculate_dormitory_breeding_speed_bonus() | calculate_dormitory_breeding_speed_bonus() | ✅ |

---

## 4. Model 层

### models/ingestion/websocket_event.py → crates/lilium-models/src/ingestion/mod.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebSocketEvent | WebSocketEvent | ✅ |
| EventEnvelope | EventEnvelope | ✅ |
| EventProcessorOffset | EventProcessorOffset | ✅ |

### models/ingestion/event_processor_offset.py → crates/lilium-models/src/ingestion/mod.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| EventProcessorOffset | EventProcessorOffset | ✅ |

### models/dzmm/message.py → crates/lilium-models/src/dzmm/message.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| Message | Message | ✅ |
| from_websocket() | from_websocket() | ✅ |
| from_api() | 未实现 | ❌ |
| add_to_history() | add_to_history() | ✅ |
| mark_deleted() | 在服务层实现 | ✅ |
| mark_recalled() | 在服务层实现 | ✅ |

### models/dzmm/user.py → crates/lilium-models/src/dzmm/user.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| User | User | ✅ |

### models/dzmm/room.py → crates/lilium-models/src/dzmm/room.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| Room | Room | ✅ |

### models/wallet/wallet.py → crates/lilium-models/src/wallet/mod.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| Wallet | Wallet | ✅ |
| TransactionType | TransactionType enum | ✅ |

### models/wallet/wallet_transaction.py → crates/lilium-models/src/wallet/mod.rs

| Python 模型 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WalletTransaction | WalletTransaction | ✅ |

### models/wallet/wallet_ids.py → crates/lilium-models/src/wallet/mod.rs

| Python 常量 | Rust 实现 | 状态 |
|-------------|-----------|------|
| FUTURES_MM_TREASURY_USER_ID | ids::FUTURES_MM_TREASURY | ✅ |
| 其他系统钱包 ID | ids:: 模块 | ✅ |

---

## 5. Database 层

### database/async_engine.py → crates/lilium-database/src/pool.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| get_async_session() | DbPool::connect() | ✅ |
| 连接池配置 | PgPoolOptions | ✅ |

### database/notification.py → crates/lilium-database/src/notifications.rs

| Python 功能 | Rust 实现 | 状态 |
|-------------|-----------|------|
| notification_manager | NotificationManager | ✅ |
| subscribe() | subscribe() | ✅ |
| unsubscribe() | unsubscribe() | ✅ |

---

## 6. API Client 层

### dzmm_client/api.py → crates/lilium-api-client/src/http.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| DZMMApi 类 | DzmmClient | ✅ |
| login() | 未实现 | ❌ |
| refresh() | 未实现 | ❌ |
| batch_get_user_info() | 未实现 | ❌ |
| fetch_room_messages() | 未实现 | ❌ |

### dzmm_client/websocket.py → crates/lilium-api-client/src/websocket.rs

| Python 方法 | Rust 实现 | 状态 |
|-------------|-----------|------|
| WebSocketEventDecoder | WsClient | ✅ |
| 连接逻辑 | run() | ✅ |
| 事件解析 | parse_event() | ✅ |

---

## 7. 统计汇总

| 模块 | Python 文件 | Rust 文件 | 功能覆盖率 |
|------|-------------|-----------|-----------|
| **Spider 核心** | | | |
| ws_arbiter.py | 576 行 | arbiter/mod.rs | 95% |
| ws_worker.py | 86 行 | worker/mod.rs | 90% |
| ws_runtime.py | 582 行 | worker/mod.rs | 85% |
| ws_ingestion.py | 251 行 | ingestion.rs | 95% |
| ws_control.py | 293 行 | control.rs | 95% |
| event_processor.py | 761 行 | processor/mod.rs | 90% |
| ws_exit_codes.py | 3 行 | constants.rs | 100% |
| **服务层** | | | |
| websocket_event_service.py | 316 行 | event.rs | 30% |
| event_processor_offset_service.py | 125 行 | event.rs | 60% |
| notification_service.py | 329 行 | notifications.rs | 50% |
| account_service.py | 497 行 | 未实现 | 0% |
| websocket_connection_service.py | 427 行 | 未实现 | 0% |
| outgoing_command_service.py | ~200 行 | 未实现 | 0% |
| message_service.py | 1071 行 | message.rs | 85% |
| room_member_service.py | ~150 行 | room_member.rs | 80% |
| user_service.py | ~500 行 | user.rs | 80% |
| **Core 层** | | | |
| media.py | 795 行 | media.rs | 70% |
| user_sync.py | 549 行 | user.rs | 60% |
| pal_work.py | 131 行 | pal_work.rs | 100% |
| work_efficiency_bonus.py | 77 行 | work_efficiency.rs | 100% |
| land_bonus.py | ~200 行 | land_bonus.rs | 100% |
| **Model 层** | | | |
| websocket_event.py | ~50 行 | ingestion/mod.rs | 100% |
| event_processor_offset.py | ~30 行 | ingestion/mod.rs | 100% |
| message.py | ~300 行 | dzmm/message.rs | 85% |
| user.py | ~100 行 | dzmm/user.rs | 100% |
| room.py | ~100 行 | dzmm/room.rs | 100% |
| wallet.py | ~100 行 | wallet/mod.rs | 100% |
| wallet_transaction.py | ~400 行 | wallet/mod.rs | 100% |
| **Database 层** | | | |
| async_engine.py | ~100 行 | pool.rs | 100% |
| notification.py | ~200 行 | notifications.rs | 80% |

---

## 8. 总体统计

| 类别 | Python 行数 | Rust 行数 | 覆盖率 |
|------|-------------|-----------|--------|
| Spider 核心 | ~2,550 | ~1,200 | 90% |
| 服务层 | ~3,600 | ~800 | 45% |
| Core 层 | ~1,500 | ~600 | 80% |
| Model 层 | ~1,200 | ~500 | 95% |
| Database 层 | ~300 | ~200 | 90% |
| **总计** | **~9,150** | **~3,300** | **65%** |

---

## 9. 关键缺失

### 高优先级（spider 直接依赖）
1. ❌ account_service.py - 账户管理
2. ❌ websocket_connection_service.py - 连接管理
3. ❌ outgoing_command_service.py - 命令服务
4. ❌ websocket_event_service.py 部分方法

### 中优先级（功能完整性）
5. ❌ dzmm_client/api.py - API 客户端
6. ❌ core/media.py 部分功能
7. ❌ core/user_sync.py 部分功能

### 低优先级（非关键路径）
8. ❌ connection_cleanup.py
9. ❌ explore_sync.py
