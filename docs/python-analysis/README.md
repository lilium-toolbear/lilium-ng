# Python 代码库逐文件分析

## 目录结构

```
docs/python-analysis/
├── README.md                    # 本文件 - 索引
├── 01-spider/                  # Spider 子系统
├── 02-services/                # 服务层
├── 03-core/                    # 核心业务逻辑
├── 04-models/                  # 数据模型
├── 05-database/                # 数据库层
├── 06-dzmm-client/             # API 客户端
├── 07-bots/                    # 聊天机器人
├── 08-archive-ui/              # Web UI 后端
├── 09-toolbear-ui/             # ToolBear UI
├── 10-cli/                     # CLI 工具
└── SUMMARY.md                  # 汇总表
```

## 分析方法

每个文件的分析包括：
1. 功能描述
2. 所有类/函数及其签名
3. 调用了哪些模块
4. 被哪些文件调用
5. Rust 映射位置和状态

## 文件统计

| 模块 | Python 文件数 | 代码行数 |
|------|---------------|----------|
| spider/ | 11 | ~3,500 |
| services/ | 124 | ~15,000 |
| core/ | 85 | ~10,000 |
| models/ | 159 | ~8,000 |
| database/ | 10 | ~2,000 |
| dzmm_client/ | 6 | ~3,000 |
| bots/ | ~30 | ~5,000 |
| archive_ui/ | ~20 | ~3,000 |
| **总计** | **~445** | **~45,000** |
