# spider/ws_client.py

## 功能
已弃用的 WebSocket 客户端入口点，委托给 ws_arbiter。

## 函数
```python
async def main(argv: Sequence[str] | None = None) -> int
```
- 委托给 `spider.ws_arbiter.main`
- 初始化 Sentry

## 依赖模块
- `database.app_name.override_process_app_name`
- `spider.ws_arbiter.main`
- `utils.setup_logging`
- `utils.sentry.init_backend_sentry`

## 被引用
无（已弃用入口点）

## Rust 映射
- 位置: 不需要（已弃用）
- 状态: N/A
