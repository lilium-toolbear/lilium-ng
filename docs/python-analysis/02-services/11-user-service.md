# services/user_service.py

## 功能
用户业务逻辑服务，处理用户配置、统计、历史。

## 类

### UserService(AsyncBaseService)

**方法**:
- `async get_by_id(user_id) -> Optional[User]`
  - 根据 ID 获取用户

- `async get_by_ids(user_ids) -> List[User]`
  - 批量获取用户

- `async search_users(filters, pagination) -> List[User]`
  - 搜索用户（支持 FTS）

- `async upsert_user(user_data) -> User`
  - 插入或更新用户

- `async increment_message_count(user_id) -> None`
  - 增加消息计数

- `async increment_deleted_count(user_id) -> None`
  - 增加删除计数

- `async increment_recalled_count(user_id) -> None`
  - 增加撤回计数

## 依赖模块
- `sqlmodel.select, func, col`
- `models.User, UserHistory`
- `services.base.AsyncBaseService`
- `services.types.UserFilters, PaginationParams`

## Rust 映射
- 位置: `crates/lilium-services/src/user.rs`
- 状态: ⚠️ 基本实现（只有 batch_fetch_and_update 和 fetch_user_profile）
