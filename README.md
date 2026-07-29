# Lilium NG

DZMM/Lilium 后端的 Rust 重写版本。

## 目标

用 Rust 重写核心摄入和事件处理链路，同时偿还技术债：

- tokio 多线程真正并行处理
- 类型安全贯穿全栈
- 模块边界清晰，无循环依赖
- 数据库、服务层、外部 API 边界显式

## 架构

```text
crates/
├── lilium-common/        # 共享工具、错误类型、常量
├── lilium-models/        # 数据模型
├── lilium-database/      # Database runtime、ORM entities、raw SQL session
├── lilium-test-fixtures/ # 测试数据库租约、reset、seed profiles
├── lilium-api-client/    # DZMM HTTP 和 Socket.IO 客户端
├── lilium-core/          # 纯业务逻辑
└── lilium-services/      # 服务层编排

binaries/
├── lilium-spider/          # WebSocket 摄入服务
└── lilium-event-processor/ # WebSocket 事件处理服务
```

## 技术栈

| 用途 | 选择 |
|------|------|
| 异步运行时 | tokio |
| 数据库 | SQLx + SeaORM (PostgreSQL) |
| WebSocket | rust_socketio fork |
| HTTP | reqwest |
| 序列化 | serde + serde_json |
| 配置 | dotenvy + 环境变量 |
| 可观测性 | tracing + Sentry |
| CLI | clap |

## 开发

```bash
# 首次运行（以及 Cargo.lock / flake.lock 更新后）创建持久 GC root，
# 保留 Rust 工具链、Cargo 源码、已编译依赖和开发工具
nix build .#cache --out-link .nix-cache

# GC 后也会复用上面的工具链和依赖
nix run .

# 编译
cargo build

# 测试
cargo test

# 运行 spider
cargo run --bin lilium-spider

# 运行事件处理器
cargo run --bin lilium-event-processor
```

`.nix-cache` 是一个被 Git 忽略的本地 symlink，同时也是 Nix GC root。普通源码
修改不需要刷新它；`Cargo.lock` 或 `flake.lock` 改变后重新执行上述 `nix build`
命令即可。删除 `.nix-cache` 会解除保护，之后这些 store paths 可以被 GC 回收。
