# models/common/base.py

## 功能
SQLModel 模型的基础配置，提供通用类型和工具函数。

## 类型

### FixedDecimal(TypeDecorator)
- 基础固定精度小数列类型
- `precision: int = 38, scale: int`

### FixedDecimal2(FixedDecimal)
- `DECIMAL(38,2)` 便捷类型

### FixedDecimal6(FixedDecimal)
- `DECIMAL(38,6)` 便捷类型

### FixedDecimal8(FixedDecimal)
- `DECIMAL(38,8)` 便捷类型

### BigNumber(TypeDecorator)
- `NUMERIC(38,0)` 列，将 DB Decimal 结果转换为 Python int

## 函数

```python
def utc_now() -> datetime
```
- 返回时区感知的 UTC 当前时间

```python
def parse_datetime(value: str) -> Optional[datetime]
```
- 解析 ISO 8601 字符串为 datetime

## 常量
```python
DECIMAL_CONTEXT_PRECISION = 50
```

## 依赖模块
- `datetime, timezone`
- `decimal.Decimal, DefaultContext, getcontext`
- `sqlalchemy.DECIMAL, TypeDecorator`

## Rust 映射
- 位置: `crates/lilium-common/src/utils.rs` (部分实现)
- 状态: ⚠️ 简化实现（缺少 FixedDecimal 类型）
