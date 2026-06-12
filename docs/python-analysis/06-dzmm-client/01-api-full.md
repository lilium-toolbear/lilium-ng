# dzmm_client/api.py (完整分析)

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
- 使用 ContextVar 跟踪重试状态

## 函数

```python
def _is_trpc_business_forbidden(response: httpx.Response) -> bool
```
- 检查 403 是否为 tRPC 业务 FORBIDDEN

```python
def _parse_trpc_response(response: list, index: int = 0, default=None)
```
- 解析 tRPC 响应，提取 JSON 数据

```python
def _extract_balanced_json_object(text: str, start: int) -> dict[str, Any]
```
- 从文本中提取平衡的 JSON 对象

```python
def _extract_balanced_json_array(text: str, start: int) -> str
```
- 从文本中提取平衡的 JSON 数组

```python
def _extract_next_flight_text(html: str) -> str
```
- 从 Next.js Flight 数据中提取文本

```python
def _extract_next_public_profile(html: str, user_id: str) -> dict[str, Any]
```
- 从 Next.js 用户页面提取公开配置

```python
def _extract_next_scalar_field(text: str, field: str) -> Any
```
- 从 Next Flight 文本中提取简单标量字段

## 类

### DZMMApi

**构造参数**:
- `email: Optional[str]` - 邮箱
- `password: Optional[str]` - 密码
- `signin_code: Optional[str]` - QR 码令牌
- `signin_code_image: Optional[bytes]` - QR 码图片
- `signin_code_image_mime: Optional[str]` - MIME 类型
- `cookies: Optional[str]` - 会话 cookies
- `user_id: Optional[str]` - 用户 ID
- `auto_refresh: bool` - 自动刷新
- `on_cookies_refreshed: Optional[Callable]` - Cookie 刷新回调

**关键方法**:

```python
async def _request(self, method, endpoint, **kwargs) -> httpx.Response
```
- 统一请求方法
- 自动重试 401/403
- Brotli 解压
- 随机请求 ID

```python
async def authenticate(self) -> bool
```
- 认证（调用 get_my_info）

```python
async def refresh_cookies(self) -> bool
```
- 刷新 cookies（token refresh → password → QR code）

```python
async def get_my_info(self) -> dict
```
- 获取当前用户信息

```python
async def get_user_info(self, user_id, room_id) -> dict
```
- 获取用户信息

```python
async def batch_get_user_info(self, pairs) -> list[dict]
```
- 批量获取用户信息（tRPC）

```python
async def fetch_room_messages(self, room_id, limit) -> list[dict]
```
- 获取房间消息

```python
async def send_heartbeat(self) -> None
```
- 发送心跳

```python
async def fetch_explore_feed(self, sort) -> dict
```
- 获取探索流

```python
async def upload_chat_image(self, room_id, image) -> str
```
- 上传聊天图片

```python
async def download_media(self, url) -> bytes
```
- 下载媒体

## 依赖模块
- `httpx`
- `dzmm_client.rate_limiter.RateLimiter`
- `dzmm_client.utils.generate_string`
- `utils.setup_logging`

## Rust 映射
- 位置: `crates/lilium-api-client/src/http.rs`
- 状态: ⚠️ 简化实现（缺少完整的 API 方法、认证逻辑、tRPC 批量请求）
