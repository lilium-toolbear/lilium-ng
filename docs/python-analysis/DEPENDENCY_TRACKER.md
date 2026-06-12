# 依赖追踪器

## 规则
1. 每个依赖必须指定完整文件路径
2. 必须读取该文件并记录其功能
3. 必须追踪该文件是否已被分析
4. 必须记录该文件在 Rust 中的位置

## 待分析文件队列

### spider/ 模块 (11 files)
- [x] spider/ws_exit_codes.py
- [x] spider/ws_client.py
- [x] spider/ws_arbiter.py
- [x] spider/ws_worker.py
- [x] spider/ws_runtime.py
- [x] spider/ws_ingestion.py
- [x] spider/ws_control.py
- [x] spider/event_processor.py
- [ ] spider/connection_cleanup.py
- [ ] spider/explore_sync.py
- [ ] spider/__init__.py

### services/ 模块 (依赖 spider 的服务)
- [x] services/websocket_event_service.py
- [x] services/event_processor_offset_service.py
- [x] services/notification_service.py
- [x] services/account_service.py
- [x] services/websocket_connection_service.py
- [x] services/outgoing_command_service.py
- [x] services/message_service.py
- [x] services/room_member_service.py
- [x] services/user_service.py
- [x] services/base.py
- [x] services/errors.py

### core/ 模块 (依赖 spider 的核心逻辑)
- [x] core/media.py
- [x] core/user_sync.py
- [ ] core/explore.py

### models/ 模块 (数据模型)
- [ ] models/ingestion/websocket_event.py
- [ ] models/ingestion/event_processor_offset.py
- [ ] models/ingestion/dzmm_account.py
- [ ] models/ingestion/websocket_connection.py
- [ ] models/dzmm/message.py
- [ ] models/dzmm/user.py
- [ ] models/dzmm/room.py
- [ ] models/wallet/wallet.py
- [ ] models/wallet/wallet_transaction.py
- [ ] models/wallet/wallet_ids.py
- [ ] models/common/base.py

### database/ 模块
- [ ] database/async_engine.py
- [ ] database/notification.py
- [ ] database/app_name.py

### dzmm_client/ 模块
- [ ] dzmm_client/api.py
- [ ] dzmm_client/websocket.py
- [ ] dzmm_client/models.py
