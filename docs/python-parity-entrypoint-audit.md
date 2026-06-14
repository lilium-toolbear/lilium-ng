# Python Parity Audit by Binary Entry Points

日期：2026-06-14
范围：从 Rust binary 入口点开始，对照 `dzmm_archive` 里的真实 Python 源码。
说明：这里只保留当前对照状态和待确认清单，历史差异和旧修复计划已移出。

## 最新状态

- 本轮按真实 Python 源码从 binary entrypoint 向下对照后，之前列出的主要入口差异已经在 Rust 侧实现。
- `lilium-spider` 的入口链、worker / ingestion / control 流程已经从骨架推进到可运行实现。
- `lilium-event-processor` 的主循环、事件覆盖、用户同步和媒体处理也已经从简化路径推进到当前实现。
- 当前文档不再保留旧的差异列表，后续重点放在防止回归和维持接口形状一致。

## 当前清单

- [ ] 触碰 `binaries/lilium-spider/src/worker/mod.rs` 或 `ingestion.rs` 时，重新对照 Python 的 `ws_worker.py` / `ws_runtime.py` / `ws_ingestion.py`。
- [ ] 触碰 `binaries/lilium-event-processor/src/processor.rs` 时，重新对照 Python 的 `event_processor.py` 和 `notification_service.py`。
- [ ] 触碰 `crates/lilium-services/src/user.rs` 或 `media.rs` 时，重新对照 Python 的 user / media 行为，确认外部数据库、JSON 和接口形状仍兼容。
