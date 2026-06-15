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
# 编译
cargo build

# 测试
cargo test

# 运行 spider
cargo run --bin lilium-spider

# 运行事件处理器
cargo run --bin lilium-event-processor
```
