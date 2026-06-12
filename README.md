# DZMM Rust

DZMM.ai 系统的 Rust 重写版本。

## 目标

用 Rust 重写整个 DZMM 系统，同时偿还技术债：

- 内存降低 10-20x
- tokio 多线程真正并行处理
- 类型安全贯穿全栈
- 模块边界清晰，无循环依赖
- 核心逻辑纯函数化

## 架构

```
crates/
├── dzmm-common/       # 共享工具、错误类型、常量
├── dzmm-models/       # 数据模型
├── dzmm-database/     # 数据库层
├── dzmm-api-client/   # API 客户端
├── dzmm-core/         # 纯业务逻辑
└── dzmm-services/     # 服务层

binaries/
├── dzmm-spider/       # WebSocket 摄入服务
├── dzmm-bot/          # 聊天机器人
├── dzmm-web/          # Web UI 后端
└── dzmm-cli/          # 命令行工具
```

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

## 开发

```bash
# 编译
cargo build

# 测试
cargo test

# 运行 spider
cargo run --bin dzmm-spider

# 运行 bot
cargo run --bin dzmm-bot
```

## 文档

- [设计文档](DESIGN.md)
- [实现计划](PLAN.md)
