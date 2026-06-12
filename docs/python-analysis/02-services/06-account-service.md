# services/account_service.py

## 功能
DZMM 账户凭据管理服务。

## 类

### AccountServiceError(DomainServiceError)
- `code = "ACCOUNT_INVALID_REQUEST"`

### AccountService(AsyncBaseService)

**类级状态**:
- `_api_client_cache: Dict[str, DZMMApi]` - API 客户端缓存

**构造参数**:
- `session: AsyncSession`
- `open_independent_session: FreshSessionOpener | None`

**方法**:
- `async create_account(user_id, user_profile, email=None, password=None, signin_code=None, signin_code_image=None, signin_code_image_mime=None, cookies=None) -> DzmmAccount`
  - 创建新账户
  - 验证认证方法（email/password 或 signin_code 或 signin_code_image）

- `async get_account(user_id: str) -> Optional[DzmmAccount]`
  - 获取账户

- `async list_accounts(enabled_only=True) -> List[DzmmAccount]`
  - 列出所有账户

- `async get_api_client(account_user_id: str) -> DZMMApi`
  - 获取 API 客户端（带缓存）

- `async create_auth_client(account: DzmmAccount) -> DZMMApi`
  - 创建认证客户端

- `async get_next_available_account() -> Optional[DzmmAccount]`
  - 获取下一个可用账户

## 依赖模块
- `dzmm_client.api.DZMMApi`
- `database.async_engine.get_async_session`
- `models.ingestion.dzmm_account.DzmmAccount`
- `services.base.AsyncBaseService`
- `services.errors.DomainServiceError`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 未实现
