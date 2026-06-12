# Rust vs Python 差距分析

## 已修复

### 1. ✅ EventWriter drain_once() 
- 现在正确先 drain 磁盘再 drain 内存队列
- 添加了 test_event_writer_drains_disk_before_memory

### 2. ✅ Control UUID 验证
- 添加了 validate_account_user_id() 函数
- from_json() 现在验证 action 和 account_user_id
- 添加了 5 个 UUID 验证测试

### 3. ✅ DiskSpillBuffer 并发安全
- 添加了 Mutex 保护所有文件操作
- 添加了 test_spill_buffer_concurrent_appends

### 4. ✅ Worker 架构修复
- Worker 不再直接处理事件
- 事件通过 EventIngestor 入队
- 符合 Python 的解耦架构

### 5. ✅ Processor 测试
- 添加了 16 个 processor 测试
- 覆盖所有事件类型分发逻辑
- 包含重试延迟计算测试

### 6. ✅ 服务边界拆分
- 拆分为 lilium-spider (arbiter+worker) 和 lilium-event-processor
- 符合 Python 的多进程微服务架构

### 7. ✅ 移除启动时迁移
- 数据库迁移由 Python Alembic 管理
- Rust 启动时不运行迁移

### 8. ✅ LISTEN/NOTIFY 支持
- 实现了 PostgreSQL LISTEN/NOTIFY
- NOTIFY 作为唤醒信号，轮询获取数据
- 符合 Python 的 stream_with_polling 模式

### 9. ✅ 服务层重构
- 创建了 lilium-services crate
- Processor 调用 services 而不是直接写 SQL
- 修复了依赖方向: processor -> services -> database
- 删除了未使用的 lilium-spider-core crate
- 修复了 SQL 重复问题

## 仍存在的差距

### 1. Processor 高级功能
- 没有用户批量获取（Python 的 batch_fetch_and_update_users）
- 没有媒体下载（Python 的 _download_media_batch）

### 2. 测试覆盖
| 模块 | Rust | Python |
|------|------|--------|
| ingestion | 8 | 13 |
| control | 11 | 5+ |
| worker | 1 | N/A (集成) |
| processor | 19 | 10+ |
| **总计** | **68** | **30+** |
