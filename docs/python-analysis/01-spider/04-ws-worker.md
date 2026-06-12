# spider/ws_worker.py

## 功能
单账户 worker 进程入口点。

## 函数
```python
def _build_parser() -> argparse.ArgumentParser
```
- 构建 CLI 参数解析器
- 参数: --account-user-id, --runtime-dir, --buffer-dir, --queue-size

```python
async def main() -> int
```
- 加载账户凭据
- 创建 AccountWorker
- 运行 worker

## 依赖模块
- `database.app_name.override_process_app_name`
- `database.async_engine.get_async_session`
- `services.account_service.AccountService`
- `services.websocket_connection_service.WebSocketConnectionLockConflictError`
- `spider.ws_control.validate_account_user_id`
- `spider.ws_exit_codes.WORKER_LOCK_CONFLICT_EXIT_CODE`
- `spider.ws_runtime.AccountWorker`
- `utils.setup_logging`
- `utils.sentry.init_backend_sentry`

## 数据流
```
main()
  → AccountService.get_account()  [DB 查询]
  → AccountService.create_auth_client()  [创建 API 客户端]
  → AccountWorker(account_user_id, buffer_path, queue_size, auth, runtime_dir)
  → worker.run()
```

## Rust 映射
- 位置: `binaries/lilium-spider/src/worker/mod.rs`
- 状态: ⚠️ 框架实现（缺少 AccountWorker 完整逻辑）
