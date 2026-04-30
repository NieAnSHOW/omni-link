# 内置 Claude Code 设计文档

> 日期：2026-04-30
> 状态：设计已确认

## 概述

为 OmniLink 内置 Claude Code CLI，提供可交互式终端和内置 Skills 系统，让用户无需额外配置即可在应用内直接使用 Claude Code。

## 架构方案

采用**方案 B：新建 ClaudeManager 服务层**，与现有 PtyManager（persona rewrite）平行运作，互不影响。

```
┌─────────────────────────────────────────────────────┐
│                      Frontend                        │
│                                                      │
│  ┌──────────────────┐   ┌─────────────────────────┐ │
│  │  SettingsView     │   │  AppLayout (改)          │ │
│  │  新增 Claude Code  │   │  ├── 主内容区 (路由)     │ │
│  │  配置区块          │   │  └── BottomPanel (新增)  │ │
│  └──────────────────┘   │      └── InteractiveTerm  │ │
│                          └─────────────────────────┘ │
└─────────────────────────────────────────────────────┘
                        │ Tauri IPC invoke()
┌─────────────────────────────────────────────────────┐
│                      Backend (Rust)                   │
│                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐│
│  │ ClaudeManager │  │ CliDownloader│  │ SkillsMgr  ││
│  │ (会话管理)    │  │ (二进制下载) │  │ (参数注入) ││
│  └──────┬───────┘  └──────────────┘  └────────────┘│
│         │ PTY                                       │
│  ┌──────┴───────┐                                  │
│  │ Claude PTY    │  ← 复用 portable_pty crate       │
│  │ Session       │                                  │
│  └──────────────┘                                  │
│                                                      │
│  现有模块不受影响：PtyManager → persona rewrite       │
└─────────────────────────────────────────────────────┘
```

### 设计决策

- `ClaudeManager` 管理 Claude CLI 会话生命周期，与现有 `PtyManager` 独立
- `CliDownloader` 负责从镜像源下载 CLI 二进制到 `~/.omnilink/claude-cli/`
- `SkillsManager` 将 `src-tauri/skills/` 的 skills 通过 `--add-dir` 参数注入
- 底部面板挂载在 `AppLayout` 中，所有页面可见

## CLI 下载管理 (CliDownloader)

### 存储路径

```
~/.omnilink/claude-cli/
├── v1.0.0/
│   └── claude          # CLI 二进制
├── current -> v1.0.0/  # 符号链接指向当前版本
└── download.lock       # 下载锁，防止并发下载
```

### 镜像策略（多源自动切换）

- 内置多个国内镜像源（如 `ghfast.top`、`ghproxy.net`、`gh-proxy.com` 等）+ GitHub 官方源
- 下载时依次尝试，选择首个响应成功的源
- 超时阈值：单源 5s 连接超时，整体 30s
- 设置页面显示"下载源"为只读（显示当前使用的源），提供"重新测试"按钮

### 下载流程

1. 应用启动或用户点击"安装 Claude Code"时触发
2. 检测 `~/.omnilink/claude-cli/current` 是否存在且可用（执行 `claude --version`）
3. 若不可用，依次尝试镜像源列表下载对应平台的二进制
4. 下载过程通过 Tauri event (`cli-download-progress`) 推送进度到前端
5. 下载完成后设置可执行权限，创建/更新 `current` 符号链接

### Tauri Commands

- `check_claude_installed` → `{ installed: bool, version: String | null }`
- `install_claude_cli` → 触发下载，流式推送进度
- `get_claude_cli_path` → 返回当前可用的 CLI 路径

### 错误处理

- 网络失败（所有源均不可用）：返回错误 + 重试按钮
- 权限不足：提示用户手动处理
- 磁盘空间不足：提前检查

## 交互式终端 (InteractiveTerminal)

### 底部面板布局

类似 VS Code 集成终端的底部面板，通过**拖拽分割条**调整高度，双击分割条收起/展开。

- 收起状态：仅显示主内容区 + 底部状态栏（显示 Claude Code 就绪状态）
- 展开状态：主内容区高度缩减，底部显示交互式终端
- 分割条支持拖拽自由调整面板高度

### 核心行为

- xterm.js 渲染（复用现有依赖），启用 stdin 输入
- 用户击键 → Tauri event (`claude-pty-input`) → `ClaudeManager` 写入 PTY master
- PTY 输出 → Tauri event (`claude-pty-output`) → xterm.js 渲染
- 窗口大小变化时同步 resize PTY（`TIOCSWINSZ`）
- Claude CLI 退出时面板显示"会话已结束" + 重启按钮

### 会话生命周期

- 首次展开面板时：检测 CLI → 未安装则触发下载 → 下载完成后自动启动 `claude` 会话
- 关闭面板：可选择保持会话运行（后台）或终止会话
- 应用退出：自动终止所有 Claude PTY 会话

## 设置页面（Claude Code 配置区块）

在现有 `SettingsView.vue` 中新增独立的 Claude Code 配置区域，位于 AI Provider 配置下方。

### 配置项

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| Claude CLI 状态 | 只读展示 | — | 已安装 v1.0.0 / 未安装（+ 安装按钮） |
| API Provider | 下拉选择 | Anthropic | Anthropic / OpenAI Compatible / 自定义 |
| API Key | 密码输入 | — | 认证凭据 |
| Base URL | 文本输入 | https://api.anthropic.com | 请求地址（切换 Provider 时自动填充） |
| Model | 文本输入 | claude-sonnet-4-20250514 | 模型映射 |
| 下载源 | 只读展示 | 自动选择 | 显示当前镜像源 + "重新测试"按钮 |

### 配置存储

写入 `~/.omnilink/config.json` 的 `claude_code` 区块：

```json
{
  "claude_code": {
    "provider": "anthropic",
    "api_key": "sk-...",
    "base_url": "https://api.anthropic.com",
    "model": "claude-sonnet-4-20250514",
    "mirror": "auto"
  }
}
```

Claude CLI 启动时通过环境变量注入配置（`ANTHROPIC_API_KEY`、`ANTHROPIC_BASE_URL` 等）。

## 内置 Skills 注入

### 核心机制

Claude CLI 启动时通过 `--add-dir` 参数将 skills 目录注入上下文：

```bash
claude --add-dir ~/.omnilink/skills/
```

### Skills 部署流程

1. 应用首次启动或 skills 文件更新时，将 `src-tauri/skills/` 下所有 `.md` 文件和目录复制到 `~/.omnilink/skills/`
2. 使用 `include_str!` 将编译时的 skills 内容打包进二进制（与现有 `builtin.rs` 模式一致）
3. `ClaudeManager` 启动会话时自动附加 `--add-dir ~/.omnilink/skills/`

### 与现有 persona 系统的关系

- 现有 persona rewrite 流程不变，继续从 `~/.claude/plugins/cache/` 读取 skills
- 新的交互式终端通过 `--add-dir` 指向独立的 `~/.omnilink/skills/` 目录
- 两套 skills 内容可以相同，但路径独立，互不干扰

## 新增模块清单

### Backend (Rust)

| 模块 | 路径 | 职责 |
|------|------|------|
| `cli_downloader` | `src-tauri/src/claude/cli_downloader.rs` | 镜像源管理、二进制下载、版本检测 |
| `claude_manager` | `src-tauri/src/claude/manager.rs` | PTY 会话管理、环境变量注入、生命周期 |
| `skills_manager` | `src-tauri/src/claude/skills.rs` | Skills 部署、路径管理 |
| `claude_commands` | `src-tauri/src/commands/claude_commands.rs` | Tauri 命令入口 |

### Frontend (Vue)

| 组件 | 路径 | 职责 |
|------|------|------|
| `BottomPanel` | `src/components/layout/BottomPanel.vue` | 底部面板容器，拖拽分割条 |
| `InteractiveTerminal` | `src/components/claude/InteractiveTerminal.vue` | 可交互 xterm.js 终端 |
| `ClaudeSettings` | `src/components/settings/ClaudeSettings.vue` | Claude Code 配置区块 |

### 配置扩展

- `config.json` 新增 `claude_code` 区块
- `AppConfig` / `ClaudeConfig` Rust 结构体扩展
- `SettingsView.vue` 集成 `ClaudeSettings` 组件
