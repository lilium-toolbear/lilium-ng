# models/dzmm/user.py

## 功能
用户配置模型，跟踪用户信息、统计数据和历史。

## 类

### User(SQLModel, table=True)

**表名**: `users`

**字段**:
- `user_id: str` - 主键
- `full_name: Optional[str]` - 显示名称
- `name_tsv: Optional[str]` - 全文搜索向量
- `avatar_url: Optional[str]` - 头像 URL
- `avatar_file: Optional[str]` - 本地头像路径
- `bio: Optional[str]` - 个人简介
- `birthday: Optional[str]` - 生日
- `birthday_public: Optional[bool]` - 生日是否公开
- `quirk: Optional[str]` - 状态消息
- `is_bot: Optional[bool]` - 是否为机器人
- `gender: Optional[str]` - 性别
- `user_metadata: Optional[Dict]` - JSON 元数据
- `raw_data: Optional[Dict]` - 原始 API 响应
- `last_seen: Optional[datetime]` - 最后在线时间
- `message_count: int` - 消息计数
- `deleted_count: int` - 删除计数
- `recalled_count: int` - 撤回计数
- `created_at: datetime` - 创建时间
- `updated_at: datetime` - 更新时间

**索引**:
- `idx_users_last_seen` - (last_seen DESC)
- `idx_users_message_count` - (message_count DESC)

**方法**:
- `from_api(data) -> User` - 从 API 响应创建
- `update_from_api(data)` - 从 API 响应更新
- `increment_message_count()` - 增加消息计数
- `increment_deleted_count()` - 增加删除计数
- `increment_recalled_count()` - 增加撤回计数

## 依赖模块
- `sqlalchemy.Column, Index, TIMESTAMP`
- `sqlalchemy.dialects.postgresql.JSONB, TSVECTOR`
- `sqlmodel.Field, SQLModel`
- `models.common.base.utc_now, parse_datetime`

## Rust 映射
- 位置: `crates/lilium-models/src/dzmm/user.rs`
- 状态: ⚠️ 简化实现（缺少 from_api, update_from_api, 统计方法）
