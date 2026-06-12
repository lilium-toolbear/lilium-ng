# spider/explore_sync.py

## 功能
探索流同步，定期同步探索内容（推文、卡片等）。

## 类

### ExploreFeedSync

**构造参数**:
- `logger: logging.Logger` - 日志记录器
- `shutdown_event: Optional[asyncio.Event]` - 关闭信号

**关键状态**:
- `min_interval: float` - 最小间隔（默认 280.0 秒 = 4:40）
- `max_interval: float` - 最大间隔（默认 320.0 秒 = 5:20）

**方法**:
- `async periodic_sync()` - 周期性同步
  - 随机延迟（避免检测）
  - 获取下一个可用账户
  - 创建 API 客户端
  - 使用 ExploreFetcher 获取和处理内容
  - 提交事务

## 数据流
```
periodic_sync()
  → 随机延迟 (4:40-5:20 分钟)
  → AccountService.get_next_available_account()
  → AccountService.create_auth_client()
  → ExploreFetcher(auth, session, data_path, config)
  → fetcher.fetch_and_process()
  → session.commit()
```

## 依赖模块
- `core.explore.ExploreFetcher`
- `core.explore.ExploreFetchConfig`
- `database.async_engine.get_async_session`
- `services.AccountService`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
