# models/ingestion/dzmm_account.py

## 功能
DZMM 账户模型，用于多账户管理。

## 类

### DzmmAccount(SQLModel, table=True)

**表名**: `dzmm_account`

**字段**:
- `user_id: str` - DZMM 用户 ID（主键，外键）
- `user_profile: Dict[str, Any]` - /api/me 响应（JSONB）
- `email: Optional[str]` - 邮箱（可选）
- `password: Optional[str]` - 密码（明文）
- `signin_code: Optional[str]` - QR 码令牌（旧版）
- `signin_code_image: Optional[bytes]` - QR 码图片
- `signin_code_image_mime: Optional[str]` - MIME 类型
- `cookies: Optional[str]` - 会话 cookies
- `is_enabled: bool` - 启用/禁用标志
- `created_at: datetime` - 创建时间
- `updated_at: datetime` - 更新时间

**认证方法**:
1. Email/Password
2. QR Code Token（旧版）
3. QR Code Image（新版）

**生命周期**:
1. Create: 通过 CLI 添加 → 调用 /api/me 验证 → INSERT
2. Refresh: Cookie 刷新 → 更新 user_profile
3. Deactivate: 防止新连接 → UPDATE is_enabled = FALSE
4. Delete: 连接关闭后 → DELETE

## 依赖模块
- `sqlmodel.Column, Field, SQLModel`
- `sqlalchemy.TIMESTAMP`
- `sqlalchemy.dialects.postgresql.JSONB`
- `models.common.base.utc_now`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
