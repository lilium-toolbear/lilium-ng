# models/dzmm/ 探索流模型汇总

## 概述
这些模型用于存储探索流内容（书籍、卡片、章节、检查点、画廊），不是 spider 核心功能。

## 模型列表

### Book (books 表)
- `book_id: str` - 主键
- `title, description, slug` - 基本信息
- `is_nsfw, is_public` - 内容标志
- `cover_image_url, local_cover_path` - 封面
- `user_id, author` - 作者信息
- `chapter_count, total_word_count` - 统计
- `likes_count, comments_count` - 互动
- `created_at, updated_at, published_at, fetched_at` - 时间戳
- `raw_data: Dict` - 原始数据

### Card (cards 表)
- `card_id: int` - 主键
- `name, card_filename, original_filename` - 文件信息
- `creator, creator_notes, user_id` - 创建者
- `tags: List[str]` - 标签
- `is_public, is_sensitive, is_image_blur, is_gamefy` - 标志
- `image_info: Dict` - 图片信息
- `weighted_rating, popularity_score` - 评分
- `likes_count, comments_count` - 互动

### Chapter (chapters 表)
- `chapter_id: str` - 主键
- `title, content` - 内容
- `is_adult, is_nsfw` - 内容标志
- `user_id, author` - 作者
- `likes_count, comments_count` - 互动

### Checkpoint (checkpoints 表)
- `checkpoint_id: str` - 主键
- `name, description` - 信息
- `is_public` - 公开标志
- `user_id, user_name, user_avatar_url` - 用户
- `rating_avg, rating_count` - 评分
- `share_code` - 分享码
- `character_cards: Dict` - 角色卡

### Gallery (galleries 表)
- `gallery_id: str` - 主键
- `title, description` - 信息
- `user_id, user_name` - 用户
- `image_urls: List[str]` - 图片 URL
- `likes_count, comments_count` - 互动

## Rust 映射
- 位置: 未实现
- 状态: ❌ 非关键（探索流内容，非 spider 核心）
