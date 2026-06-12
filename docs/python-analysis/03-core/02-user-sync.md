# core/user_sync.py

## 功能
共享用户配置同步逻辑，消除 spider、history_fetcher、explore_fetcher 之间的重复。

## 函数

```python
def _chunk_user_ids(user_ids: List[str], chunk_size: int = 5000) -> List[List[str]]
```
- 将用户 ID 分块，用于安全的 DB 参数计数

```python
async def _download_avatars_background(avatar_updates: List[Tuple[str, str]], log: logging.Logger) -> None
```
- 后台下载头像（fire-and-forget）
- 使用独立的数据库会话

```python
async def fetch_and_update_user(auth, session, user_id, room_id, cache_hours=1, log=None) -> Optional[User]
```
- 从 API 获取用户信息并更新数据库
- 支持智能缓存（默认 1 小时）

```python
async def batch_fetch_and_update_users(auth, session, user_room_pairs, log=None) -> Tuple[int, int]
```
- 批量获取和更新用户
- 返回 (new_count, updated_count)
- 按账户分组，每个账户使用自己的凭据

## 依赖模块
- `dzmm_client.DZMMApi`
- `models.User`
- `services.UserService`
- `core.media.MediaDownloader`
- `database.async_engine.get_async_session`

## Rust 映射
- 位置: `crates/lilium-services/src/user.rs`
- 状态: ⚠️ 简化实现（缺少完整的 API 调用逻辑）
