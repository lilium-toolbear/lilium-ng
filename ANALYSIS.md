# Lilium NG - 现有系统分析文档

## 1. 架构概览

### 1.1 六层架构

```
Presentation (cli/, archive_ui/, toolbear_ui/, bots/, telegram_bot.py)
    → Service Layer (services/, 126 modules)
        → Core Layer (core/) — 纯业务逻辑，无 async/DB
            → Domain Layer (models/) — SQLModel tables
                → Data Access (database/) — async session management
                    → API Client (dzmm_client/) — external API calls
```

### 1.2 关键设计原则

1. **Never skip layers** — 不允许跨层调用
2. **Result types over exceptions** — 业务失败返回 `@dataclass` result with `success: bool`
3. **No raw SQL in presentation code** — 所有 DB 操作通过 services
4. **Never directly UPDATE wallet.balance** — 所有余额变更通过 WalletService
5. **Transaction audit trail** — 每个余额变更必须创建 `*Transaction` 记录

---

## 2. 钱包系统 (Critical)

### 2.1 双重记账设计

钱包使用**无锁双重记账**系统，不是直接 debit/credit：

```
Wallet 表 (元数据 + 快照)
├── user_id (PK)
├── snapshot_balance: Decimal      -- 已压缩的账本余额
├── snapshot_escrow_balance: Decimal -- 已压缩的托管余额
├── snapshot_tx_id: int            -- 快照包含的最高 transaction id
├── allow_negative_balance: bool
├── last_daily_credit: date
├── total_credited: Decimal
└── created_at: datetime

WalletTransaction 表 (审计日志)
├── id (PK, auto-increment)
├── user_id
├── amount: Decimal               -- +credit, -debit
├── escrow_delta: Decimal         -- +freeze, -release
├── balance_after: Decimal (nullable)
├── tx_type: str
├── description: str
├── counterparty_id: str          -- 双重记账对手方
├── tx_group_id: str              -- 逻辑分组
├── reference_id: str (nullable)
├── memo: str (nullable)
├── metadata_json: JSONB (nullable)
├── escrow_after: Decimal (nullable)
├── principal_id: UUID (nullable)
└── created_at: datetime
```

### 2.2 余额计算

```python
# 余额 = 快照余额 + 尾部交易总和
balance = snapshot_balance + sum(transactions WHERE id > snapshot_tx_id)
```

### 2.3 交易类型 (TransactionType)

```python
class TransactionType(StrEnum):
    # 每日签到
    DAILY_CREDIT = "daily_credit"
    
    # 21点
    BLACKJACK_BET = "blackjack_bet"
    BLACKJACK_WIN = "blackjack_win"
    # ... 40+ types
    
    # 股票
    STOCK_BUY = "stock_buy"
    STOCK_SELL = "stock_sell"
    
    # 大头菜
    TURNIP_BUY = "turnip_buy"
    TURNIP_SELL = "turnip_sell"
    
    # 期货
    FUTURES_MARGIN = "futures_margin"
    FUTURES_PNL = "futures_pnl"
    
    # 转账
    TRANSFER_OUT = "transfer_out"
    TRANSFER_IN = "transfer_in"
```

### 2.4 核心操作

```python
# Credit (创建交易记录，不直接更新余额)
await wallet_svc.credit(user_id, amount, tx_type, description)

# Debit (检查余额后创建交易记录)
success, balance = await wallet_svc.debit(user_id, amount)

# Escrow (冻结/释放)
await wallet_svc.freeze(user_id, amount, tx_type, description)
await wallet_svc.release(user_id, amount, tx_type, description)

# Transfer (双向记账)
await wallet_svc.transfer(from_id, to_id, amount, tx_type, description)
```

### 2.5 快照压缩

钱包系统定期压缩旧交易到快照中：
- `materialize_wallet_snapshot_window()` — 将交易窗口压缩到快照
- `compact_wallet_snapshots()` — 前进快照指针
- 使用 PostgreSQL advisory locks 防止并发压缩

### 2.6 系统钱包

```python
# 系统钱包 ID
BANK_USER_ID = get_bank_user_id()
FUTURES_MM_TREASURY_USER_ID = "__futures_mm_treasury__"
TURNIP_SPOT_TREASURY_USER_ID = "__turnip_spot_treasury__"
# ... 等
```

---

## 3. Spider 子系统

### 3.1 架构

```
WebSocket (DZMM.ai)
  → ws_arbiter.py (主进程)
    → ws_worker.py (每账户一个进程)
      → ws_ingestion.py (有界队列)
        → websocket_events 表
          → event_processor.py (批量消费)
            → messages / users / rooms 表
```

### 3.2 关键组件

**ws_arbiter.py** — 进程监督器
- 管理每账户 worker 进程
- 处理 reload/restart/status 命令
- 使用 Unix socket 控制协议

**ws_worker.py** — 单账户 worker
- 连接 DZMM WebSocket
- 接收事件并入队
- 使用 PostgreSQL advisory locks

**ws_ingestion.py** — 事件队列
- 有界内存队列 + 磁盘溢出
- `EventIngestor` — 接收事件
- `EventWriter` — 批量写入 DB
- `DiskSpillBuffer` — JSONL 磁盘缓冲

**event_processor.py** — 事件处理器
- NOTIFY + 轮询 fallback
- 批量处理 (100 events/batch)
- 事件类型分发:
  - `message:new` → 创建消息
  - `message:updated` → 更新消息
  - `message:deleted/recalled` → 标记删除/撤回
  - `presence:user-online` → 获取用户信息

### 3.3 控制协议

```python
@dataclass
class ControlCommand:
    action: str  # status, reconnect, reload, restart, start, stop
    account_user_id: Optional[str]
    reason: Optional[str]

@dataclass
class ControlResponse:
    ok: bool
    message: str
    data: Optional[dict]
```

---

## 4. 核心业务逻辑

### 4.1 帕鲁工作 (Pal Work)

```python
# 效率计算
efficiency = EFF_BASE * (1 + EFF_LEVEL_FACTOR * level) * (1 + EFF_SUIT_RARITY_STEP * rarity) ** suitability * role_match

# 经验值
exp_needed = PAL_EXP_BASE * level^PAL_EXP_POWER

# 工作效率乘数
multiplier = m_collect * m_rich * m_element * m_gender_collect * m_gender_active
```

### 4.2 大头菜市场 (Turnip Market)

- **FairValue** — EMA 公允价值
- **NPCEngine** — NPC 做市商
- **MemoryOrderBook** — 内存订单簿
- **CentralBankAllocator** — 央行分配器

### 4.3 期货市场 (Futures Market)

- **FuturesMarketMaker** — 做市商
- **FuturesSettlementEngine** — 结算引擎
- **FuturesTickEngine** — Tick 引擎

### 4.4 探索系统 (Raid)

- **RaidEngine** — 状态机引擎 (~9K lines)
- **RngSnapshot** — 可序列化 RNG 状态
- **MapStorage** — 地图存储协议
- 数据驱动: YAML 配置

---

## 5. 数据库层

### 5.1 连接管理

```python
# 连接池配置
pool_size = 5
max_overflow = 10
pool_recycle = 1800  # 30 minutes
pool_pre_ping = True

# 时区
connect_args = {"server_setting": {"timezone": "utc"}}
```

### 5.2 会话管理

```python
async with get_async_session() as session:
    svc = SomeService(session)
    await svc.do_something()
    # Auto-commits on success, rolls back on exception
```

### 5.3 迁移

- 使用 Alembic
- ~158 个迁移文件
- 分区表: `websocket_events`, `messages`, `raid_action_log`

---

## 6. 测试边界

### 6.1 测试结构

```
tests/
├── unit/
│   ├── services/       # 服务层测试 (最大)
│   ├── core/           # 核心逻辑测试
│   ├── spider/         # Spider 组件测试
│   └── ...
└── integration/        # 集成测试 (需要运行服务器)
```

### 6.2 测试模式

**单元测试 (无 DB)**
```python
def test_calculate_efficiency():
    eff = calculate_efficiency(level=10, suitability=3, matched=True, rarity=5)
    assert eff > 0
```

**服务测试 (有 DB)**
```python
@pytest.mark.asyncio
async def test_credit(async_sqlmodel_session):
    svc = WalletService(async_sqlmodel_session)
    await svc.get_or_create_wallet("user1")
    await svc.credit("user1", Decimal("100"), "test", "test credit")
    balance = await svc.get_balance("user1")
    assert balance == Decimal("100")
```

**Spider 测试 (模拟组件)**
```python
@pytest.mark.asyncio
async def test_reload_stops_then_starts_one_account(tmp_path):
    calls = []
    
    async def start_worker(account_user_id):
        calls.append(("start", account_user_id))
        return object()
    
    async def stop_worker(account_user_id):
        calls.append(("stop", account_user_id))
    
    arbiter = WebSocketArbiter(
        socket_path=tmp_path / "ws_arbiter.sock",
        start_worker=start_worker,
        stop_worker=stop_worker,
    )
    
    response = await arbiter.handle_command(
        ControlCommand(action="reload", account_user_id="user_a")
    )
    
    assert response.ok is True
    assert calls == [("stop", "user_a"), ("start", "user_a")]
```

### 6.3 测试边界

1. **Core 层** — 纯函数，无 I/O，直接测试
2. **Service 层** — 需要 DB session，使用 fixture
3. **Spider 层** — 模拟组件，测试交互逻辑
4. **Wallet** — 重点测试双重记账、快照压缩、并发安全

### 6.4 关键测试场景

**钱包系统**
- 创建/获取钱包
- Credit/Debit 操作
- 余额计算 (snapshot + tail)
- 快照压缩
- 并发交易安全
- 每日签到

**Spider**
- 事件入队/出队
- 磁盘溢出
- 批量写入
- 控制命令处理
- Worker 生命周期

**核心逻辑**
- 效率计算
- 经验值计算
- 市场模拟
- 状态机 (raid)

---

## 7. Rust 实现注意事项

### 7.1 钱包系统

- 必须实现双重记账，不是简单 debit/credit
- 需要实现快照压缩机制
- 使用 PostgreSQL advisory locks 防止并发问题
- 交易类型需要完整的 enum 定义

### 7.2 Spider

- 使用 tokio 多线程，不是 Python asyncio
- WebSocket 连接管理需要更仔细
- 磁盘溢出使用 tokio::fs
- 控制协议使用 Unix socket

### 7.3 测试

- 使用 sqlx::test 宏进行 DB 测试
- 模拟组件进行单元测试
- 对齐 Python 测试的覆盖范围
