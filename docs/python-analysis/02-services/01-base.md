# services/base.py

## 功能
所有业务逻辑服务的基类，提供通用功能。

## 类型定义
```python
FreshSessionOpener = Callable[[], AsyncContextManager[AsyncSession]]
```

## 类

### AsyncBaseService

**构造参数**:
- `session: AsyncSession` - 异步数据库会话（通过依赖注入）

**方法**:
- `async commit()` - 提交当前事务
- `async rollback()` - 回滚当前事务
- `async refresh(instance)` - 刷新实例（先 flush 再 refresh）
- `add(instance)` - 添加实例到会话（同步）
- `async delete(instance)` - 删除实例
- `async flush()` - 刷新更改到数据库（不提交）

## 依赖模块
- `sqlmodel.ext.asyncio.session.AsyncSession`

## 设计模式
- **无状态服务**: 每个服务实例绑定一个会话
- **会话范围**: 会话由上下文管理器管理，自动提交/回滚
- **依赖注入**: 会话通过构造函数注入

## Rust 映射
- 位置: 不需要基类（Rust 使用 trait）
- 状态: N/A（Rust 模式不同）
