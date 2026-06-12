# spider/ws_control.py

## 功能
控制协议，用于 arbiter 和 worker 之间的通信。

## 类型
```python
ControlAction = Literal["status", "reconnect", "reload", "stop", "start", "restart", "rescan"]
ACCOUNT_ACTIONS: frozenset[ControlAction] = {"reconnect", "reload", "stop", "start", "restart"}
ARBITER_ACTIONS: frozenset[ControlAction] = {"status", "rescan"}
```

## 类

### ControlCommand
```python
@dataclass(frozen=True, slots=True)
class ControlCommand:
    action: ControlAction
    account_user_id: str | None = None
    reason: str = "requested"
    data: dict[str, Any] | None = None
    
    def to_json(self) -> str
    @classmethod
    def from_json(cls, raw: str | bytes) -> "ControlCommand"
```
- 验证 action 是否有效
- 验证 account_user_id 是否为 canonical UUID
- 验证 reason 是否为字符串
- 验证 data 是否为字典

### ControlResponse
```python
@dataclass(frozen=True, slots=True)
class ControlResponse:
    ok: bool
    message: str
    data: dict[str, Any] | None = None
    
    def to_json(self) -> str
    @classmethod
    def from_json(cls, raw: str | bytes) -> "ControlResponse"
```

## 函数
```python
def validate_account_user_id(account_user_id: str) -> str
```
- 验证 UUID 格式
- 验证 canonical 形式

```python
def runtime_path_id(account_user_id: str) -> str
```
- 生成运行时路径 ID

```python
def arbiter_socket_path(runtime_dir: Path) -> Path
```
- 生成 arbiter socket 路径

```python
def worker_socket_path(runtime_dir: Path, account_user_id: str) -> Path
```
- 生成 worker socket 路径

```python
async def bind_unix_control_socket(socket_path: Path) -> tuple[Socket, UnixSocketIdentity]
```
- 绑定 Unix socket
- 检查并移除陈旧 socket

```python
async def read_command(reader: asyncio.StreamReader) -> ControlCommand
```
- 从 reader 读取命令

```python
async def write_command(writer: asyncio.StreamWriter, command: ControlCommand) -> None
```
- 向 writer 写入命令

```python
async def read_response(reader: asyncio.StreamReader) -> ControlResponse
```
- 从 reader 读取响应

```python
async def write_response(writer: asyncio.StreamWriter, response: ControlResponse) -> None
```
- 向 writer 写入响应

```python
def remove_stale_or_refuse_unix_socket(socket_path: Path) -> bool
```
- 检查并移除陈旧 socket

```python
def unlink_bound_unix_socket(socket_path: Path, identity: UnixSocketIdentity) -> None
```
- 安全移除已绑定的 socket

## 依赖模块
无外部依赖

## Rust 映射
- 位置: `binaries/lilium-spider/src/control.rs`
- 状态: ✅ 基本实现（缺少完整的 socket 管理逻辑）
