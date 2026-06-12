# dzmm_client/websocket.py

## 功能
DZMM WebSocket 事件处理，将 Socket.IO 事件解码为类型化的 Message 对象。

## 类

### WebSocketEventDecoder

**构造参数**:
- `logger: Optional[logging.Logger]` - 日志记录器

**方法**:
- `decode_data(data: Any) -> Optional[Dict]`
  - 解码 WebSocket 数据（二进制、JSON、字典）
  - 返回解码后的字典

- `classify_event(event_data: Dict) -> Tuple[str, bool]`
  - 分类事件类型
  - 返回 (事件类型, 是否为房间消息)

- `extract_room_id(event_data: Dict) -> Optional[str]`
  - 从各种 WebSocket 事件结构中提取房间 ID

- `extract_message_data(event_data: Dict) -> Optional[Dict]`
  - 提取消息数据

- `decode_message(event_data: Dict) -> Optional[Message]`
  - 将 WebSocket 事件解码为 Message 对象

## 依赖模块
- `json`
- `dzmm_client.models.Message`

## Rust 映射
- 位置: `crates/lilium-api-client/src/websocket.rs`
- 状态: ⚠️ 简化实现（缺少完整的解码逻辑）
