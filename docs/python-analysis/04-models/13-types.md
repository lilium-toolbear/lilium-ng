# models/dzmm/types.py

## 功能
领域模型的共享枚举定义。

## 枚举类

### ContentType(StrEnum)
- TEXT, IMAGE, VIDEO, VOICE, STICKER, SYSTEM, RECALLED, DELETED

### ChatType(StrEnum)
- PUBLIC, PRIVATE, GROUP, CHANNEL, ONE_ON_ONE

### RoomMemberRole(StrEnum)
- OWNER, ADMIN, MEMBER

### RoomMemberEventType(StrEnum)
- JOIN, LEAVE, KICK, INVITE, PROMOTE, DEMOTE

## 设计说明
- 使用 plain string 而不是 enum 类型用于模型字段
- 枚举保留用于文档、向后兼容、验证逻辑

## 依赖模块
- `enum.StrEnum`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 需要实现（用于验证逻辑）
