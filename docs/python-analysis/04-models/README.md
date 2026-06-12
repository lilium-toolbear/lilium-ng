# Models 模块分析

## 目录结构

```
models/
├── common/           # 通用基础类型
├── dzmm/             # DZMM 核心模型
├── ingestion/        # 摄入层模型
├── wallet/           # 钱包模型
├── auth/             # 认证模型
├── bot/              # 机器人模型
├── clearing/         # 清算模型
├── economy/          # 经济模型
├── exchange/         # 交易所模型
├── farm/             # 农场模型
├── futures/          # 期货模型
├── games/            # 游戏模型
├── market/           # 市场模型
├── pal/              # 帕鲁模型
├── partner/          # 合作伙伴模型
├── platform/         # 平台模型
├── player/           # 玩家模型
├── poll/             # 投票模型
├── raid/             # 探索模型
├── season/           # 赛季模型
├── stock/            # 股票模型
├── trpg/             # TRPG 模型
├── turnip/           # 大头菜模型
└── wallet/           # 钱包模型
```

## 模型统计

| 模块 | 文件数 | 说明 |
|------|--------|------|
| common | 1 | 基础类型 |
| dzmm | 12 | 核心业务模型 |
| ingestion | 5 | 摄入层模型 |
| wallet | 3 | 钱包模型 |
| auth | 6 | 认证模型 |
| bot | 8 | 机器人模型 |
| clearing | 6 | 清算模型 |
| economy | 1 | 经济模型 |
| exchange | 4 | 交易所模型 |
| farm | 5 | 农场模型 |
| futures | 7 | 期货模型 |
| games | 9 | 游戏模型 |
| market | 4 | 市场模型 |
| pal | 5 | 帕鲁模型 |
| partner | 4 | 合作伙伴模型 |
| platform | 5 | 平台模型 |
| player | 3 | 玩家模型 |
| poll | 1 | 投票模型 |
| raid | 9 | 探索模型 |
| season | 1 | 赛季模型 |
| stock | 10 | 股票模型 |
| trpg | 2 | TRPG 模型 |
| turnip | 15 | 大头菜模型 |
| **总计** | **~120** | |

## 已分析的模型

| 模块 | 文件 | 状态 |
|------|------|------|
| ingestion | websocket_event.py | ✅ |
| ingestion | event_processor_offset.py | ✅ |
| ingestion | dzmm_account.py | ✅ |
| ingestion | websocket_connection.py | ✅ |
| ingestion | outgoing_command.py | ✅ |

## 待分析的模型

| 模块 | 文件 | 状态 |
|------|------|------|
| common | base.py | 待分析 |
| dzmm | message.py | 待分析 |
| dzmm | user.py | 待分析 |
| dzmm | room.py | 待分析 |
| dzmm | room_member.py | 待分析 |
| dzmm | tweet.py | 待分析 |
| dzmm | types.py | 待分析 |
| wallet | wallet.py | 待分析 |
| wallet | wallet_transaction.py | 待分析 |
| wallet | wallet_ids.py | 待分析 |
| farm | land.py | 待分析 |
| farm | land_assignment.py | 待分析 |
| farm | resource_production.py | 待分析 |
| farm | turnip_seed.py | 待分析 |
| turnip | price.py | 待分析 |
| turnip | order.py | 待分析 |
| turnip | inventory.py | 待分析 |
| futures | futures_order.py | 待分析 |
| futures | futures_position.py | 待分析 |
