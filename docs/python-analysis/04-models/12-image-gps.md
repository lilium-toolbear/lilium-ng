# models/dzmm/image_gps.py

## 功能
统一的图片 GPS 数据存储，用于隐私检测和分析。

## 类

### ImageGPS(SQLModel, table=True)

**表名**: `image_gps`

**字段**:
- `message_id: str` - 主键（多态：message_id 或 tweet_id）
- `latitude: float` - 纬度
- `longitude: float` - 经度
- `altitude: Optional[float]` - 海拔
- `timestamp: Optional[datetime]` - GPS 时间戳
- `created_at: datetime` - 创建时间

## 依赖模块
- `sqlalchemy.TIMESTAMP, Column, Index, Text`
- `sqlalchemy.dialects.postgresql.ARRAY, JSONB`
- `sqlmodel.Field, SQLModel`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 非关键（媒体下载时使用）
