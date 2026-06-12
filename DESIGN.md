# DZMM Rust 重写设计文档

## [S1] 问题

当前 DZMM 系统使用 Python 实现，经过多次架构迭代积累了技术债：

- **内存开销大**：每账户独立 Python 进程，约 50MB/worker
- **GIL 限制**：asyncio 单线程，无法真正并行处理 CPU 密集型 batch 操作
- **类型安全缺失**：部分 API 使用 dict，运行时才发现错误
- **模块边界模糊**：126 个 service 文件，职责重叠
- **遗留兼容代码**：`ws_client.py` 委托给 `ws_arbiter` 等冗余抽象
- **Session 管理复杂**：async session 生命周期难以追踪

## [S2] 目标

用 Rust 重写整个系统，同时偿还技术债：

- 内存降低 10-20x（3-5MB/worker vs 50MB）
- tokio 多线程真正并行处理
- 类型安全贯穿全栈
- 模块边界清晰，无循环依赖
- 核心逻辑纯函数化
- 消除所有遗留兼容代码

## [S3] 设计原则

1. **每个逻辑层一个 crate** — 无循环依赖
2. **核心逻辑必须是纯函数** — 无 async/IO
3. **所有 DB 访问通过 dedicated crate** — database 层
4. **API client 必须 async + resilient** — 重试、超时、反检测
5. **一个概念只有一个生命周期所有者** — 不要重复抽象
6. **强类型优先** — 消除 dict-based API

## [S4] 当前架构

```
6-Layer Architecture (Python):
Presentation (cli/, archive_ui/, toolbear_ui/, bots/, telegram_bot.py)
    → Service Layer (services/, 126 files)
        → Core Layer (core/) — 纯业务逻辑
            → Domain Layer (models/) — SQLModel tables
                → Data Access (database/) — async session management
                    → API Client (dzmm_client/) — external API calls
```

### 技术债清单

| 问题 | Python 现状 | Rust 消除方式 |
|------|------------|--------------|
| AsyncBaseService 冗余 | 每个 service 继承基类 | trait + async fn |
| 126 个 service 文件 | 按功能细分 | 按领域合并成 ~15 个模块 |
| dict-based API | 部分返回 dict | 全部强类型结构体 |
| 遗留兼容代码 | ws_client.py → ws_arbiter | 直接删除 |
| 复杂迁移系统 | 158 个迁移文件 | sqlx migrations |
| 混合同步/异步 | 部分 sync 函数 | 纯 async |
| Session 管理 | 复杂生命周期 | 连接池直连 |
| 命名不一致 | Python 风格 | Rust 命名规范 |

## [S5] 目标架构

### 5.1 Crate 依赖关系

```
dzmm-common (工具、错误类型、常量)
    ↓
dzmm-models (数据模型、枚举、schema)
    ↓
dzmm-database (连接池、迁移、通知)
    ↓
dzmm-api-client (HTTP/WebSocket 客户端)
    ↓
dzmm-core (纯业务逻辑、计算公式)
    ↓
dzmm-services (服务编排、业务流程)
    ↓
binaries (spider, bot, web, cli)
```

### 5.2 目录结构

```
dzmm-rust/
├── Cargo.toml              # Workspace 配置
├── crates/
│   ├── dzmm-common/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs    # 统一错误类型
│   │       ├── constants.rs
│   │       └── utils.rs
│   ├── dzmm-models/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── message.rs
│   │       ├── user.rs
│   │       ├── room.rs
│   │       ├── wallet.rs
│   │       ├── pal.rs
│   │       ├── farm.rs
│   │       ├── turnip.rs
│   │       ├── futures.rs
│   │       ├── game.rs
│   │       └── ingestion.rs
│   ├── dzmm-database/
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pool.rs
│   │       ├── notifications.rs
│   │       └── queries/
│   ├── dzmm-api-client/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── auth.rs
│   │       ├── http.rs
│   │       ├── websocket.rs
│   │       ├── rate_limiter.rs
│   │       └── anti_detection.rs
│   ├── dzmm-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pal_work.rs
│   │       ├── work_efficiency.rs
│   │       ├── land.rs
│   │       ├── tax.rs
│   │       ├── turnip/
│   │       │   ├── mod.rs
│   │       │   ├── fv.rs
│   │       │   ├── npc.rs
│   │       │   ├── memory_book.rs
│   │       │   ├── central_bank.rs
│   │       │   └── market_config.rs
│   │       ├── futures/
│   │       │   ├── mod.rs
│   │       │   └── market_maker.rs
│   │       └── raid/
│   │           ├── mod.rs
│   │           ├── engine.rs
│   │           ├── types.rs
│   │           ├── mapgen.rs
│   │           └── dice.rs
│   └── dzmm-services/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── traits.rs    # 服务 trait 定义
│           ├── message.rs
│           ├── user.rs
│           ├── wallet.rs
│           ├── turnip.rs
│           ├── pal.rs
│           ├── farm.rs
│           ├── futures.rs
│           ├── game_escrow.rs
│           └── tick/
│               ├── mod.rs
│               ├── work_tick.rs
│               ├── turnip_tick.rs
│               └── futures_tick.rs
├── binaries/
│   ├── dzmm-spider/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── arbiter/
│   │       ├── worker/
│   │       ├── processor/
│   │       └── control/
│   ├── dzmm-bot/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── dzmm-web/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── api/
│   │       └── graphql/
│   └── dzmm-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── frontend/                # Vue.js 前端 (保持不变)
└── scripts/
    └── new_worktree.py
```

## [S6] 各 Crate 详细设计

### 6.1 dzmm-common

```rust
// 错误类型
pub enum DzmmError {
    Database(sqlx::Error),
    Api(reqwest::Error),
    Serialization(serde_json::Error),
    Io(std::io::Error),
    Config(String),
    Business(String),
}

impl std::fmt::Display for DzmmError { ... }
impl std::error::Error for DzmmError { ... }

// 常量
pub const TICK_INTERVAL_MINUTES: i64 = 10;
pub const BATCH_SIZE: usize = 100;
pub const MAX_CATCHUP_TICKS: i64 = 144;

// 工具函数
pub fn utc_now() -> DateTime<Utc> { ... }
pub fn quantize_decimal(d: Decimal, scale: Decimal) -> Decimal { ... }
```

### 6.2 dzmm-models

按领域组织，每个领域一个模块：

```rust
pub mod message {
    #[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
    pub struct Message {
        pub message_id: String,
        pub room_id: String,
        pub sent_by: Option<String>,
        pub content_text: Option<String>,
        pub content_type: Option<String>,
        pub sent_at: Option<DateTime<Utc>>,
        pub is_deleted: bool,
        pub is_recalled: bool,
        pub history: Option<serde_json::Value>,
    }
    
    impl Message {
        pub fn from_websocket(data: &serde_json::Value) -> Result<Self> { ... }
        pub fn from_api(data: &serde_json::Value) -> Result<Self> { ... }
        pub fn add_to_history(&mut self, old_content: String) { ... }
    }
}

pub mod user {
    #[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
    pub struct User {
        pub user_id: String,
        pub display_name: Option<String>,
        pub username: Option<String>,
        pub avatar_url: Option<String>,
    }
}

pub mod wallet {
    #[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
    pub struct Wallet {
        pub user_id: String,
        pub balance: Decimal,
    }
    
    #[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
    pub struct WalletTransaction {
        pub id: i64,
        pub user_id: String,
        pub amount: Decimal,
        pub balance_after: Decimal,
        pub tx_type: String,
        pub description: Option<String>,
        pub created_at: DateTime<Utc>,
    }
}

pub mod ingestion {
    #[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
    pub struct WebSocketEvent {
        pub id: Option<i64>,
        pub event: String,
        pub data: serde_json::Value,
        pub user_id: String,
        pub timestamp: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventEnvelope {
        pub event_type: String,
        pub data: serde_json::Value,
        pub user_id: String,
        pub timestamp: DateTime<Utc>,
    }
}
```

### 6.3 dzmm-database

```rust
pub struct DbPool {
    inner: sqlx::PgPool,
}

impl DbPool {
    pub async fn connect(url: &str, pool_size: u32) -> Result<Self> { ... }
    pub async fn run_migrations(&self) -> Result<()> { ... }
    pub fn inner(&self) -> &sqlx::PgPool { ... }
}

pub struct NotificationManager {
    // PostgreSQL LISTEN/NOTIFY
}

impl NotificationManager {
    pub async fn subscribe(&self, channel: &str) -> Result<mpsc::Receiver<Notification>> { ... }
    pub async fn unsubscribe(&self, sub_id: &str) -> Result<()> { ... }
}

// 查询模块
pub mod queries {
    pub mod events {
        pub async fn insert_events(pool: &PgPool, events: &[EventEnvelope]) -> Result<()> { ... }
        pub async fn get_events_after_offset(
            pool: &PgPool,
            timestamp: Option<NaiveDateTime>,
            id: i64,
            limit: i64,
        ) -> Result<Vec<WebSocketEvent>> { ... }
    }
    
    pub mod messages {
        pub async fn create_message(pool: &PgPool, message: &Message) -> Result<bool> { ... }
        pub async fn update_message(pool: &PgPool, message: &Message) -> Result<()> { ... }
        pub async fn mark_deleted(pool: &PgPool, message_id: &str, deleted_by: Option<&str>) -> Result<()> { ... }
    }
    
    pub mod wallet {
        pub async fn get_balance(pool: &PgPool, user_id: &str) -> Result<Decimal> { ... }
        pub async fn credit(pool: &PgPool, user_id: &str, amount: Decimal, tx_type: &str, desc: &str) -> Result<()> { ... }
        pub async fn debit(pool: &PgPool, user_id: &str, amount: Decimal) -> Result<(bool, Decimal)> { ... }
    }
}
```

### 6.4 dzmm-api-client

```rust
pub struct DzmmClient {
    http: reqwest::Client,
    cookies: CookieJar,
    rate_limiter: RateLimiter,
    config: ClientConfig,
}

impl DzmmClient {
    pub fn new(config: ClientConfig) -> Result<Self> { ... }
    
    // 认证
    pub async fn login(&self, email: &str, password: &str) -> Result<()> { ... }
    pub async fn refresh(&self) -> Result<()> { ... }
    pub async fn login_with_qr(&self) -> Result<()> { ... }
    
    // 用户/房间
    pub async fn get_user_info(&self, user_id: &str, room_id: &str) -> Result<UserInfo> { ... }
    pub async fn batch_get_user_info(&self, pairs: &[(String, String)]) -> Result<Vec<UserInfo>> { ... }
    pub async fn fetch_room_members(&self, room_id: &str) -> Result<Vec<RoomMember>> { ... }
    
    // 消息
    pub async fn fetch_room_messages(&self, room_id: &str, limit: usize) -> Result<Vec<Message>> { ... }
    pub async fn send_heartbeat(&self) -> Result<()> { ... }
    
    // 媒体
    pub async fn upload_chat_image(&self, room_id: &str, image: &[u8]) -> Result<String> { ... }
    pub async fn download_media(&self, url: &str) -> Result<Vec<u8>> { ... }
    
    // WebSocket
    pub async fn connect_websocket(&self, account_id: &str) -> Result<WsConnection> { ... }
}

pub struct WsConnection {
    stream: WebSocketStream<TcpStream>,
    account_id: String,
}

impl WsConnection {
    pub async fn next_event(&mut self) -> Result<Option<EventEnvelope>> { ... }
    pub async fn close(&mut self) -> Result<()> { ... }
}
```

### 6.5 dzmm-core

纯函数，无 async，无 DB：

```rust
pub mod pal_work {
    pub fn calculate_efficiency(level: i32, suitability: i32, matched: bool, rarity: i32) -> f64 { ... }
    pub fn calculate_exp_needed(level: i32) -> i64 { ... }
    pub fn calculate_turnip_consumption(level: i32, suitability: i32) -> f64 { ... }
    pub fn get_work_score(suitabilities: &HashMap<String, i32>, role: &str) -> i32 { ... }
}

pub mod work_efficiency {
    pub fn calculate_work_efficiency_multiplier(
        discovered_species: i32,
        total_species: i32,
        active_unique_species: i32,
        active_elements: i32,
        paired_species: i32,
        active_male_count: i32,
        active_female_count: i32,
    ) -> f64 { ... }
}

pub mod turnip {
    pub mod fv {
        pub struct FairValue { value: Decimal }
        impl FairValue {
            pub fn new(initial: Decimal) -> Self { ... }
            pub fn shift_by_pct(&mut self, pct: Decimal) { ... }
            pub fn value(&self) -> Decimal { ... }
        }
    }
    
    pub mod npc {
        pub struct NPCEngine {
            strategies: Vec<NPCStrategy>,
            buckets: HashMap<String, (i64, Decimal)>,
        }
        
        impl NPCEngine {
            pub fn new(strategies: Vec<NPCStrategy>) -> Self { ... }
            pub fn generate_orders(&self, ctx: &MarketContext) -> Vec<Order> { ... }
            pub fn build_buckets(&self, fv: Decimal, reference_notional: Decimal) -> HashMap<String, (i64, Decimal)> { ... }
        }
    }
    
    pub mod memory_book {
        pub struct MemoryOrderBook {
            bids: Vec<(Decimal, i64)>,
            asks: Vec<(Decimal, i64)>,
        }
        
        impl MemoryOrderBook {
            pub fn new() -> Self { ... }
            pub fn clear(&mut self) { ... }
            pub fn add_bid(&mut self, price: Decimal, qty: i64) { ... }
            pub fn add_ask(&mut self, price: Decimal, qty: i64) { ... }
        }
    }
}

pub mod land {
    pub fn calculate_resource_income_per_hour(land_type: &LandType, level: i32) -> f64 { ... }
    pub fn calculate_resource_cache_cap(land_type: &LandType, level: i32) -> i64 { ... }
}

pub mod raid {
    pub struct RaidEngine {
        map_storage: Box<dyn MapStorage>,
    }
    
    impl RaidEngine {
        pub fn new(map_storage: Box<dyn MapStorage>) -> Self { ... }
        pub fn process(&self, state: &RaidState, action: &Action) -> (RaidState, Vec<Effect>) { ... }
    }
}
```

### 6.6 dzmm-services

```rust
// 服务 trait 定义
pub mod traits {
    #[async_trait]
    pub trait MessageService: Send + Sync {
        async fn create_message_if_missing(&self, message: Message) -> Result<bool>;
        async fn update_message(&self, message: Message) -> Result<()>;
        async fn mark_deleted(&self, message_id: &str, deleted_by: Option<&str>) -> Result<()>;
        async fn mark_recalled(&self, message_id: &str) -> Result<()>;
    }
    
    #[async_trait]
    pub trait WalletService: Send + Sync {
        async fn credit(&self, user_id: &str, amount: Decimal, tx_type: &str, desc: &str) -> Result<()>;
        async fn debit(&self, user_id: &str, amount: Decimal) -> Result<(bool, Decimal)>;
        async fn transfer(&self, from: &str, to: &str, amount: Decimal, tx_type: &str, desc: &str) -> Result<()>;
    }
    
    #[async_trait]
    pub trait TurnipService: Send + Sync {
        async fn consume(&self, user_id: &str, quantity: i64, desc: &str) -> Result<(bool, i64, Vec<i64>)>;
        async fn get_inventory(&self, user_id: &str) -> Result<Vec<TurnipBatch>>;
    }
}

// 具体实现
pub mod message {
    pub struct PostgresMessageService {
        pool: PgPool,
    }
    
    impl PostgresMessageService {
        pub fn new(pool: PgPool) -> Self { ... }
    }
    
    #[async_trait]
    impl traits::MessageService for PostgresMessageService { ... }
}

pub mod wallet { ... }
pub mod turnip { ... }

// Tick 引擎
pub mod tick {
    pub struct WorkTickEngine {
        pool: PgPool,
    }
    
    impl WorkTickEngine {
        pub fn new(pool: PgPool) -> Self { ... }
        
        pub async fn process_tick(&self, now: DateTime<Utc>) -> Result<TickSummary> { ... }
        pub async fn process_user_tick(&self, user_id: &str, now: DateTime<Utc>) -> Result<UserTickSummary> { ... }
    }
    
    pub struct TurnipTickEngine { ... }
    pub struct FuturesTickEngine { ... }
}
```

## [S7] Binaries 设计

### 7.1 dzmm-spider

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let pool = DbPool::connect(&config.database.url, config.database.pool_size).await?;
    pool.run_migrations().await?;
    
    let arbiter = Arbiter::new(config.clone(), pool.clone());
    arbiter.run().await
}
```

### 7.2 dzmm-bot

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let pool = DbPool::connect(&config.database.url, config.database.pool_size).await?;
    let client = DzmmClient::new(config.api.clone())?;
    
    let bot = Bot::new(config, pool, client);
    bot.run().await
}
```

### 7.3 dzmm-web

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let pool = DbPool::connect(&config.database.url, config.database.pool_size).await?;
    
    let app = build_router(pool, config);
    let listener = TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await
}
```

## [S8] 配置管理

```toml
# config/common.toml (共享配置)
[database]
url = "postgresql://localhost/dzmm"
pool_size = 10

[logging]
level = "info"
format = "pretty"

# config/spider.toml (spider 专用)
[worker]
queue_size = 5000
batch_size = 100
buffer_dir = "data/event/buffer"

[processor]
polling_interval_secs = 5

[control]
socket_path = "runtime/spider/control.sock"
```

环境变量覆盖：
- `DATABASE_URL` - 数据库连接字符串
- `RUST_LOG` - 日志级别

## [S9] 迁移策略

### Phase 1: 基础层（3-5 天）
- dzmm-common
- dzmm-models
- dzmm-database

### Phase 2: 客户端层（2-3 天）
- dzmm-api-client

### Phase 3: 业务逻辑层（5-7 天）
- dzmm-core (pal_work, turnip, raid 等)

### Phase 4: 服务层（5-7 天）
- dzmm-services (消息、钱包、tick 引擎等)

### Phase 5: Spider 服务（5-7 天）
- dzmm-spider (第一个迁移目标)

### Phase 6: 其他服务（后续）
- dzmm-bot
- dzmm-web
- dzmm-cli

**总计：约 4-6 周**

## [S10] 验证方式

1. **单元测试**：每个 crate 独立测试
2. **集成测试**：对比 Rust 和 Python 处理结果
3. **性能测试**：内存、吞吐量、延迟对比
4. **压力测试**：高并发连接和事件风暴

## [S11] 技术栈（已确认）

| 用途 | 选择 | 理由 |
|------|------|------|
| 异步运行时 | **tokio** | 生态最成熟，社区最大 |
| 数据库驱动 | **sqlx** | 编译时 SQL 检查，类型安全 |
| WebSocket | **tungstenite** | 底层灵活，性能好 |
| HTTP 客户端 | **reqwest** | 功能全面，基于 hyper |
| Web 框架 | **axum** | 性能最好，生态最大 |
| 序列化 | **serde + serde_json** | Rust 标准 |
| 配置 | **toml + dotenvy** | 简洁实用 |
| 日志 | **tracing + tracing-subscriber** | 结构化日志 |
| CLI | **clap** | 功能全面 |
| 错误处理 | **anyhow + thiserror** | 应用级 + 库级 |
| 日期时间 | **chrono** | 成熟稳定 |
| 小数 | **rust_decimal** | 精确计算 |
| 异步 trait | **async-trait** | 编译器支持前的必要方案 |

## [S12] Cargo.toml (Workspace)

```toml
[workspace]
members = [
    "crates/dzmm-common",
    "crates/dzmm-models",
    "crates/dzmm-database",
    "crates/dzmm-api-client",
    "crates/dzmm-core",
    "crates/dzmm-services",
    "binaries/dzmm-spider",
    "binaries/dzmm-bot",
    "binaries/dzmm-web",
    "binaries/dzmm-cli",
]
resolver = "2"

[workspace.dependencies]
# 异步运行时
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"

# 数据库
sqlx = { version = "0.8", features = [
    "runtime-tokio",
    "tls-native-tls",
    "postgres",
    "chrono",
    "json",
    "migrate",
] }

# WebSocket
tungstenite = { version = "0.21", features = ["tokio-native-tls"] }

# HTTP
reqwest = { version = "0.12", features = ["json", "stream", "cookies"] }

# Web 框架
axum = { version = "0.8", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 配置
toml = "0.8"
dotenvy = "0.15"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# CLI
clap = { version = "4", features = ["derive"] }

# 错误处理
anyhow = "1"
thiserror = "2"

# 日期时间
chrono = { version = "0.4", features = ["serde"] }

# 小数
rust_decimal = { version = "1", features = ["serde-with-str"] }

# 异步 trait
async-trait = "0.1"

# 工具
futures = "0.3"
dashmap = "6"
```
