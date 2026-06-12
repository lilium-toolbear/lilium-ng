# spider/ws_runtime.py

## 功能
单账户 WebSocket 运行时，管理 Socket.IO 连接生命周期。

## 类

### SocketRuntime

**构造参数**:
- `account_user_id: str` - 账户 ID
- `ingestor: EventIngestor` - 事件接收器
- `auth: DZMMApi | None` - API 认证客户端
- `shutdown_event: asyncio.Event | None` - 关闭信号

**关键状态**:
- `sio: socketio.AsyncClient | None` - Socket.IO 客户端
- `lock_id: int | None` - Advisory lock ID
- `event_count: int` - 事件计数
- `was_connected_before: bool` - 之前是否已连接
- `last_disconnect_time: datetime | None` - 最后断开时间
- `_swap_lock: asyncio.Lock` - 热交换锁
- `_reconnect_requested: asyncio.Event` - 重连请求信号

**方法**:
- `_create_sio_instance() -> socketio.AsyncClient`
  - 创建 Socket.IO 客户端实例
  - 注册 connect/disconnect/connect_error/catch_all 事件处理器

- `_connect_sio_instance(sio) -> None`
  - 认证 (auth.authenticate())
  - 构建 Cookie/Header
  - 连接到 DZMM WebSocket

- `hot_swap_connection(reason) -> bool`
  - 创建新连接
  - 等待新连接稳定 (0.5s)
  - 替换旧连接
  - 断开旧连接

- `process_event(event, args) -> None`
  - 将事件包装为 EventEnvelope
  - 调用 ingestor.accept_event()

- `execute_socket_command(cmd, cmd_svc) -> bool`
  - 执行 WebSocket 命令
  - 支持 ACK 和 fire-and-forget

- `run() -> None`
  - 主循环：心跳、重连、重连请求处理

- `graceful_shutdown() -> None`
  - 释放 advisory lock
  - 断开 Socket.IO
  - 关闭 HTTP 客户端

### AccountWorker

**构造参数**:
- `account_user_id: str`
- `buffer_path: Path`
- `queue_size: int` (默认 5000)
- `auth: DZMMApi | None`
- `runtime_dir: Path | None`

**关键状态**:
- `ingestor: EventIngestor`
- `socket_runtime: SocketRuntime`
- `writer: EventWriter`
- `shutdown_event: asyncio.Event`

**方法**:
- `reconnect(*, reason) -> bool`
  - 委托给 socket_runtime.hot_swap_connection()

- `begin_shutdown() -> None`
  - 停止接受事件
  - 设置关闭信号

- `handle_control_command(command) -> ControlResponse`
  - 处理 status/reconnect/stop 命令

- `run_control_server() -> None`
  - 运行 Unix socket 控制服务器

- `run() -> None`
  - 启动 5 个并发任务：
    1. writer_task - 事件写入
    2. socket_runtime - Socket.IO 连接
    3. outgoing_command_listener - 命令监听
    4. worker_control - 控制服务器
    5. shutdown_wait - 关闭等待

- `outgoing_command_listener() -> None`
  - 使用 NotificationService.stream_with_polling()
  - 监听 outgoing_command_inserted 通道
  - 执行待处理命令

- `_execute_command(cmd, cmd_svc) -> bool`
  - 处理 system:reconnect 命令
  - 委托其他命令给 socket_runtime

- `_insert_events(events) -> int`
  - 批量插入事件到数据库

## 依赖模块
- `database.async_engine.get_async_session`
- `dzmm_client.api.DZMMApi`
- `services.notification_service.NotificationService`
- `services.outgoing_command_service.OutgoingCommandService`
- `services.websocket_connection_service.WebsocketConnectionService`
- `services.websocket_event_service.WebSocketEventService`
- `spider.ws_control.*`
- `spider.ws_ingestion.*`

## 数据流
```
AccountWorker.run()
  ├── writer_task: EventWriter.run()
  ├── socket_runtime: SocketRuntime.run()
  │   ├── acquire_connection_lock()
  │   ├── connect_sio_instance()
  │   ├── heartbeat loop
  │   └── hot_swap_connection() on disconnect
  ├── outgoing_command_listener()
  │   └── NotificationService.stream_with_polling()
  ├── worker_control: run_control_server()
  └── shutdown_wait
```

## Rust 映射
- 位置: `binaries/lilium-spider/src/worker/mod.rs`
- 状态: ⚠️ 框架实现（缺少 Socket.IO 连接、心跳、热交换等核心逻辑）
