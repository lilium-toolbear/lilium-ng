# DZMM Rust 重写实现计划

## 技术栈

| 用途 | 选择 |
|------|------|
| 异步运行时 | tokio |
| 数据库 | sqlx (PostgreSQL) |
| WebSocket | tungstenite |
| HTTP | reqwest |
| Web | axum |
| 序列化 | serde + serde_json |
| 配置 | toml + dotenvy |
| 日志 | tracing |
| CLI | clap |
| 错误 | anyhow + thiserror |
| 小数 | rust_decimal |

---

## 阶段 1: 项目骨架 + 基础层

### T1.1 初始化 Workspace
```bash
mkdir dzmm-rust && cd dzmm-rust
cargo init --name dzmm-common crates/dzmm-common
cargo init --name dzmm-models crates/dzmm-models
cargo init --name dzmm-database crates/dzmm-database
```

### T1.2 实现 dzmm-common
- 错误类型 `DzmmError`
- 常量 `TICK_INTERVAL_MINUTES`, `BATCH_SIZE` 等
- 工具函数 `utc_now()`, `quantize_decimal()`

### T1.3 实现 dzmm-models
按领域组织：
- `message.rs` - Message, MessageContent
- `user.rs` - User, UserInfo
- `room.rs` - Room, RoomMember
- `wallet.rs` - Wallet, WalletTransaction
- `pal.rs` - Pal, PalEgg
- `farm.rs` - Land, LandAssignment, ResourceProduction
- `turnip.rs` - TurnipPrice, TurnipInventory, TurnipOrder
- `futures.rs` - FuturesOrder, FuturesPosition
- `ingestion.rs` - WebSocketEvent, EventEnvelope

### T1.4 实现 dzmm-database
- `pool.rs` - DbPool 连接池
- `migrations/` - SQL 迁移文件
- `notifications.rs` - LISTEN/NOTIFY
- `queries/` - 按领域组织的查询

**预估时间：3-5 天**

---

## 阶段 2: 客户端层

### T2.1 初始化 dzmm-api-client
```bash
cargo init --name dzmm-api-client crates/dzmm-api-client
```

### T2.2 实现 HTTP 客户端
- `auth.rs` - 认证 (login, refresh, QR code)
- `http.rs` - 统一请求方法
- `rate_limiter.rs` - 请求限流
- `anti_detection.rs` - 反检测 (UA, headers, delays)

### T2.3 实现 WebSocket 客户端
- `websocket.rs` - Socket.IO 握手
- `WsConnection` - 事件接收

### T2.4 实现 API 方法
- 用户/房间: get_user_info, batch_get_user_info, fetch_room_members
- 消息: fetch_room_messages, send_heartbeat
- 媒体: upload_chat_image, download_media

**预估时间：3-5 天**

---

## 阶段 3: 业务逻辑层

### T3.1 初始化 dzmm-core
```bash
cargo init --name dzmm-core crates/dzmm-core
```

### T3.2 实现 pal_work 模块
- `calculate_efficiency()`
- `calculate_exp_needed()`
- `calculate_turnip_consumption()`
- `get_work_score()`

### T3.3 实现 work_efficiency 模块
- `calculate_work_efficiency_multiplier()`
- `calculate_gender_balance()`

### T3.4 实现 turnip 模块
- `fv.rs` - FairValue
- `npc.rs` - NPCEngine
- `memory_book.rs` - MemoryOrderBook
- `central_bank.rs` - CentralBankAllocator
- `market_config.rs` - 市场配置常量

### T3.5 实现 land 模块
- `calculate_resource_income_per_hour()`
- `calculate_resource_cache_cap()`

### T3.6 实现 raid 模块
- `engine.rs` - RaidEngine 状态机
- `types.rs` - RaidState, Action, Effect
- `mapgen.rs` - WFC 地图生成
- `dice.rs` - D20 骰子

**预估时间：5-7 天**

---

## 阶段 4: 服务层

### T4.1 初始化 dzmm-services
```bash
cargo init --name dzmm-services crates/dzmm-services
```

### T4.2 定义服务 trait
- `traits.rs` - MessageService, WalletService, TurnipService 等

### T4.3 实现核心服务
- `message.rs` - PostgresMessageService
- `wallet.rs` - PostgresWalletService
- `turnip.rs` - PostgresTurnipService
- `user.rs` - PostgresUserService

### T4.4 实现 Tick 引擎
- `tick/work_tick.rs` - WorkTickEngine
- `tick/turnip_tick.rs` - TurnipTickEngine
- `tick/futures_tick.rs` - FuturesTickEngine

**预估时间：5-7 天**

---

## 阶段 5: Spider 服务（第一个迁移目标）

### T5.1 初始化 dzmm-spider
```bash
cargo init --name dzmm-spider binaries/dzmm-spider
```

### T5.2 实现 Arbiter
- `arbiter/mod.rs` - 主进程监督
- `arbiter/worker.rs` - Worker 管理
- `arbiter/signal.rs` - 信号处理

### T5.3 实现 Worker
- `worker/mod.rs` - 单账户 worker
- `worker/ws_client.rs` - WebSocket 客户端
- `worker/ingestion.rs` - 事件队列
- `worker/writer.rs` - 批量写入

### T5.4 实现 EventProcessor
- `processor/mod.rs` - 主处理循环
- `processor/batch.rs` - 批量处理
- `processor/media.rs` - 媒体下载

### T5.5 实现控制协议
- `control/socket.rs` - Unix socket server

**预估时间：5-7 天**

---

## 阶段 6: 集成测试

### T6.1 单元测试
- 每个 crate 的核心逻辑测试

### T6.2 集成测试
- 对比 Rust 和 Python 处理相同事件
- 验证 DB 兼容性

### T6.3 性能测试
- 内存使用对比
- 吞吐量对比
- 延迟对比

**预估时间：3-5 天**

---

## 阶段 7: 部署

### T7.1 编译优化
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### T7.2 Systemd 服务
```ini
[Unit]
Description=DZMM Spider Rust Service
After=network.target postgresql.service

[Service]
Type=simple
User=dzmm
ExecStart=/opt/dzmm-rust/dzmm-spider
Restart=always

[Install]
WantedBy=multi-user.target
```

### T7.3 监控
- Prometheus metrics
- 结构化日志

**预估时间：2-3 天**

---

## 后续阶段（Spider 验证通过后）

### 阶段 8: dzmm-bot
- 40+ 命令迁移
- ACL、限流

### 阶段 9: dzmm-web
- FastAPI → axum
- GraphQL (async-graphql)
- 静态文件服务

### 阶段 10: dzmm-cli
- wsctl 等命令行工具

---

## 总预估时间

| 阶段 | 时间 |
|------|------|
| 1: 基础层 | 3-5 天 |
| 2: 客户端层 | 3-5 天 |
| 3: 业务逻辑层 | 5-7 天 |
| 4: 服务层 | 5-7 天 |
| 5: Spider | 5-7 天 |
| 6: 测试 | 3-5 天 |
| 7: 部署 | 2-3 天 |
| **总计** | **4-6 周** |

---

## 里程碑

### M1: 基础可用 (第 1 周末)
- Workspace 骨架
- dzmm-common, dzmm-models, dzmm-database

### M2: 客户端可用 (第 2 周末)
- dzmm-api-client
- 能连接 DZMM API

### M3: Spider 可用 (第 3 周末)
- dzmm-spider 独立运行
- 能处理 WebSocket 事件

### M4: 生产就绪 (第 4 周末)
- 所有测试通过
- 性能达标
- 部署就绪

---

## 并行任务

以下任务可以并行：
- T1.2 (dzmm-common) 和 T1.3 (dzmm-models)
- T3.2-T3.6 (dzmm-core 各模块)
- T5.2-T5.5 (dzmm-spider 各组件)

---

## 验证检查点

每个阶段结束时：
1. `cargo build` 通过
2. `cargo test` 通过
3. `cargo clippy` 无警告
4. 代码审查

Spider 阶段额外验证：
1. 能连接 DZMM WebSocket
2. 能接收事件
3. 能写入 DB
4. Python 服务层能读取数据
