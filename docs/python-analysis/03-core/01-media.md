# core/media.py

## 功能
媒体处理 - 下载和处理所有媒体类型（图片、视频、语音、贴纸）。

## 类

### MediaDownloader

**构造参数**:
- `data_path: str` - 数据目录（默认 "./data"）

**方法**:
- `async close()` - 关闭 HTTP 客户端
- `async __aenter__()` / `async __aexit__()` - 异步上下文管理器
- `_get_ext_from_url(url) -> Optional[str]` - 从 URL 提取扩展名
- `_normalize_url(url) -> str` - 规范化 URL
- `_relative_data_path(file_path) -> str` - 获取相对路径
- `_is_public_ip_address(ip) -> bool` - 检查是否为公共 IP
- `async _validate_remote_url(url) -> str` - 验证 URL（SSRF 防护）
- `async download_avatar(avatar_url, user_id) -> Optional[str]` - 下载头像
- `async download_image(url, message_id) -> Optional[str]` - 下载图片
- `async download_video(url, message_id) -> Optional[str]` - 下载视频
- `async download_voice(url, message_id) -> Optional[str]` - 下载语音
- `async download_sticker(url, message_id) -> Optional[str]` - 下载贴纸

## 函数

```python
async def process_message_media(message_id: str, downloader: MediaDownloader) -> bool
```
- 处理消息媒体
- 下载附件
- 更新消息记录

## 依赖模块
- `httpx`
- `mutagen._file.File`
- `utils.setup_logging`
- `utils.image_utils.extract_gps_data`
- `database.async_engine.get_async_session`
- `models.ImageGPS`
- `models.dzmm.message.extract_content_attachment_url, merge_content_metadata`
- `dzmm_client.DZMM_HEADERS`
- `services.MessageService`

## Rust 映射
- 位置: `crates/lilium-services/src/media.rs`
- 状态: ⚠️ 简化实现（缺少完整的下载逻辑、GPS 提取、路径组织）
