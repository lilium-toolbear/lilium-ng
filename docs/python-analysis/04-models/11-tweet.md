# models/dzmm/tweet.py

## 功能
推文模型，用于探索流内容和 GPS 跟踪。

## 类

### Tweet(SQLModel, table=True)

**表名**: `tweets`

**字段**:
- `tweet_id: str` - 主键
- `user_id: Optional[str]` - 作者 ID
- `content: Optional[str]` - 推文内容
- `media_urls: Optional[List[str]]` - 媒体 URL 数组
- `local_media_paths: Optional[List[str]]` - 本地文件路径数组
- `source: Optional[str]` - 平台来源
- `tweet_type: Optional[str]` - 推文类型
- `parent_tweet_id: Optional[str]` - 父推文 ID
- `is_reply: bool` - 是否为回复
- `is_retweet: bool` - 是否为转推
- `is_quote: bool` - 是否为引用
- `reply_count: int` - 回复数
- `retweet_count: int` - 转推数
- `like_count: int` - 点赞数
- `created_at: datetime` - 创建时间
- `fetched_at: datetime` - 获取时间
- `raw_data: Optional[Dict]` - 原始数据

## 依赖模块
- `sqlalchemy.TIMESTAMP, Column, Text`
- `sqlalchemy.dialects.postgresql.ARRAY, JSONB`
- `sqlmodel.Field, SQLModel`

## Rust 映射
- 位置: 未实现
- 状态: ❌ 非关键（探索流内容，非 spider 核心）
