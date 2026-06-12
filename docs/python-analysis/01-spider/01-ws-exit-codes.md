# spider/ws_exit_codes.py

## 功能
定义退出码常量，用于 worker 和 arbiter 之间的通信。

## 常量
```python
WORKER_LOCK_CONFLICT_EXIT_CODE = 75
```

## 依赖模块
无

## 被引用
- ws_arbiter.py: 检查 worker 退出码
- ws_worker.py: 使用此退出码退出

## Rust 映射
- 位置: `crates/lilium-common/src/constants.rs`
- 状态: ✅ 已实现
