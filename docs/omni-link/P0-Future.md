# P0-Future

## 新功能规划

### ~~新增操作日志~~ ✅

src-tauri 后端服务日志按照等级划分输出至应用目录 /logs 文件夹中

**状态**: 已完成
- 实现了基于 tracing 的日志系统
- 支持按等级分文件输出(debug/info/warn/error)
- 支持日志轮转(10MB/文件,最多保留5个)
- 支持动态配置日志等级
- 日志目录: `~/.omnilink/logs/`

