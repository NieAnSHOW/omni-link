# P0 优化 - 2026-04-22

## 问题描述

删除 link 时报错:
```
FOREIGN KEY constraint failed
```

所有删除操作都失败,无法删除知识列表中的数据。

## Root Cause 分析

通过系统性调试发现:

1. **外键约束已启用**: `database.rs` 中正确设置了 `PRAGMA foreign_keys = ON`
2. **Schema 定义正确**: `contents` 和 `ai_results` 表都定义了 `ON DELETE CASCADE`
3. **实际约束错误**: 数据库中 `ai_results` 表的外键约束实际是 `NO ACTION`,而不是 `CASCADE`

### 问题根源

`drop_classification` 迁移函数(schema.rs:140-162)在重建 `ai_results` 表时使用了**表级外键约束**语法:

```sql
CREATE TABLE ai_results_new (
    content_id INTEGER NOT NULL UNIQUE,
    FOREIGN KEY (content_id) REFERENCES contents(id) ON DELETE CASCADE
);
```

而初始 schema 使用的是**内联外键约束**语法:

```sql
CREATE TABLE ai_results (
    content_id INTEGER NOT NULL UNIQUE REFERENCES contents(id) ON DELETE CASCADE
);
```

在某些情况下,表级外键约束在 `execute_batch` 中可能没有被正确应用。

### 删除链路

```
links(id) → contents(link_id) → ai_results(content_id)
                              → content_tags(content_id)
```

当删除 link 时:
1. 触发 `contents` 的级联删除 ✓
2. `contents` 删除时应触发 `ai_results` 的级联删除
3. 但由于 `ai_results` 的外键约束是 `NO ACTION`,删除被阻止 ✗

## 解决方案

### 1. 修复迁移函数

统一使用内联外键约束语法:

```rust
fn drop_classification(conn: &Connection) -> AppResult<()> {
    // ...
    conn.execute_batch(
        "CREATE TABLE ai_results_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_id INTEGER NOT NULL UNIQUE REFERENCES contents(id) ON DELETE CASCADE,
            // ...
        );"
    )?;
}
```

### 2. 添加修复迁移

新增 `fix_ai_results_foreign_key` 函数,自动检测并修复已存在数据库中的外键约束问题:

```rust
fn fix_ai_results_foreign_key(conn: &Connection) -> AppResult<()> {
    let mut stmt = conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='ai_results'")?;
    let sql: String = stmt.query_row([], |row| row.get(0))?;

    if !sql.contains("ON DELETE CASCADE") {
        // 重建表,添加正确的外键约束
        conn.execute_batch("...")?;
    }
    Ok(())
}
```

## 验证

测试删除操作:

```sql
-- 创建测试数据
INSERT INTO links VALUES (21, 'https://test.com', ...);
INSERT INTO contents VALUES (15, 21, ...);
INSERT INTO ai_results VALUES (1, 15, ...);

-- 删除 link
DELETE FROM links WHERE id=21;

-- 验证级联删除
-- Links: 0 ✓
-- Contents: 0 ✓
-- AI Results: 0 ✓
```

## 影响范围

- **修复文件**: `src-tauri/src/db/schema.rs`
- **影响功能**: 所有删除 link 的操作
- **数据迁移**: 自动执行,无需手动干预

## 经验教训

1. **外键约束语法**: SQLite 支持两种外键约束语法,内联语法更可靠
2. **迁移验证**: 表结构迁移后应验证外键约束是否正确应用
3. **系统性调试**: 遵循 Root Cause Investigation → Pattern Analysis → Hypothesis Testing → Implementation 流程,避免盲目修复

## 相关文件

- `src-tauri/src/db/schema.rs` - Schema 定义和迁移
- `src-tauri/src/db/database.rs` - 数据库初始化
- `src-tauri/src/repositories/link_repo.rs` - Link 删除逻辑
