# Rust vs Python 差距分析

## 关键差距

### 1. EventWriter 不完整
- `drain_once()` 跳过了内存队列的 drain
- 没有 `run()` 循环和优雅关闭
- 没有错误恢复（失败时应 spill 回磁盘）

### 2. Control 缺少 UUID 验证
- Python 验证 account_user_id 必须是 canonical UUID
- Rust 直接接受任何字符串

### 3. Worker 架构违规
- Rust 在 worker 中直接处理事件
- Python 正确解耦：worker 只入队，event_processor 负责处理

### 4. Processor 缺少关键功能
- 没有 LISTEN/NOTIFY（只有轮询）
- 没有重试和退避
- 没有事务性批处理
- 没有用户获取
- 没有媒体下载
- 没有房间成员追踪
- 零测试

### 5. 测试覆盖差距
| 模块 | Rust | Python |
|------|------|--------|
| ingestion | 6 | 13 |
| control | 4 | 5+ |
| worker | 1 | N/A (集成) |
| processor | 0 | 10+ |
