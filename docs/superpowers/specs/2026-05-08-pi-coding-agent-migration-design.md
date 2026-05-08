# Pi Coding Agent Migration Design

## Overview

将 OmniLink 内置的 Claude Code 集成替换为 pi-coding-agent（`@mariozechner/pi-coding-agent`）。核心诉求：更轻量的智能体，原生支持 Agent Skills 标准调用。

**方案**：CLI 直接替换——保持 PTY + xterm.js 架构不变，底层二进制从 `claude` 换为 `pi`。

## Architecture

```
替换前：
  Frontend (xterm.js) ↔ Tauri IPC ↔ ClaudeManager ↔ PTY ↔ claude CLI (standalone binary)

替换后：
  Frontend (xterm.js) ↔ Tauri IPC ↔ PiManager ↔ PTY ↔ pi CLI (Node.js)
```

### 删除的模块

- `claude/cli_downloader.rs` — Claude CLI 下载器
- `claude/manager.rs` — ClaudeManager
- `claude/skills.rs` — SkillsManager
- `claude/mod.rs` — claude 模块入口
- `terminal/pty_manager.rs` — 旧 Persona PTY 系统
- `terminal/session.rs` — 旧 PTY 会话
- `terminal/mod.rs` — terminal 模块入口
- `commands/claude_commands.rs` — Claude 专属 commands
- `commands/terminal_commands.rs` — Persona rewrite commands
- `persona/builtin.rs` — 旧 skills 部署到 `~/.claude/plugins/cache/` 的逻辑

### 新增/重写的模块

- `pi/runtime.rs` — Node.js runtime 检测与下载
- `pi/manager.rs` — PiManager（PTY 会话管理 + persona rewrite）
- `pi/skills.rs` — Skills 部署到 `~/.pi/agent/skills/`
- `pi/mod.rs` — pi 模块入口
- `commands/pi_commands.rs` — 精简后的 Tauri commands

### 保留不变的模块

- 前端 `InteractiveTerminal.vue`（xterm.js 渲染逻辑）
- 前端 `BottomPanel.vue`（拖拽面板）
- 前端 `ClaudeSettings.vue` → 重命名为 `AgentSettings.vue`
- 数据库 schema、notes 相关模块

## Pi Installation & Runtime

### Node.js Runtime

策略：优先使用系统 Node.js，不存在则自动下载。

```
检测流程：
1. 检查系统 PATH 中是否有 node >= 18
2. 检查 ~/.omnilink/node/bin/node 是否存在
3. 都没有 → 下载 Node.js LTS 到 ~/.omnilink/node/
   - macOS: node-v22.x-darwin-arm64.tar.gz
   - Linux: node-v22.x-linux-x64.tar.gz
```

### Pi CLI 安装

```bash
~/.omnilink/node/bin/npm install -g --prefix ~/.omnilink/pi-cli @mariozechner/pi-coding-agent
# 最终可执行路径：~/.omnilink/pi-cli/bin/pi
```

### Rust 端

```rust
struct PiRuntime {
    node_path: PathBuf,
    pi_path: PathBuf,
}

impl PiRuntime {
    fn check_installed() -> PiStatus;
    fn install_node() -> Result<()>;
    fn install_pi() -> Result<()>;
    fn get_pi_command() -> Command;
}
```

## Tauri Commands

原来 13 个 Claude Commands + 6 个 Terminal Commands → 精简为 6 个：

```
1. check_pi_installed       — 检测 node + pi 是否可用
2. install_pi                — 安装 Node.js + pi CLI（带进度事件）
3. start_pi_session          — 创建交互式 PTY 会话
4. start_pi_output           — 启动输出读取循环
5. write_pi_input            — 写入用户输入
6. resize_pi_terminal        — 调整终端大小
```

## Skills Management

### 部署策略

pi 原生从 `~/.pi/agent/skills/` 自动发现 skills。内置 skills 部署到该路径，无需 `--add-dir`。

```
~/.pi/agent/skills/
├── fangqikiki-perspective/SKILL.md
├── leitanzhang-perspective/SKILL.md
├── huashu-perspective/SKILL.md
├── luo-yonghao-perspective/SKILL.md
├── zhangxiaolong-perspective/SKILL.md
├── nuwa-skill/SKILL.md
└── unified-search/SKILL.md + *.py 工具
```

### 迁移

当前 7 个内置 skills 已遵循 Agent Skills 标准格式（`SKILL.md`），零修改迁移。

```rust
struct PiSkillsManager {
    deploy_dir: PathBuf,  // ~/.pi/agent/skills/
}

impl PiSkillsManager {
    fn deploy_builtin_skills() -> Result<()>;  // include_str! 写入 ~/.pi/agent/skills/
    fn list_skills() -> Vec<SkillInfo>;         // 扫描部署目录
}
```

## Session Management

### 交互式会话

```rust
struct PiManager {
    sessions: HashMap<String, PiSession>,
    runtime: PiRuntime,
}

impl PiManager {
    fn create_session(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        let mut cmd = self.runtime.get_pi_command();
        cmd.env("ANTHROPIC_API_KEY", config.api_key);
        cmd.env("ANTHROPIC_MODEL", config.model);
        // skills 在 ~/.pi/agent/skills/ 自动发现，无需 --add-dir
    }
}
```

前端事件名从 `claude-pty-output-*` 改为 `pi-pty-output-*`。

### Persona Rewrite

```rust
fn create_persona_session(&self, prompt: &str, session_id: &str) -> Result<()> {
    let mut cmd = self.runtime.get_pi_command();
    cmd.env("ANTHROPIC_API_KEY", config.api_key);
    cmd.arg("-p").arg(prompt);
    // print mode 天然非交互：
    // - 不需要 --dangerously-skip-permissions
    // - 不需要模拟键盘输入接受权限警告
    // - 不需要 PID 轮询超时（PTY EOF 自然结束）
}
```

| 维度 | 当前（claude） | 替换后（pi -p） |
|------|------|------|
| 权限处理 | `--dangerously-skip-permissions` + 模拟按键 | print mode 天然非交互 |
| 超时控制 | 自建 PID 轮询线程（120s） | PTY EOF 自然结束 |
| Skills 注入 | 手动拼接 skill 内容到 prompt | pi 自动发现 |
| 代码量 | 约 340 行 | 约 50 行 |

## Configuration

OmniLink 只管 API Key 和 Provider 级别配置。pi 自身的精细配置由用户通过 pi 的 `/settings` 命令或 `~/.pi/agent/settings.json` 管理。

```rust
struct AgentConfig {
    provider: String,     // "anthropic" | "openai-compatible"
    api_key: String,
    base_url: Option<String>,
    model: String,
}
```

存储位置不变：`~/.omnilink/config.json`。

环境变量映射：
- Anthropic 模式：`ANTHROPIC_API_KEY` + `ANTHROPIC_MODEL`
- OpenAI Compatible 模式：`OPENAI_API_KEY` + `OPENAI_BASE_URL`

## Frontend Changes

- `ClaudeSettings.vue` → `AgentSettings.vue`（UI 结构不变，文案更新）
- `InteractiveTerminal.vue`：事件名 `claude-pty-output-*` → `pi-pty-output-*`，API 调用改为 pi commands
- `BottomPanel.vue`：状态文案更新
- `useClaude.ts` → `usePi.ts`：IPC 调用层重命名
- `types/persona.ts`：类型更新（`ClaudeConfig` → `AgentConfig` 等）

## Code Size Estimate

| 维度 | 当前 | 替换后 |
|------|------|------|
| Rust 后端文件 | 10 个 / ~2034 行 | 4 个 / ~600 行 |
| 前端文件 | 6 个 / ~825 行 | 6 个 / ~700 行 |
| Tauri Commands | 19 个 | 6 个 |
| 内置 Skills | 7 个 | 7 个（零修改） |
