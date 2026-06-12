# models/dzmm/room.py

## 功能
聊天室模型，跟踪房间元数据、统计和历史。

## 类

### Room(SQLModel, table=True)

**表名**: `rooms`

**字段**:
- `room_id: str` - 主键
- `title: str` - 房间名称
- `chat_type: Optional[str]` - 房间类型
- `avatar_url: Optional[str]` - 头像 URL
- `member_count: Optional[int]` - 成员数量
- `tags: Optional[List[str]]` - 标签数组
- `is_public: Optional[bool]` - 是否公开
- `creator_id: Optional[str]` - 创建者 ID
- `account_ids: List[str]` - 已加入的账户 ID 数组
- `last_message_at: Optional[datetime]` - 最后消息时间
- `first_message_at: Optional[datetime]` - 首条消息时间
- `backfill_until: Optional[datetime]` - 回填截止时间
- `history_complete: bool` - 历史是否完整
- `message_count: int` - 消息计数
- `deleted_count: int` - 删除计数
- `recalled_count: int` - 撤回计数
- `edited_count: int` - 编辑计数
- `image_count: int` - 图片计数
- `is_active: bool` - 是否活跃
- `dissolved_at: Optional[datetime]` - 解散时间
- `raw_data: Optional[Dict]` - 原始数据
- `created_at: datetime` - 创建时间
- `updated_at: datetime` - 更新时间

**索引**:
- `idx_rooms_is_active` - (is_active)
- `idx_rooms_last_message_at` - (last_message_at DESC)

## 依赖模块
- `sqlalchemy.Column, Index, Text, TIMESTAMP`
- `sqlalchemy.dialects.postgresql.ARRAY, JSONB`
- `sqlmodel.Field, SQLModel`
- `models.common.base.utc_now, parse_datetime`

## Rust 映射
- 位置: `crates/lilium-models/src/dzmm/room.rs`
- 状态: ⚠️ 简化实现（缺少 from_api, update_from_api, 统计方法）
