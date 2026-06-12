# models/dzmm/room_member.py

## 功能
房间成员模型，跟踪用户在房间中的成员关系、角色和设置。

## 类

### RoomMember(SQLModel, table=True)

**表名**: `room_members`

**字段**:
- `room_id: str` - 复合主键
- `user_id: str` - 复合主键
- `role: Optional[str]` - 成员角色
- `joined_at: Optional[datetime]` - 加入时间
- `left_at: Optional[datetime]` - 离开时间
- `raw_data: Optional[Dict]` - 原始数据
- `created_at: datetime` - 创建时间
- `updated_at: datetime` - 更新时间

**方法**:
- `from_api(data, room_id) -> RoomMember` - 从 API 创建

## 依赖模块
- `sqlalchemy.TIMESTAMP`
- `sqlalchemy.dialects.postgresql.JSONB`
- `sqlmodel.Column, Field, SQLModel`
- `models.common.base.utc_now, parse_datetime`

## Rust 映射
- 位置: `crates/lilium-models/src/dzmm/room_member.rs` (未创建)
- 状态: ❌ 未实现
