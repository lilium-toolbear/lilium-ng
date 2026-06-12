# dzmm_client/api.py

## 功能
DZMM API 客户端，统一处理所有 API 端点。

## 常量
```python
DZMM_HEADERS = {
    "User-Agent": "Mozilla/5.0 ...",
    "Accept-Language": "en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7",
    "Accept-Encoding": "gzip, deflate",
    "Cache-Control": "no-cache",
    "Pragma": "no-cache",
    "Sec-Ch-Ua": ...,
    ...
}
```

## 装饰器

```python
def _with_auth_retry(func)
```
- 自动 401/403 重试
- 刷新 cookies 后重试一次

## 函数

```python
def _is_trpc_business_forbidden(response: httpx.Response) -> bool
```
- 检查 403 是否为 tRPC 业务 FORBIDDEN

```python
def _parse_trpc_response(response: list, index: int = 0, default=None)
```
- 解析 tRPC 响应

```python
def _extract_balanced_json_object(text: str, start: int) -> dict[str, Any]
```
- 从文本中提取 JSON 对象

## 类

### DZMMApi

**构造参数**:
- `base_url: str` - API 基础 URL
- `auto_refresh: bool` - 是否自动刷新认证
- `on_cookies_refreshed: Optional[Callable]` - Cookie 刷新回调

**关键方法**:
- `async _request(method, endpoint, **kwargs) -> httpx.Response`
  - 统一请求方法
  - 自动重试 401/403
  - Brotli 解压
  - 随机请求 ID

- `async authenticate() -> None`
  - 认证（token refresh → email/password → QR code）

- `async refresh_cookies() -> bool`
  - 刷新 cookies

- `async get_user_info(user_id, room_id) -> Dict`
  - 获取用户信息

- `async batch_get_user_info(pairs) -> List[Dict]`
  - 批量获取用户信息（tRPC）

- `async fetch_room_messages(room_id, limit) -> List[Dict]`
  - 获取房间消息

- `async send_heartbeat() -> None`
  - 发送心跳

- `async fetch_explore_feed(sort) -> Dict`
  - 获取探索流

- `async upload_chat_image(room_id, image) -> str`
  - 上传聊天图片

- `async download_media(url) -> bytes`
  - 下载媒体

## 依赖模块
- `httpx`
- `dzmm_client.rate_limiter.RateLimiter`
- `dzmm_client.utils.generate_string`
- `utils.setup_logging`

## Rust 映射
- 位置: `crates/lilium-api-client/src/http.rs`
- 状态: ⚠️ 简化实现（缺少完整的 API 方法）
