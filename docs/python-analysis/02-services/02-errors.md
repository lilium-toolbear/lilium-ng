# services/errors.py

## 功能
服务层共享的异常基类。

## 类

### ServiceError(Exception)
**属性**:
- `code: str` - 错误代码
- `message: str` - 错误消息
- `status_code: int` - HTTP 状态码
- `retryable: bool` - 是否可重试
- `details: Any | None` - 详细信息
- `headers: dict[str, str] | None` - HTTP 头

**构造参数**:
- `message: str | None` - 错误消息
- `code: str | None` - 错误代码
- `status_code: int | None` - HTTP 状态码
- `retryable: bool | None` - 是否可重试
- `details: Any | None` - 详细信息
- `headers: Mapping[str, str] | None` - HTTP 头

### DomainServiceError(ServiceError, ValueError)
**属性**:
- `code = "INVALID_REQUEST"`
- `status_code = 400`

## 依赖模块
无

## Rust 映射
- 位置: `crates/lilium-common/src/error.rs`
- 状态: ⚠️ 简化实现（缺少完整的错误属性）
