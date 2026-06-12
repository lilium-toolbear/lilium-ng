# spider/ws_arbiter.py

## 功能
主进程监督器，管理每账户的 worker 进程。

## 类

### WebSocketArbiter

**构造参数**:
- `socket_path: Path` - 控制 socket 路径
- `runtime_dir: Path | None` - 运行时目录
- `start_worker: StartWorker | None` - 启动 worker 的回调
- `stop_worker: StopWorker | None` - 停止 worker 的回调
- `forward_to_worker: ForwardToWorker | None` - 转发命令到 worker
- `list_account_user_ids: ListAccountUserIds | None` - 列出账户 ID
- `worker_monitor_interval: float` - 监控间隔（默认 30s）
- `worker_control_timeout: float` - 控制超时（默认 3s）
- `worker_shutdown_timeout: float` - 关闭超时（默认 10s）

**关键状态**:
- `worker_handles: dict[str, WorkerHandle]` - 账户到 worker 的映射
- `_shutdown_event: asyncio.Event` - 关闭信号
- `_lifecycle_lock: asyncio.Lock` - 生命周期锁
- `_manually_stopped_accounts: set[str]` - 手动停止的账户
- `_restart_suppressed_accounts: set[str]` - 抑制重启的账户

**方法**:
- `scan_and_update_workers(retry_suppressed=False) -> ControlResponse`
  - 从 DB 加载启用的账户
  - 计算 desired = enabled - manually_stopped - restart_suppressed
  - 停止死亡的 worker，启动缺少的 worker，停止多余的 worker

- `start_account_worker(account_user_id) -> WorkerHandle`
  - 启动指定账户的 worker

- `stop_account_worker(account_user_id)`
  - 停止指定账户的 worker

- `manually_stop_account_worker(account_user_id)`
  - 手动停止（会被追踪，不会自动重启）

- `reload_account_worker(account_user_id)`
  - 重新加载 worker（停止 + 启动）

- `handle_command(command: ControlCommand) -> ControlResponse`
  - 处理控制命令：status, reconnect, reload, restart, start, stop, rescan

- `run()`
  - 主循环：启动信号处理、扫描 worker、运行控制服务器、监控 worker 生命周期

- `monitor_worker_lifecycle()`
  - 每 30s 扫描一次 worker 状态

- `run_control_server()`
  - 运行 Unix socket 控制服务器

**依赖模块**:
- `database.async_engine.get_async_session`
- `services.account_service.AccountService`
- `spider.ws_control.*`
- `spider.ws_exit_codes.WORKER_LOCK_CONFLICT_EXIT_CODE`
- `utils.setup_logging`
- `utils.sentry.init_backend_sentry`

## 数据流
```
main() 
  → arbiter.run()
    → scan_and_update_workers()
      → _load_enabled_account_user_ids()  [DB 查询]
      → _start_account_worker()  [启动子进程]
      → _stop_account_worker()  [停止子进程]
    → run_control_server()  [Unix socket 监听]
    → monitor_worker_lifecycle()  [每 30s 扫描]
```

## Rust 映射
- 位置: `binaries/lilium-spider/src/arbiter/mod.rs`
- 状态: ✅ 基本实现（缺少完整的 worker 管理逻辑）
