# 日志系统设计文档

**日期**: 2026-04-21  
**状态**: 已批准  
**实现方案**: tracing + tracing-appender

## 概述

为 OmniLink 添加后端日志功能,将日志按等级输出到应用目录的 `/logs` 文件夹,支持日志轮转和等级配置。

## 需求

### 功能需求

1. 支持标准四级日志等级: ERROR, WARN, INFO, DEBUG
2. 按等级分文件存储: `error.log`, `warn.log`, `info.log`, `debug.log`
3. 日志文件自动轮转: 单文件达到 10MB 时归档,保留最近 5 个归档文件
4. 日志格式: 时间戳 + 等级 + 模块 + 消息
5. 通过配置文件控制日志等级,重启应用生效
6. 默认日志等级: DEBUG
7. 记录关键操作: 链接添加、内容解析、AI 处理、数据库操作
8. 无需前端展示,用户自行到 `~/.omnilink/logs` 目录查看

### 非功能需求

1. 异步写入,不阻塞主线程
2. 日志写入失败不影响业务逻辑
3. 总磁盘占用控制在 240MB 以内(4 等级 × 6 文件 × 10MB)

## 整体架构

### 系统流程

```
应用启动
    ↓
初始化日志系统(main.rs)
    ↓
读取配置文件(~/.omnilink/config.json)
    ↓
创建日志目录(~/.omnilink/logs/)
    ↓
配置 tracing subscriber
    ├─ 按等级分文件写入器(error.log, warn.log, info.log, debug.log)
    ├─ 日志格式化器(时间戳 + 等级 + 模块 + 消息)
    └─ 日志等级过滤器(从配置读取,默认 DEBUG)
    ↓
应用运行期间
    ├─ 各模块使用 tracing 宏记录日志
    ├─ 自动按等级路由到对应文件
    └─ 达到 10MB 时自动轮转,保留 5 个归档
```

### 核心组件

#### 1. 日志初始化模块 (`src-tauri/src/logger/mod.rs`)

负责在应用启动时初始化日志系统:
- 读取配置文件中的日志等级
- 创建日志目录和文件
- 配置 tracing subscriber

#### 2. 日志配置 (`~/.omnilink/config.json`)

添加 `log_level` 字段:
```json
{
  "log_level": "DEBUG"
}
```

可选值: "ERROR", "WARN", "INFO", "DEBUG"  
默认值: "DEBUG"

#### 3. 日志文件结构

```
~/.omnilink/logs/
├── error.log       (当前错误日志)
├── error.log.1     (归档 1)
├── error.log.2     (归档 2)
├── error.log.3     (归档 3)
├── error.log.4     (归档 4)
├── error.log.5     (归档 5)
├── warn.log        (当前警告日志)
├── warn.log.1-5    (归档)
├── info.log        (当前信息日志)
├── info.log.1-5    (归档)
├── debug.log       (当前调试日志)
└── debug.log.1-5   (归档)
```

## 技术实现

### 技术选型

**方案**: tracing + tracing-appender

**理由**:
- Rust 生态标准方案,与 tokio 生态集成良好
- 结构化日志,支持字段级过滤
- 性能优秀,异步写入不阻塞主线程
- 社区活跃,文档完善,长期支持有保障

### 依赖项

在 `src-tauri/Cargo.toml` 中添加:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
tracing-appender = "0.2"
```

### 日志格式

每条日志的格式示例:

```
2026-04-21 10:30:45.123 [ERROR] [parser::extractor] Failed to extract content from URL: https://example.com
2026-04-21 10:30:46.456 [INFO] [commands::link] Link added successfully: id=123
2026-04-21 10:30:47.789 [DEBUG] [ai::openai_provider] API request: model=gpt-4, tokens=150
```

格式说明:
- **时间戳**: 精确到毫秒,格式 `YYYY-MM-DD HH:MM:SS.mmm`
- **等级**: 用方括号包裹,如 `[ERROR]`, `[INFO]`
- **模块**: 显示代码位置,如 `[parser::extractor]`
- **消息**: 包含关键上下文信息

### 日志轮转规则

- **触发条件**: 单个文件达到 10MB
- **轮转方式**: 
  - `error.log` → `error.log.1`
  - `error.log.1` → `error.log.2`
  - `error.log.2` → `error.log.3`
  - `error.log.3` → `error.log.4`
  - `error.log.4` → `error.log.5`
  - `error.log.5` 被删除
- **保留数量**: 每个等级最多保留 5 个归档文件
- **总磁盘占用**: 最多 4 个等级 × 6 个文件 × 10MB = 240MB

## 日志记录范围

### 需要记录的关键操作

#### 1. 链接管理 (INFO/ERROR)

- **添加链接**: `INFO` 记录 URL 和 ID
- **更新链接**: `INFO` 记录 ID 和变更字段
- **删除链接**: `INFO` 记录 ID
- **操作失败**: `ERROR` 记录错误原因和堆栈

#### 2. 内容解析 (INFO/WARN/ERROR/DEBUG)

- **开始解析**: `DEBUG` 记录 URL 和平台类型
- **平台识别**: `DEBUG` 记录识别结果
- **HTTP 抓取**: `DEBUG` 记录请求和响应状态
- **正文提取**: `DEBUG` 记录提取的内容长度
- **解析成功**: `INFO` 记录 URL 和内容摘要
- **解析失败**: `ERROR` 记录 URL 和错误详情
- **降级处理**: `WARN` 记录降级原因

#### 3. AI 处理 (INFO/WARN/ERROR/DEBUG)

- **API 调用**: `DEBUG` 记录模型、token 数量
- **调用成功**: `INFO` 记录处理结果摘要
- **调用失败**: `ERROR` 记录错误信息
- **Provider 切换**: `WARN` 记录切换原因
- **降级到规则引擎**: `WARN` 记录降级原因

#### 4. 数据库操作 (ERROR/DEBUG)

- **SQL 执行**: `DEBUG` 记录 SQL 语句(仅在 DEBUG 等级)
- **事务操作**: `DEBUG` 记录开始/提交/回滚
- **数据库错误**: `ERROR` 记录 SQL 和错误信息
- **连接池状态**: `DEBUG` 记录连接数

#### 5. 应用生命周期 (INFO/ERROR)

- **应用启动**: `INFO` 记录版本号和配置
- **应用关闭**: `INFO` 记录运行时长
- **配置加载**: `INFO` 记录配置路径
- **配置错误**: `ERROR` 记录错误详情

### 不记录的内容

- 用户敏感信息(密码、API Key 完整值)
- 完整的网页 HTML 内容
- 大量重复的轮询操作
- 正常的 UI 交互事件

## 实现计划

### 文件结构

```
src-tauri/src/
├── logger/
│   ├── mod.rs           (日志系统初始化和配置)
│   └── filter.rs        (自定义日志过滤器,按等级路由到不同文件)
├── main.rs              (在 main 函数中初始化日志)
└── lib.rs               (无需修改)
```

### 实现步骤

#### 步骤 1: 创建日志模块

创建 `src-tauri/src/logger/mod.rs`:
- 实现 `init_logger()` 函数
- 读取配置文件中的 `log_level`
- 创建日志目录 `~/.omnilink/logs/`
- 配置 tracing subscriber

#### 步骤 2: 实现按等级分文件

- 创建 4 个 `RollingFileAppender`(error/warn/info/debug)
- 配置每个文件的轮转策略(10MB, 5 个归档)
- 使用 `EnvFilter` 实现等级过滤

#### 步骤 3: 在 main.rs 中初始化

- 在 `main()` 函数开始处调用 `logger::init_logger()`
- 记录应用启动日志

#### 步骤 4: 在关键模块中添加日志

- 在 `commands/` 各模块中使用 `tracing::info!`, `tracing::error!` 等宏
- 在 `parser/` 模块中添加解析流程日志
- 在 `ai/` 模块中添加 AI 调用日志
- 在 `repositories/` 模块中添加数据库操作日志(仅 DEBUG 等级)

#### 步骤 5: 更新配置文件结构

- 在配置文件读取逻辑中添加 `log_level` 字段
- 提供默认值 "DEBUG"

### 错误处理

- **日志目录创建失败**: 打印到 stderr 并继续运行(不阻断应用启动)
- **配置文件读取失败**: 使用默认 DEBUG 等级
- **日志写入失败**: 不影响业务逻辑执行

### 测试验证

1. 启动应用,检查 `~/.omnilink/logs/` 目录是否创建
2. 执行各种操作,检查对应等级的日志文件是否有内容
3. 修改配置文件中的 `log_level`,重启应用,验证日志等级过滤是否生效
4. 生成大量日志,验证文件轮转是否正常工作

## 安全考虑

1. **敏感信息保护**: 不记录完整的 API Key、密码等敏感信息
2. **路径安全**: 日志目录固定为 `~/.omnilink/logs/`,不接受用户输入路径
3. **磁盘空间**: 通过轮转策略限制总磁盘占用(最多 240MB)
4. **权限控制**: 日志文件仅当前用户可读写

## 未来扩展

1. 支持动态调整日志等级(无需重启)
2. 添加日志查看器 UI
3. 支持远程日志上报
4. 添加性能指标日志
5. 支持结构化日志查询
