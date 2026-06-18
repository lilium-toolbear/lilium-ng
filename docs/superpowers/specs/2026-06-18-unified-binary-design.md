# 单一二进制合并设计

Date: 2026-06-18
Status: Approved

## 目标

把当前分散的三个二进制（`lilium-spider`、`lilium-event-processor`、`lilium-cli`）
合并成单一二进制 `lilium`，所有命令与服务通过同一个二进制的子命令调用。

## 背景

现状三个 binary crate：

- `binaries/lilium-spider` — 长运行 WebSocket arbiter（arbiter/worker/control/ingestion 模块）
- `binaries/lilium-event-processor` — 长运行数据库轮询器（processor 模块）
- `binaries/lilium-cli` — clap 命令分发（send-command / sync-members / sync-rooms / explore）

三者各自有近重复的 `config.rs`（共享 `DatabaseConfig`/`NotificationConfig`，加上各自特有的
`worker`/`processor`/`data_path`），并各自带 `main.rs` 做 dotenv + sentry + tokio runtime bootstrap。

## 设计

### 命名

crate、目录、二进制名全部统一为 `lilium`：

- `binaries/lilium-cli/` → `binaries/lilium/`
- `package.name` = `lilium`，`[[bin]] name` = `lilium`
- 根 `Cargo.toml` workspace members 更新路径
- clap `name = "lilium"`，completion 生成的 bin name 用 `"lilium"`
- `AGENTS.md` 中所有 `lilium-cli`/`lilium-spider`/`lilium-event-processor` 引用更新为 `lilium`
- `binaries/lilium-spider` 和 `binaries/lilium-event-processor` 两个 crate 从 workspace 删除

### 调用形态

```
lilium ws-client                # 长运行 WS arbiter（原 lilium-spider）
lilium ws-client worker --account <id>   # 内部命令：arbiter 用它自拉起 worker 子进程
lilium event-processor          # 长运行 DB 轮询（原 lilium-event-processor）
lilium send-command <verb>      # 原有
lilium sync-members             # 原有
lilium sync-rooms               # 原有
lilium explore                  # 原有
lilium completion <shell>       # 新增：生成 shell 自动补全
```

所有动词平级，无命名空间分层。`spider` 重命名为 `ws-client`。`lilium ws-client worker --account <id>` 是 arbiter 自拉起子进程的内部入口，普通用户不应直接调用。

### 模块树

每个顶级动词对应 `src/commands/<name>/` 下的一个 module。`main.rs` 只做 clap 解析与
runtime bootstrap，分发到各命令 module 的 `run(...)` 入口。

```
binaries/lilium/src/
  main.rs                 # 统一 clap 分发 + runtime bootstrap
  config.rs               # 统一 Config
  commands/
    mod.rs                # re-export 各子命令 module + Verb enum
    ws_client/
      mod.rs              # arbiter/run 入口 + worker 子命令参数
      arbiter.rs          # 自 lilium-spider 迁入
      control.rs          # 自 lilium-spider 迁入
      ingestion.rs        # 自 lilium-spider 迁入
      worker.rs           # 自 lilium-spider 迁入
    event_processor.rs    # processor 自 lilium-event-processor 迁入（单文件 module）
    send_command.rs       # 原有，迁入
    sync_members.rs       # 原有，迁入
    sync_rooms.rs         # 原有，迁入
    explore.rs            # 原有，迁入
```

ws-client 的内部子模块拆为独立文件（arbiter/control/ingestion/worker），不挤进单个 `mod.rs`。arbiter 通过 `lilium ws-client worker --account <id>` self-re-exec 子进程，子进程逻辑从 `worker.rs` 进入。

### 统一 Config

把三个 `config.rs` 合并成一份 `binaries/lilium/src/config.rs`：

```rust
struct Config {
    database: DatabaseConfig,         // url + max_connections（共用）
    notification: NotificationConfig, // url（共用）
    spider: SpiderConfig,             // queue_size, batch_size, buffer_dir,
                                      //   runtime_dir, websocket_url,
                                      //   reconnect_delay_ms（原 worker 配置）
    processor: ProcessorConfig,       // polling_interval_secs, batch_size
    cli: CliConfig,                   // data_path
}
```

- 三份重复的 `env_*` helper 合并为一组公用 helper（`env_required_string` /
  `env_fallback_string` / `env_string` / `env_u32` / `env_u64` / `env_usize` / `env_path`）。
- 三个 crate 重复的 `From` impl（`DatabaseConfig`/`NotificationConfig` → `lilium_database` 类型）
  保留一份；spider 那份额外的 `DedicatedDatabaseConfig` From impl 也并入。
- 三份重复的单元测试（`fallback_string_prefers_primary_value`、`fallback_string_uses_fallback_value`、
  `notification_config_converts_url`）合并去重保留一份。

加载策略：`Config::load()` 一次性读全部 env，每个子命令按需取自己那部分字段。`DATABASE_URL`
仍为必需；未用到部分（如跑 `explore` 时 `processor` 字段）构造后不用，无副作用。

环境变量名保持不变（`DATABASE_URL`、`DATABASE_POOL_SIZE`、`DATABASE_NOTIFICATION_URL`、
`SPIDER_*`、`EVENT_PROCESSOR_*`、`DATA_PATH`）——`SPIDER_*` 前缀虽子命令改名 ws-client，
但 env 名属存储契约，沿用不动以免破坏部署侧 `.env`。

### Clap 分发 + 自动补全

`main.rs` 用单个 `#[derive(Parser)]` 的 `Cli` 驱动一切，flat `Verb` 枚举：

```rust
#[derive(Parser)]
#[command(name = "lilium", about = "Lilium unified binary")]
struct Cli {
    #[command(subcommand)]
    command: Verb,
}

#[derive(Subcommand)]
enum Verb {
    WsClient { /* args */ },
    EventProcessor { /* args */ },
    SendCommand { #[command(subcommand)] cmd: SendCommand },
    SyncMembers(SyncMembersArgs),
    SyncRooms(SyncRoomsArgs),
    Explore(ExploreArgs),
    /// Generate shell completion scripts
    Completion { #[arg(value_enum)] shell: clap_complete::Shell },
}
```

自动补全通过 `clap_complete` crate 生成；它从活的 `Cli` 定义派生补全，新增命令时自动保持同步。

workspace `Cargo.toml` 增加 `clap_complete = "4"` 依赖；`binaries/lilium/Cargo.toml` 引用之。

用法：

```sh
lilium completion bash  > /etc/bash_completion.d/lilium
lilium completion zsh   > _lilium           # 放进一个 fpath 目录
lilium completion fish  > ~/.config/fish/completions/lilium.fish
lilium completion powershell > _lilium.ps1
```

- `clap_complete::Shell` 已实现 `ValueEnum`（bash/zsh/fish/elvish/powershell），无需额外定义 flag。
- `clap_complete::generate` 需要一个 bin name 字符串，硬编码 `"lilium"` 以保证补全输出稳定
  （不用 `argv[0]`）。

### Worker 进程隔离

当前 Rust 实现是单进程多 task：arbiter 用 `tokio::spawn` 启动每个 account 的 `Worker`，
所有 worker 共享 arbiter 进程。这与 Python 原版（`ws_arbiter.py`/`ws_worker.py`）不同——
原版 worker 是独立进程，可单独 crash/重启。

合并为单一二进制后，用 **self-re-exec 子进程** 恢复进程隔离：

1. **arbiter 是 `lilium ws-client` 主进程。** 它维护当前启用 account 集合，并负责启动/停止/监控子进程。
2. **每个 account worker 是子进程。** arbiter 通过 `tokio::process::Command` spawn `lilium ws-client worker --account <id>`。
3. **子进程走 `commands/ws_client/mod.rs` 里的 worker 入口：** 解析 `--account`，直接实例化 `worker::Worker::new(...)` 并 `.run().await`。
4. **stdout/stderr 继承父进程：** tracing/sentry 日志流入同一流，Sentry 名与原本一致（子进程用 `ws_arbiter` 或更细的 `ws_worker` 维度可按实现选择，暂定统一 `ws_arbiter`）。

**WorkerSpawner 抽象改造：**

`arbiter.rs` 里已有 `WorkerSpawner` trait 和 `TokioWorkerSpawner` 实现，是理想的分发点。

- `TokioWorkerSpawner` 改名为 `ProcessWorkerSpawner`（或新增）。
- `spawn_worker` 内部不再 `tokio::spawn`，而是构造 `tokio::process::Command::new(current_exe())`，
  参数为 `ws-client worker --account <account_id>`，并注入所需环境变量（或用同一 `.env`）。
- `WorkerHandle` 从 `Arc<Notify>` 改成持有 `tokio::process::Child`（或子进程 PID + abort handle）。
- 子进程 exit 后，arbiter 的 watcher task 按退避策略自动重启 crash 的 worker。

**停止单个 worker：**

1. arbiter 先通过 worker 自己的 unix control socket 发送 graceful shutdown 命令（复用 `control.rs`）。
2. 等待 `WORKER_STOP_TIMEOUT_SECS`（默认 10s）。
3. 超时后发送 SIGTERM；再超时发送 SIGKILL。

全局 shutdown 时对所有 worker 并行执行上述流程，然后等全部子进程退出再清理 arbiter control socket。

**故障恢复：**

- 子进程非零退出或 panic 时，arbiter 标记该 account worker 为需重启。
- 退避策略：立即第一次，随后 `min(2^n * 100ms, 30s)`，避免崩溃循环。
- 若连续失败超过阈值，arbiter 进入该 account 的 back-off 挂起状态，可通过 `lilium send-command` 或 rescan 重新触发。

**配置不变量：**

- `SPIDER_BUFFER_DIR`、`SPIDER_RUNTIME_DIR` 等路径在父子进程间共享；arbiter 与 worker 都读同一 `.env` / `Config`。
- advisory lock 由 worker 子进程自行获取（保持原有语义，只是换到独立进程）。
- worker 子进程各自建立独立的 database pool / dedicated connection / notification listener，不再与父进程共享 pool。

**退出码：** worker 子进程成功退出返回 0；被 signal 或 panic 返回非 0；arbiter 据此判断是否需要重启。

### 错误处理 / 退出码 / 可观测性

**Bootstrap 统一在 `main.rs`：**

```rust
fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _sentry = lilium_common::observability::init_backend_sentry(sentry_name);
    let code = tokio::runtime::Builder::new_multi_thread()
        .enable_all().build().unwrap()
        .block_on(async_main())?;
    if code != 0 { std::process::exit(code as i32); }
    Ok(())
}
```

**Sentry name（保留 Python parity 区分）：** 按子命令选 sentry name：

- `ws-client` → `ws_arbiter`
- `event-processor` → `event_processor`
- 其余（send-command / sync-members / sync-rooms / explore / completion） → `lilium_cli`

这保留了原三个二进制的 sentry 维度，成本仅在 `async_main` 按 verb 选 name。

**退出码：** 统一走 `Result<u8>`。长运行服务成功返回 0、出错经 `?` 由 anyhow 转 exit 1。
保留现有退出码契约（AGENTS.md 点名可断言的 protocol 值）。

**Ctrl-C / shutdown：** spider（ws-client）与 event-processor 各自已有的 `tokio::signal::ctrl_c()`
+ `Notify` shutdown 逻辑原样保留在各命令模块内，main 不插手。

### 测试

- 现有各 crate 的 `#[cfg(test)]` 模块随模块迁移到 `binaries/lilium/` 下；config 测试去重保留一份。
- 不新增常量断言测试（遵守 AGENTS.md）。
- completion 命令本身无需测试。
- `ProcessWorkerSpawner` 可通过 mock trait 继续被单元测试；真实进程启动路径可通过集成测试覆盖。
- 验证命令：`cargo build -p lilium`、`cargo test -p lilium`、
  `cargo clippy --all-targets --all-features`、`cargo fmt --all`。

### 文档更新（同 change set）

- `AGENTS.md`：`lilium-spider`/`lilium-event-processor`/`lilium-cli` 引用统一改为 `lilium`；
  调用示例改为 `cargo run --bin lilium ws-client` / `event-processor` / 等；
  架构图顶部 binary 列表更新。
- 各被迁入模块保留原有 `// Python parity source:` 头注释不动。

## 非目标

- 不改动任何业务逻辑（arbiter/worker/processor/explore 等实现原样迁移）。
- 不改动环境变量名。
- 不引入 supervisord/daemon manager；但 ws-client 自身会管理 worker 子进程。
- 不为 completion 写测试。

## 风险

- **Sentry 维度**：若统一成一个 sentry name 会丢失组件区分；本设计按 verb 选 name 规避。
- **退出码语义**：长运行服务从 `Result<()>` 改走 `Result<u8>`，需确认出错路径仍正确转 exit 1；
  worker 子进程退出码需要正确区分 graceful shutdown（不重启）与 crash（重启）。
- **环境变量契约**：`SPIDER_*` 前缀虽改名 ws-client 但沿用，部署侧无需改 `.env`。
- **子进程管理复杂度**：worker 进程生命周期、信号处理、Zombie 进程回收、共享文件句柄关闭
  需仔细实现；`tokio::process` 已处理部分问题，但仍需测试 crash/重启路径。
