# models/dzmm/user_history.py

## 功能
用户配置变更历史，存储用户配置的完整快照。

## 类

### UserHistory(SQLModel, table=True)

**表名**: `user_history`

**字段**:
- `id: Optional[int]` - 自增主键
- `user_id: str` - 用户 ID（外键）
- `full_name: Optional[str]` - 显示名称
- `avatar_url: Optional[str]` - 头像 URL
- `avatar_file: Optional[str]` - 本地头像路径
- `bio: Optional[str]` - 个人简介
- `birthday: Optional[str]` - 生日
- `birthday_public: Optional[bool]` - 生日是否公开
- `quirk: Optional[str]` - 状态消息
- `is_bot: Optional[bool]` - 是否为机器人
- `gender: Optional[str]` - 性别
- `user_metadata: Optional[Dict]` - JSON 元数据
- `raw_data: Optional[Dict]` - 原始数据
- `recorded_at: datetime` - 记录时间

## 依赖模块
- `sqlalchemy.Column, TIMESTAMP`
- `sqlalchemy.dialects.postgresql.JSONB`
- `sqlmodel.Field, SQLModel`
- `models.common.base.utc_now`
- `models.dzmm.user.User`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 非关键（历史追踪）
