# Lilium NG - 深度架构分析

> 基于 8 个并行 subagent 对 dzmm_archive 代码库的深度分析

---

## 1. Spider 子系统架构

### 1.1 进程模型

多进程 + asyncio 架构：

```
1 arbiter 进程 (ws_arbiter.py) — 主进程监督器
  ├── N worker 进程 (ws_worker.py) — 每账户一个，独立子进程
  ├── 1 event_processor 进程 (event_processor.py) — 批量事件消费
  └── 1 connection_cleanup 守护进程 — 过期连接清理
```

### 1.2 组件清单

| 文件 | 角色 | 行数 | 并发模型 |
|------|------|------|----------|
| ws_arbiter.py | 主进程监督器 | 576 | asyncio + subprocess |
| ws_worker.py | 单账户 worker 入口 | 86 | asyncio |
| ws_runtime.py | Socket.IO 运行时 + 命令处理 | 582 | asyncio |
| ws_ingestion.py | 有界队列 + 磁盘溢出 + 批量写入 | 251 | asyncio.Queue |
| ws_control.py | Unix socket JSON 控制协议 | 293 | asyncio |
| event_processor.py | 批量事件消费 (NOTIFY + 轮询) | 761 | asyncio |
| connection_cleanup.py | 过期连接清理 | 184 | asyncio |
| explore_sync.py | 探索流同步 | 135 | asyncio |

### 1.3 关键数据结构

```python
# ws_control.py
ControlAction = Literal["status", "reconnect", "reload", "stop", "start", "restart", "rescan"]
ACCOUNT_ACTIONS = {"reconnect", "reload", "stop", "start", "restart"}
ARBITER_ACTIONS = {"status", "rescan"}

@dataclass(frozen=True, slots=True)
class ControlCommand:
    action: ControlAction
    account_user_id: str | None
    reason: str
    data: dict[str, Any] | None
    # methods: to_json(), from_json()

@dataclass(frozen=True, slots=True)
class ControlResponse:
    ok: bool
    message: str
    data: dict[str, Any] | None
    # methods: to_json(), from_json()

# ws_ingestion.py
@dataclass(slots=True)
class EventEnvelope:
    account_user_id: str
    event_type: str
    payload: dict[str, Any]
    received_at: datetime
    source: Literal["socket", "disk_replay"]

# ws_ingestion.py
@dataclass(slots=True)
class DiskSpillBuffer:
    path: Path
    # schema_version: 2
    # methods: append(), read_replay_batch(), discard_replay_batch(), has_pending()
```

### 1.4 WebSocketArbiter 核心逻辑

**构造参数**: socket_path, runtime_dir, start_worker, stop_worker, forward_to_worker, list_account_user_ids, worker_monitor_interval(30s), worker_control_timeout(3s), worker_shutdown_timeout(10s)

**关键状态**:
- worker_handles: dict[str, WorkerHandle] — 账户→子进程映射
- _shutdown_event: asyncio.Event
- _lifecycle_lock: asyncio.Lock — 序列化所有 worker 启停
- _manually_stopped_accounts: set[str] — 操作员手动停止的账户
- _restart_suppressed_accounts: set[str] — 锁冲突抑制的账户

**核心方法**:
```
scan_and_update_workers(retry_suppressed=False) -> ControlResponse:
  1. 从 DB 加载启用的账户列表
  2. 计算 desired = enabled - manually_stopped - restart_suppressed
  3. 停止已死亡的 worker
  4. 启动缺少的 worker
  5. 停止多余的 worker

_stop_account_worker_locked(account_user_id):
  1. 请求优雅关闭 (通过控制 socket)
  2. 等待 10s, SIGTERM
  3. 等待 10s, SIGTERM
  4. 等待 10s, SIGKILL

_default_start_worker(account_user_id) -> Process:
  asyncio.create_subprocess_exec(sys.executable, "-m", "spider.ws_worker", ...)
```

**run() 主循环**: 启动 3 个并发任务
1. shutdown_wait — 监听关闭信号
2. control_server — Unix socket 控制服务
3. monitor_worker_lifecycle — 每 30s 扫描

**信号处理**: SIGINT/SIGTERM → 关闭; SIGHUP/SIGUSR1 → 重新扫描

### 1.5 AccountWorker 运行时 (ws_runtime.py)

**构造**: 接收 account_user_id, buffer_path, queue_size(5000), auth(DZMMApi), runtime_dir

**内部组件**:
- spill: DiskSpillBuffer — 磁盘溢出
- ingestor: EventIngestor — 有界队列
- socket_runtime: SocketRuntime — Socket.IO 连接
- writer: EventWriter — 批量写入 DB

**run() 启动 5 个并发任务**:
1. writer_task — EventWriter.run()
2. socket_runtime — SocketRuntime.run()
3. outgoing_command_listener — NOTIFY + 30s 轮询
4. worker_control — Unix socket 控制服务
5. shutdown_wait — 关闭信号

**控制命令**: status, reconnect, stop

### 1.6 SocketRuntime (ws_runtime.py)

**核心职责**: 管理一个账户的 Socket.IO 连接生命周期

**关键状态**: sio(socketio.AsyncClient), lock_id(advisory lock), event_count

**run() 主循环**:
1. 获取 PostgreSQL advisory lock
2. 连接 Socket.IO
3. 心跳/重连循环:
   - 每 1-3s: 检查关闭/重连请求
   - 已连接: 发送心跳, 续租 advisory lock
   - 未连接: 指数退避 (2^n, 最大 30s), 热交换重连

**hot_swap_connection()**: 新连接建立后才关闭旧连接

### 1.7 EventIngestor + EventWriter (ws_ingestion.py)

```
EventIngestor:
  - asyncio.Queue(maxsize=5000)
  - accept_event(event) -> bool: put_nowait(), 满则 spill
  - stop_accepting(): 之后的事件走磁盘

EventWriter:
  - run(stop_event): 循环 drain_once()
  - drain_once() -> int:
    1. 优先回放磁盘 spill
    2. 从队列取 batch_size 个事件
    3. 批量 INSERT 到 websocket_events
    4. 失败时重新 spill 到磁盘
  - batch_size=100, batch_max_wait=0.25s, idle_sleep=0.05s
```

### 1.8 EventProcessor 事件处理 (event_processor.py)

**NOTIFY + 轮询 fallback**:
- LISTEN `websocket_event_inserted` channel
- 5s 轮询 fallback

**事件类型分发**:
```
message:new → Message.from_websocket() + create_message_if_missing()
             + 收集用户批量获取 + 检测房间成员变化
message:updated → 检查 recall vs 内容更新 + 保留编辑历史
message:deleted → mark_deleted()
message:recalled → mark_recalled()
presence:user-online → 收集用户批量获取
group:member-joined → upsert_room_member + 收集用户
group:member-left → mark_member_left

忽略: presence:user-offline, message:user-left, message:joined,
      message:online-status, match:limit, connected, disconnected
```

**重试机制**: 3 次指数退避 (1s→2s→4s, 最大 60s, 带抖动)
**降级**: 批量失败后逐事件处理 (隔离毒事件)
**媒体下载**: 批量提交后 spawn, Semaphore(10) 限并发

### 1.9 通信模式

| 路径 | 协议 | 方向 |
|------|------|------|
| Arbiter ↔ Workers | Unix socket + newline-delimited JSON | 双向 |
| Workers → DB Queue | asyncio.Queue + 批量 INSERT | 单向 |
| Event Processor ← DB | PostgreSQL LISTEN/NOTIFY + 5s 轮询 | 单向 |
| Workers ← 外部命令 | PostgreSQL NOTIFY + 30s 轮询 | 单向 |

### 1.10 韧性模式

1. **进程隔离**: 每账户独立进程
2. **锁冲突抑制**: exit code 75 → 自动抑制重启
3. **手动停止追踪**: 不会被自动重启
4. **热交换重连**: 新连接建立后才关闭旧连接
5. **磁盘溢出**: 队列满或 DB 写入失败时 spill
6. **批量重试 + 降级**: 批量失败→逐事件处理
7. **渐进式关闭**: 优雅请求→SIGTERM→SIGTERM→SIGKILL
8. **过期连接清理**: ConnectionCleanupDaemon

---

## 2. 服务层依赖关系

### 2.1 WorkTickService 依赖图

```
WorkTickService(AsyncBaseService)
├── core.pal_data (get_all_species_count, get_pal_by_code)
├── core.pal_work (calculate_efficiency, calculate_exp_gain_per_hour, ...)
├── core.land_bonus (calculate_resource_cache_cap, calculate_resource_income_per_hour)
├── core.work_efficiency_bonus (calculate_work_efficiency_multiplier)
├── core.pal_work_constants (TICK_INTERVAL_MINUTES, TICKS_PER_HOUR)
├── database.sql_identifiers (quote_model_identifier)
├── database.async_engine (get_async_session)
├── models.farm.land (Land, LandType, LandUpgradeStatus)
├── models.farm.land_assignment (AssignmentType, LandAssignment)
├── models.pal.pal_egg (EggStatus, PalEgg)
├── models.pal.pal (Pal)
├── models.farm.resource_production (ResourceProduction)
├── models.farm.turnip_seed (SeedStatus, TurnipSeed)
├── services.pal_assignment_locking (lock_active_assignments_then_pals, ...)
├── services.turnip_service (TurnipService) — get_inventory, consume
├── services.achievement_service (AchievementService) — increment_stat
├── services.user_notification_service (UserNotificationService) — create
├── services.farm_service (FarmService) — apply_farm_tick_growth
├── services.land_bonus_service (LandBonusService) — compute_farm_pool_bonus
├── services.land_service (LandService) — get_dormitory_speed_modifiers
└── utils (quantize_decimal, scheduler_trace)
```

### 2.2 关键方法签名

```python
class WorkTickService(AsyncBaseService):
    # Tick 处理
    process_tick(*, now: datetime | None = None) -> dict
    process_user_tick(user_id: str, *, now: datetime | None = None) -> dict
    calculate_due_ticks(last_work_at: datetime, now: datetime, max_ticks: int = 144) -> tuple[int, datetime]
    _process_loaded_assignments(assignments, pal_map, *, now) -> dict

    # 批量加载
    _load_active_assignments_with_pals() -> tuple[list[LandAssignment], dict[int, Pal]]
    _load_user_active_assignments_with_pals(user_id) -> tuple[list[LandAssignment], dict[int, Pal]]
    _load_active_assignment_user_counts() -> list[tuple[str, int]]
    _prefetch_resource_data(user_ids, types) -> tuple[dict, dict]
    _prefetch_upgrading_lands(user_ids) -> dict[str, list[Land]]
    _prefetch_users_with_growing_seeds(user_ids) -> set[str]
    _prefetch_users_with_active_eggs(user_ids) -> set[str]
    _prefetch_collection_profile(user_ids) -> dict[str, tuple[int, int]]
    _compute_user_efficiency_multipliers(assignments, pal_map) -> dict[str, float]

    # 效果计算
    _calculate_assignment_tick_effects(assignment, pal, turnips, tick_count, eff_mult) -> tuple[float, int, float]
    _calculate_pal_exp_state(pal, exp_amount) -> tuple[int, int]
    _turnips_per_tick(assignment, pal) -> float
    _calculate_consumption_pure(assignment, pal, tick_count) -> int

    # 批量更新
    _batch_update_assignment_tick_state(updates) -> None
    _batch_update_pal_exp_state(updates) -> None
    _apply_resource_production(user_id, type, efficiency_by_ticks) -> float
    _add_upgrade_progress(land, work_amount) -> None
    _apply_dormitory_time_acceleration(user_id, tick_interval_hours, ...) -> None

# TurnipService
consume(user_id: str, quantity: int, description: str) -> tuple[bool, int, list[int]]
get_inventory(user_id: str) -> list[TurnipInventory]
```

### 2.3 WalletService 核心方法

```python
# 双重记账 — 余额从不直接更新
get_balance(user_id, for_update=False) -> Decimal
get_ledger_balance(user_id) -> Decimal
get_ledger_position(user_id) -> WalletLedgerPosition
# WalletLedgerPosition: balance, escrow_balance, snapshot_tx_id

credit(user_id, amount, tx_type, description, ...) -> Decimal  # 2 条记录
debit(user_id, amount, ...) -> tuple[bool, Decimal]  # 0 或 2 条记录
transfer(from_id, to_id, amount, ...) -> tuple[bool, Decimal, Decimal]  # 2 条记录
freeze(user_id, amount, tx_type, description, ...) -> tuple[bool, Decimal, Decimal]  # 2 条记录
release(from_id, to_id, amount, ...) -> tuple[bool, Decimal, Decimal]  # 2 条记录
```

---

## 3. Models 层精确 Schema

### 3.1 Wallet (wallet)

| 字段 | Python 类型 | DB 类型 | 约束 |
|------|------------|---------|------|
| user_id | str | FK users.user_id | **PK** |
| allow_negative_balance | bool | BOOLEAN | NOT NULL, default False |
| snapshot_balance | Decimal | DECIMAL(38,2) | NOT NULL, default 0 |
| snapshot_escrow_balance | Decimal | DECIMAL(38,2) | NOT NULL, default 0 |
| snapshot_tx_id | int | BIGINT | NOT NULL, default 0 |
| last_daily_credit | date | DATE | nullable |
| total_credited | Decimal | DECIMAL(38,2) | NOT NULL, default 0 |
| created_at | datetime | TIMESTAMPTZ | NOT NULL, default utc_now |

### 3.2 WalletTransaction (wallet_transaction)

| 字段 | Python 类型 | DB 类型 | 约束 |
|------|------------|---------|------|
| id | int | BIGSERIAL | **PK** |
| user_id | str | FK users.user_id | INDEX, NOT NULL |
| amount | Decimal | DECIMAL(38,2) | NOT NULL |
| escrow_delta | Decimal | DECIMAL(38,2) | NOT NULL, default 0 |
| balance_after | Decimal | DECIMAL(38,2) | nullable |
| tx_type | str | VARCHAR(50) | NOT NULL |
| description | str | VARCHAR(200) | NOT NULL |
| reference_id | str | VARCHAR(100) | nullable |
| memo | str | VARCHAR(200) | nullable |
| counterparty_id | str | VARCHAR(100) | INDEX, NOT NULL |
| tx_group_id | str | VARCHAR(100) | INDEX, NOT NULL |
| principal_id | UUID | FK issued_principal | INDEX, nullable |
| metadata_json | dict | JSONB | nullable |
| escrow_after | Decimal | DECIMAL(38,2) | nullable |
| created_at | datetime | TIMESTAMPTZ | NOT NULL |

**索引**: 5 个覆盖索引用于 snapshot tail 查询

### 3.3 WebSocketEvent (websocket_events) — 分区表

| 字段 | Python 类型 | DB 类型 | 约束 |
|------|------------|---------|------|
| id | int | BIGINT GENERATED ALWAYS AS IDENTITY | **PK (复合)** |
| timestamp | datetime | TIMESTAMPTZ | **PK (复合)** |
| user_id | str | native | NOT NULL |
| event | str | native | NOT NULL |
| data | dict | JSONB | NOT NULL |

**索引**: 3 个复合索引

### 3.4 EventProcessorOffset (event_processor_offsets)

| 字段 | Python 类型 | DB 类型 | 约束 |
|------|------------|---------|------|
| processor_id | str | native | **PK** |
| last_processed_id | int | native | NOT NULL, default 0 |
| last_processed_timestamp | datetime | TIMESTAMPTZ | nullable |
| last_processed_at | datetime | TIMESTAMPTZ | nullable |
| updated_at | datetime | TIMESTAMPTZ | NOT NULL |

### 3.5 Message (messages) — 分区表

| 字段 | Python 类型 | DB 类型 | 约束 |
|------|------------|---------|------|
| message_id | str | native | **PK (复合)** |
| sent_at | datetime | TIMESTAMPTZ | **PK (复合)** |
| room_id | str | native | INDEX |
| sent_by | str | native | INDEX |
| content_type | str | native | INDEX |
| content_text | str | native | nullable |
| is_deleted | bool | BOOLEAN | INDEX, default False |
| is_recalled | bool | BOOLEAN | INDEX, default False |
| is_edited | bool | BOOLEAN | INDEX, default False |
| history | list[dict] | JSONB | nullable |
| raw_data | dict | JSONB | NOT NULL |
| source | str | native | INDEX |

**Factory methods**: `from_api()`, `from_websocket()`
**Helpers**: `mark_deleted()`, `mark_recalled()`, `add_to_history()`

### 3.6 User (users)

| 字段 | 类型 | 约束 |
|------|------|------|
| user_id | str | **PK** |
| full_name | str | INDEX, nullable |
| avatar_url | str | nullable |
| message_count | int | NOT NULL, default 0 |
| last_seen | datetime | nullable |

### 3.7 Room (rooms)

| 字段 | 类型 | 约束 |
|------|------|------|
| room_id | str | **PK** |
| title | str | required |
| account_ids | ARRAY(Text) | NOT NULL |
| message_count | int | INDEX, NOT NULL |

---

## 4. Wallet 服务双重记账

### 4.1 核心设计

**余额从不直接存储或更新。** 余额始终从账本派生：

```
balance = snapshot_balance + SUM(amount WHERE id > snapshot_tx_id)
escrow  = snapshot_escrow_balance + SUM(escrow_delta WHERE id > snapshot_tx_id)
```

### 4.2 操作矩阵

| 操作 | 记录数 | 对手方 | 余额检查 | Flush | 返回 |
|------|--------|--------|----------|-------|------|
| credit() | 2 (用户+银行) | 银行/系统 | 无 | Yes | Decimal |
| debit() | 0 或 2 | 银行/系统 | balance < amount | Yes | (bool, Decimal) |
| transfer() | 2 (from+to) | 互相 | allow_negative_from | Yes | (bool, Decimal, Decimal) |
| freeze() | 2 (同一用户) | 自己 | balance < amount | **No** | (bool, Decimal, Decimal) |
| release() | 2 (from+to) | 互相 | escrow < amount | Yes | (bool, Decimal, Decimal) |

### 4.3 双重记账模式

```python
# credit: 用户+100, 银行-100
WalletTransaction(user_id="user1", amount=+100, counterparty_id="bank", tx_group_id="abc")
WalletTransaction(user_id="bank", amount=-100, counterparty_id="user1", tx_group_id="abc")

# transfer: user1-50, user2+50
WalletTransaction(user_id="user1", amount=-50, counterparty_id="user2", tx_group_id="def")
WalletTransaction(user_id="user2", amount=+50, counterparty_id="user1", tx_group_id="def")

# freeze: user1 可用-200, user1 托管+200
WalletTransaction(user_id="user1", amount=-200, counterparty_id="user1", tx_group_id="ghi")
WalletTransaction(user_id="user1", amount=0, escrow_delta=+200, counterparty_id="user1", tx_group_id="ghi")
```

---

## 5. Core 层纯函数

### 5.1 pal_work.py — 帕鲁工作计算

| 函数 | 签名 | 公式 |
|------|------|------|
| calculate_efficiency | `(level, suitability, matched, rarity=1) -> float` | `EFF_BASE * (1+LEVEL_FACTOR*level) * (1+SUIT_RARITY_STEP*rarity)^suitability * role_match` |
| calculate_exp_needed | `(level) -> int` [LRU] | `PAL_EXP_BASE * level^PAL_EXP_POWER` |
| calculate_exp_gain_per_hour | `(efficiency) -> float` | `efficiency * PAL_EXP_RATE` |
| calculate_turnip_consumption_per_hour | `(food, level, role) -> float` | `food * (1+CONSUMPTION_LEVEL_FACTOR*level) * ROLE_COST[role]` |
| get_work_score | `(suitabilities, role) -> int` | sum of matching suitability values |
| is_role_matched | `(suitabilities, role) -> bool` | any matching suitability > 0 |

### 5.2 pal_work_constants.py — 常量值

| 常量 | 值 | 用途 |
|------|-----|------|
| EFF_BASE | 40 | 基础效率 |
| EFF_LEVEL_FACTOR | 0.03 | 等级乘数 |
| EFF_SUIT_RARITY_STEP | 0.05 | 稀有度指数步进 |
| ROLE_MATCH_BONUS | 1.0 | 角色匹配乘数 |
| ROLE_MISMATCH_PENALTY | 0.6 | 角色不匹配乘数 |
| PAL_MAX_LEVEL | 120 | 最大等级 |
| PAL_EXP_BASE | 12 | 经验基础 |
| PAL_EXP_POWER | 1.4 | 经验指数 |
| PAL_EXP_RATE | 0.5 | 经验速率 |
| TICK_INTERVAL_MINUTES | 10 | Tick 间隔 |
| TICKS_PER_HOUR | 6 | 每小时 tick 数 |
| CREDIT_RATE | 100 | 收益率 |
| UPGRADE_WORK_BASE | 8000 | 升级基础工作量 |
| UPGRADE_WORK_MULTIPLIER | 2 | 升级倍率 |
| WORK_MULTIPLIER_CAP | 1.55 | 效率乘数上限 |
| WORK_ACTIVE_SPECIES_TARGET | 12 | 活跃物种目标 |
| WORK_TOTAL_ELEMENTS | 9 | 元素总数 |
| ROLE_COST | {farm:1.0, warehouse:1.1, upgrade:1.2, mine:1.3, lumber_mill:1.3, workshop:1.1, dormitory:1.1} | 各角色消耗系数 |
| ROLE_SUITABILITIES | {farm:["浇水","播种","采集","牧场","制药"], warehouse:["搬运","手工作业","采矿","伐木","冷却"], upgrade:["手工作业","采矿","伐木","发电","生火"], mine:["采矿"], lumber_mill:["伐木"], workshop:["手工作业"], dormitory:["手工作业","搬运","制药","牧场"]} | 角色→适性映射 |

### 5.3 work_efficiency_bonus.py — 收集多样性加成

| 函数 | 签名 | 描述 |
|------|------|------|
| calculate_work_efficiency_multiplier | `(discovered_species, total_species, active_unique_species, active_elements, paired_species, active_male_count, active_female_count) -> float` | 5 个子乘数相乘, clamp to [1.0, 1.55] |
| calculate_gender_balance | `(male_count, female_count) -> float` | 1.0 - abs(m-f)/total, clamp to [0,1] |

### 5.4 land_bonus.py — 土地加成

| 函数 | 描述 |
|------|------|
| calculate_farm_time_bonus(eff) | `cap * eff / (eff + scale)`, scale=500, cap=1.0 |
| calculate_farm_capacity_bonus(eff) | scale=1000, cap=2.5 |
| calculate_farm_harvest_bonus(eff) | scale=1200, cap=1.5 |
| calculate_warehouse_capacity_bonus(eff) | 线性: eff / 333 |
| calculate_credit_income_per_hour(eff) | eff * 100 |
| calculate_resource_level_multiplier(level) | S-curve: 1.0 + 4.0 * l^2 / (l^2 + 6^2) |
| calculate_resource_cache_hours(level) | 24.0 * s_curve(level, cap=6, scale=10, power=2) |
| calculate_resource_worker_multiplier(eff) | capped_bonus(eff, scale=1200, cap=3.0) |
| calculate_resource_income_per_hour(eff, level) | 200 * 1200 * level_mult * worker_mult |
| calculate_resource_cache_cap(eff, level) | income_per_hour * cache_hours |
| calculate_dormitory_hatch_speed_bonus(eff) | log2(1 + eff/500) |
| calculate_dormitory_breeding_speed_bonus(eff) | log2(1 + eff/500) |

### 5.5 turnip_fv.py — 公允价值

```python
class FairValue:
    def __init__(self, initial_price: Decimal, alpha: Decimal = EMA_ALPHA)
    def value(self) -> Decimal
    def shift_by_pct(self, pct: Decimal | float) -> Decimal
    def update(self, trade_price: Decimal) -> Decimal  # EMA + tanh attenuation
```

### 5.6 turnip_npc.py — NPC 引擎

```python
# 策略类型
class NPCStrategy(ABC): ...
class RandomTrader(NPCStrategy): ...
class MomentumTrader(NPCStrategy): ...
class MeanReversionTrader(NPCStrategy): ...
class WhaleTrader(NPCStrategy): ...
class InformedTrader(NPCStrategy): ...
class PanicLiquidator(NPCStrategy): ...
class EuphoriaLifter(NPCStrategy): ...

# 数据结构
@dataclass(frozen=True, slots=True)
class MarketContext:
    fv: Decimal
    last_trade_price: Decimal | None
    fv_history: tuple[Decimal, ...]
    remaining_turnip_bucket: int
    remaining_cash_bucket: Decimal
    treasury_imbalance: Decimal | None
    direction_bias: float
    activity_multiplier: float
    spread_tolerance: float
    random_trader_boost: float
    best_bid: Decimal | None
    best_ask: Decimal | None
    bid_depth: tuple[tuple[Decimal, int], ...]
    ask_depth: tuple[tuple[Decimal, int], ...]
    panic_severity: float
    euphoria_severity: float

# NPC 引擎
class NPCEngine:
    def __init__(self, strategies: Sequence[BaseNPCConfig])
    def build_buckets(self, *, fv: Decimal, reference_notional: Decimal) -> dict
    def run_sub_tick(self, ctx: MarketContext, rng: Random) -> list[tuple[str, Decimal, int, OrderSide]]
```

### 5.7 turnip_memory_book.py — 内存订单簿

```python
class OrderSide(StrEnum):
    BUY = "buy"
    SELL = "sell"

@dataclass(slots=True)
class MemoryOrder:
    order_id: int
    side: OrderSide
    price: Decimal
    quantity: int
    owner: str
    sequence: int

class MemoryOrderBook:
    def add_order(side, price, quantity, owner) -> MemoryOrder
    def cancel_order(order_id) -> MemoryOrder | None
    def match() -> list[MatchFill]  # price-time matching
    def get_bids() -> list[MemoryOrder]
    def get_asks() -> list[MemoryOrder]
    def get_best_bid() -> Decimal | None
    def get_best_ask() -> Decimal | None
    def get_depth(levels=10) -> tuple[list, list]
    def clear() -> None
```

### 5.8 turnip_central_bank.py — 央行分配器

```python
class CentralBankAllocator:
    def calculate_turnip_allocation(farm_output, rng) -> int
    def calculate_cash_allocation(*, turnip_allocation, fv, multiplier, rng) -> Decimal
    def allocate(*, farm_output, harvest_ema, ambient_supply, fv, multiplier, rng) -> tuple[int, Decimal]
    def apply_safety_valve(amount, last_amount) -> int | Decimal
    def update_drift(rng) -> None  # OU process
```

### 5.9 turnip_market_config.py — 市场配置

```python
SELL_RATE = Decimal("0.05")
BUY_RATE = Decimal("0.05")
EMA_ALPHA = Decimal("0.10")
TURNIP_ALLOCATION_RATIO = Decimal("0.25")
TURNIP_ALLOCATION_FLOOR = 1_000
SUB_TICKS_PER_TICK = 12
SOURCE_DEPTH_LEVELS = 12
SINK_DEPTH_LEVELS = 12
DRIFT_STEP_SIGMA = 0.003
DRIFT_DECAY = 0.998
DRIFT_MAX = 0.15

NPC_STRATEGIES = (random, momentum, mean_reversion, whale_bull, whale_bear, panic_liquidator, euphoria_lifter)
```

---

## 6. 测试边界

### 6.1 测试结构

```
tests/
├── conftest.py                    # 根: DB 安全, async engine, fixtures
├── unit/
│   ├── conftest.py                # Bot user ID mocking
│   ├── services/                  # 服务层测试 (最大)
│   ├── core/                      # 纯逻辑测试
│   ├── spider/                    # Spider 组件测试
│   └── ...
└── integration/                   # 集成测试 (需要运行服务器)
```

### 6.2 测试模式

**Core 层测试** — 无 DB，纯函数直接测试
```python
def test_calculate_efficiency():
    eff = calculate_efficiency(level=10, suitability=3, matched=True, rarity=5)
    assert eff > 0
```

**Service 层测试** — 需要 DB session
```python
@pytest.mark.asyncio
async def test_credit(async_sqlmodel_session):
    svc = WalletService(async_sqlmodel_session)
    await svc.get_or_create_wallet("user1")
    await svc.credit("user1", Decimal("100"), "test", "test credit")
    balance = await svc.get_balance("user1")
    assert balance == Decimal("100")
```

**Spider 层测试** — 模拟组件
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
        start_worker=start_worker, stop_worker=stop_worker,
    )
    response = await arbiter.handle_command(
        ControlCommand(action="reload", account_user_id="user_a"))
    assert response.ok is True
    assert calls == [("stop", "user_a"), ("start", "user_a")]
```

### 6.3 测试边界

| 层 | 测试方式 | DB 需求 | Mock 策略 |
|----|----------|---------|-----------|
| Core | 纯单元测试 | 无 | 无 |
| Service | async DB 测试 | 有 | fixture 提供 session |
| Spider | 组件模拟测试 | 可选 | 模拟 start/stop worker |

### 6.4 Rust 测试需求

| 模块 | Python 测试参考 | Rust 测试类型 |
|------|----------------|---------------|
| lilium-common | 无 | 工具函数测试 |
| lilium-models | 无 | 结构体序列化测试 |
| lilium-core | tests/unit/core/ | 纯函数单元测试 |
| lilium-database | tests/unit/services/ | sqlx::test 集成测试 |
| lilium-api-client | tests/unit/spider/ | mock HTTP/WebSocket |
| lilium-spider | tests/unit/spider/ | 组件模拟测试 |

---

## 7. Rust 架构设计

### 7.1 Crate 依赖关系

```
lilium-common (错误类型, 常量, 工具函数)
    ↓
lilium-models (数据模型, 枚举, schema)
    ↓
lilium-database (连接池, 查询, 通知)
    ↓
lilium-api-client (HTTP/WebSocket 客户端)
    ↓
lilium-core (纯业务逻辑 — 与 Python core/ 1:1 对应)
    ↓
lilium-services (服务编排 — 双重记账, tick 引擎)
    ↓
binaries/lilium-spider (WebSocket 摄入服务)
```

### 7.2 模块映射

| Python 模块 | Rust crate | 纯函数? |
|-------------|-----------|---------|
| core/pal_work.py | lilium-core::pal_work | Yes |
| core/pal_work_constants.py | lilium-core::pal_work_constants | Yes |
| core/work_efficiency_bonus.py | lilium-core::work_efficiency | Yes |
| core/land_bonus.py | lilium-core::land_bonus | Yes |
| core/turnip_fv.py | lilium-core::turnip::fv | Yes |
| core/turnip_npc.py | lilium-core::turnip::npc | Yes |
| core/turnip_memory_book.py | lilium-core::turnip::memory_book | Yes |
| core/turnip_central_bank.py | lilium-core::turnip::central_bank | Yes |
| core/turnip_market_config.py | lilium-core::turnip::config | Yes |
| spider/ws_arbiter.py | lilium-spider::arbiter | No (async+subprocess) |
| spider/ws_worker.py | lilium-spider::worker | No (async+DB) |
| spider/ws_runtime.py | lilium-spider::runtime | No (async+WebSocket) |
| spider/ws_ingestion.py | lilium-spider::ingestion | No (async+tokio::fs) |
| spider/ws_control.py | lilium-spider::control | No (async+Unix socket) |
| spider/event_processor.py | lilium-spider::processor | No (async+DB) |
| services/wallet_service.py | lilium-services::wallet | No (async+DB) |
| services/work_tick_service.py | lilium-services::tick::work | No (async+DB) |
| models/wallet/*.py | lilium-models::wallet | N/A |
| models/dzmm/*.py | lilium-models::dzmm | N/A |
| models/ingestion/*.py | lilium-models::ingestion | N/A |

### 7.3 实现顺序

```
Phase 1: lilium-common + lilium-models + lilium-database
  ↓ 基础层, 所有其他 crate 依赖
Phase 2: lilium-core (与 Python core/ 1:1 对应)
  ↓ 纯函数, 可独立测试
Phase 3: lilium-services::wallet (双重记账)
  ↓ 钱包是所有服务的基础
Phase 4: lilium-services (其他服务)
  ↓ 依赖 core + models + database
Phase 5: lilium-spider (WebSocket 摄入)
  ↓ 最独立的二进制, 第一个迁移目标
Phase 6: 测试验证
  ↓ 对齐 Python 测试覆盖
```
