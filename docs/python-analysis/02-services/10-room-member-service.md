# services/room_member_service.py

## 功能
房间成员管理服务。

## 类

### RoomMemberService(AsyncBaseService)

**方法**:
- `async get_member_info(room_id, user_id) -> Optional[RoomMember]`
  - 获取成员信息

- `async is_member(room_id, user_id) -> bool`
  - 检查用户是否为成员

- `async get_active_members_by_ids(room_id, user_ids, *, account_user_id=None) -> Dict[str, RoomMember]`
  - 按 ID 批量获取活跃成员

- `async upsert_member_simple(room_id, user_id, role, joined_at=None) -> None`
  - 简单插入或更新成员

- `async mark_member_left(room_id, user_id, left_at=None) -> bool`
  - 标记成员离开

- `async get_member_count(room_id) -> int`
  - 获取成员数量

- `async get_room_members(room_id, limit=50, offset=0) -> List[RoomMember]`
  - 获取房间成员列表

## 依赖模块
- `sqlalchemy.delete, func`
- `sqlalchemy.orm.aliased`
- `sqlmodel.select, col`
- `models.Room, RoomMember, User`
- `models.common.base.utc_now`
- `services.base.AsyncBaseService`

## Rust 映射
- 位置: `crates/lilium-services/src/room_member.rs`
- 状态: ⚠️ 基本实现（只有 upsert_member 和 mark_member_left）
