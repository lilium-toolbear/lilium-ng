# Lilium NG

DZMM.ai 监控和归档系统的 Rust 重写版本。

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
├── lilium-common/       # 共享工具、错误类型、常量
├── lilium-models/       # 数据模型
├── lilium-database/     # 数据库层
├── lilium-api-client/   # API 客户端
├── lilium-core/         # 纯业务逻辑
└── lilium-services/     # 服务层

binaries/
├── lilium-spider/       # WebSocket 摄入服务
├── lilium-bot/          # 聊天机器人
├── lilium-web/          # Web UI 后端
└── lilium-cli/          # 命令行工具
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
| 配置 | dotenvy + 环境变量 |
| 日志 | tracing |
| CLI | clap |

## 开发

```bash
# 编译
cargo build

# 测试
cargo test

# 运行 spider
cargo run --bin lilium-spider

# 运行 bot
cargo run --bin lilium-bot
```

## 文档

- [交接文档](docs/HANDOFF.md)
- [测试迁移目标](docs/TEST_MIGRATION_GOAL.md)
- [Rust 代码审计最新状态](docs/rust-code-audit.md)
- [Python 入口点 parity 审计](docs/python-parity-entrypoint-audit.md)
- [Python 代码库逐文件分析](docs/python-analysis/README.md)
