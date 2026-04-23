# 人格系统设计文档

**日期：** 2026-04-23  
**版本：** v1.0  
**状态：** 待审阅

## 一、概述

为 OmniLink 添加"人格系统"功能，允许用户使用不同的"人格 skills"（如房琪 kiki、雷探长、花叔等）来重构笔记内容。核心思想是借用用户本地已安装的 Claude Code CLI 工具及其 skills，通过内嵌终端的方式实现自动化调用。

## 二、核心设计决策

### 2.1 CLI 调用方式

**选择：内嵌终端组件（xterm.js + PTY）**

- 在 OmniLink 界面中嵌入终端组件（xterm.js）
- Rust 后端创建 PTY（伪终端）启动用户本地的 `claude` 命令
- 调用的是用户本地的 Claude Code，包括其配置、API key 和所有已安装的 skills
- 用户体验最佳，无需切换窗口

### 2.2 人格选择界面

**选择：单一对话框 + Tab 切换**

- 点击"人格撰写"按钮后弹出对话框
- 对话框包含两个 Tab："智能选择"和"手动选择"
- 智能选择：AI 自动分析笔记内容并匹配合适的人格
- 手动选择：用户从列表中选择指定人格

### 2.3 终端交互流程

**选择：全自动模式**

- OmniLink 自动构造完整的命令并执行
- 终端为只读模式，用户只能观看执行过程
- 无需用户手动输入或确认
- 执行完成后自动刷新笔记内容

### 2.4 人格商店

**选择：独立的人格管理页面**

- 在导航菜单中添加"人格商店"入口
- 支持人格分类、搜索、详情查看
- 可以从本地 skills 目录导入人格
- 未来可扩展为在线商店

### 2.5 女娲定位

**选择：女娲作为人格生成器**

- 女娲不是一个可选的人格，而是用来生成新人格的工具
- 用户输入人名或主题，女娲自动调研并生成对应的 perspective skill
- 女娲不会出现在人格选择列表中
- 符合"女娲造人"的隐喻

## 三、实现方案

### 3.1 渐进式实现策略

采用三阶段渐进式实现，每个阶段都能交付可用功能：

**Phase 1（MVP，2-3 周）：基础人格撰写功能**
- 内嵌终端组件（xterm.js + PTY）
- 笔记详情中的"人格撰写"按钮
- 人格选择对话框（智能/手动 Tab 切换）
- 只支持手动选择模式
- 全自动执行流程

**Phase 2（1-2 周）：人格商店**
- 独立的人格管理页面
- 人格分类、搜索、详情查看
- 从本地 skills 目录导入人格

**Phase 3（2-3 周）：女娲造人**
- 在人格商店中添加"创建新人格"功能
- 调用女娲 skill 生成新人格
- 启用智能选择模式

**Why:** 渐进式实现风险可控，每个阶段都有明确的交付物，可以根据用户反馈调整优先级。

**How to apply:** 严格按照 Phase 顺序开发，不跳过阶段。每个 Phase 完成后进行用户测试和反馈收集。

## 四、系统架构

### 4.1 整体架构

系统分为三层：

**前端层（Vue 3）：**
- `PersonaDialog.vue` - 人格选择对话框组件
- `EmbeddedTerminal.vue` - 内嵌终端组件
- `PersonasView.vue` - 人格商店页面（Phase 2）
- `PersonaCreator.vue` - 人格创建对话框（Phase 3）

**Tauri IPC 层：**
- `persona_commands.rs` - 人格相关命令
- `terminal_commands.rs` - 终端管理命令
- `cli_commands.rs` - Claude Code CLI 调用命令

**Rust 后端层：**
- `terminal/pty_manager.rs` - PTY（伪终端）管理
- `terminal/session.rs` - 终端会话管理
- `persona/scanner.rs` - 扫描本地 skills 目录
- `persona/repository.rs` - 人格数据持久化
- `cli/executor.rs` - Claude Code 命令执行器

## 五、数据模型

### 5.1 Rust 后端模型

```rust
// 人格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: i64,
    pub name: String,           // 显示名称，如"房琪 kiki"
    pub skill_name: String,     // skill 名称，如"fangqikiki-perspective"
    pub category: String,       // 分类：技术类、旅游攻略、电影剧集
    pub description: String,    // 描述
    pub is_builtin: bool,       // 是否内置
    pub is_installed: bool,     // 是否已安装
    pub created_at: String,
}

// 终端会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,             // 会话 ID
    pub note_id: i64,           // 关联的笔记 ID
    pub persona_skill: String,  // 使用的人格 skill
    pub mode: String,           // "smart" 或 "manual"
    pub status: String,         // "running", "completed", "failed"
    pub created_at: String,
}
```

### 5.2 前端类型

```typescript
export interface Persona {
  id: number;
  name: string;
  skillName: string;
  category: string;
  description: string;
  isBuiltin: boolean;
  isInstalled: boolean;
  createdAt: string;
}

export interface TerminalSession {
  id: string;
  noteId: number;
  personaSkill: string;
  mode: 'smart' | 'manual';
  status: 'running' | 'completed' | 'failed';
  createdAt: string;
}
```

### 5.3 数据库表

```sql
-- 人格表
CREATE TABLE personas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    skill_name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    description TEXT,
    is_builtin INTEGER DEFAULT 0,
    is_installed INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
);

-- 终端会话表
CREATE TABLE terminal_sessions (
    id TEXT PRIMARY KEY,
    note_id INTEGER NOT NULL,
    persona_skill TEXT NOT NULL,
    mode TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (note_id) REFERENCES links(id)
);
```

## 六、Phase 1 核心功能流程

### 6.1 用户点击"人格撰写"按钮

1. 前端弹出 `PersonaDialog` 组件
2. 默认显示"智能选择" Tab（但暂时禁用，提示"Phase 2 开放"）
3. 切换到"手动选择" Tab
4. 调用 `invoke('scan_local_personas')` 扫描本地 skills

### 6.2 扫描本地人格

**Rust 后端逻辑：**

```rust
// src-tauri/src/persona/scanner.rs
pub fn scan_local_skills() -> Result<Vec<Persona>> {
    let home = dirs::home_dir().ok_or("无法获取 home 目录")?;
    let skills_dir = home.join(".claude/plugins/cache");
    
    let mut personas = Vec::new();
    
    // 遍历所有插件目录
    for entry in fs::read_dir(skills_dir)? {
        let plugin_dir = entry?.path();
        let skills_path = plugin_dir.join("skills");
        
        if !skills_path.exists() {
            continue;
        }
        
        // 查找 *-perspective.md 文件
        for skill_file in fs::read_dir(skills_path)? {
            let path = skill_file?.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            
            // 排除女娲
            if filename.ends_with("-perspective.md") && filename != "nuwa-skill.md" {
                if let Ok(persona) = parse_skill_file(&path) {
                    personas.push(persona);
                }
            }
        }
    }
    
    Ok(personas)
}
```

**Why:** 直接从用户本地 skills 目录读取，自动同步用户已安装的所有人格 skills，无需手动维护。

**How to apply:** 每次打开人格选择对话框时都重新扫描，确保列表是最新的。

### 6.3 用户选择人格并确认

1. 用户从列表中选择一个人格（如"房琪 kiki"）
2. 点击"开始重构"按钮
3. 前端调用 `invoke('start_persona_rewrite', { noteId, personaSkill, mode: 'manual' })`

### 6.4 启动终端会话

**Rust 后端逻辑：**

```rust
// src-tauri/src/terminal/pty_manager.rs
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
}

impl PtyManager {
    pub fn create_session(&mut self, note_path: &str, skill: &str) -> Result<String> {
        let pty_system = native_pty_system();
        
        // 创建 PTY
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            ..Default::default()
        })?;
        
        // 启动 shell
        let mut cmd = CommandBuilder::new("claude");
        let mut child = pair.slave.spawn_command(cmd)?;
        
        // 读取输出并发送到前端
        let reader = pair.master.try_clone_reader()?;
        self.spawn_output_reader(session_id, reader);
        
        // 自动输入命令
        self.write_command(&pair.master, &format!(
            "使用 {} 重构 {}，直接覆盖内容\n",
            skill, note_path
        ))?;
        
        Ok(session_id)
    }
    
    fn spawn_output_reader(&self, session_id: String, reader: Box<dyn Read + Send>) {
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        // 发送输出到前端
                        emit_event("pty-output", &buf[..n]);
                    }
                    _ => break,
                }
            }
        });
    }
}
```

**Why:** PTY 允许我们像真实终端一样与 Claude Code 交互，可以捕获所有输出并自动输入命令。

**How to apply:** 使用 `portable-pty` crate 实现跨平台的 PTY 支持（macOS、Windows、Linux）。

### 6.5 前端显示终端

**核心组件：**

```typescript
// src/components/terminal/EmbeddedTerminal.vue
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { listen } from '@tauri-apps/api/event';

const term = new Terminal({ 
  cursorBlink: false,  // 只读模式，不显示光标
  disableStdin: true   // 禁用输入
});

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

// 监听 PTY 输出
await listen('pty-output', (event) => {
  term.write(event.payload);
});

// 监听会话状态
await listen('session-status', (event) => {
  if (event.payload.status === 'completed') {
    // 刷新笔记内容
    await notesStore.fetchNoteDetail(noteId);
    // 关闭终端
    closeTerminal();
  }
});
```

**Why:** xterm.js 是成熟的终端模拟器，支持 ANSI 转义序列、颜色、光标控制等，能完整呈现 Claude Code 的输出。

**How to apply:** 终端组件以全屏或半屏模式显示，用户可以实时看到 Claude Code 的执行过程，但无法输入（只读模式）。

### 6.6 完成后刷新笔记

1. 后端监听 PTY 输出，检测完成标志（如返回提示符）
2. 更新会话状态为 "completed"
3. 发送 `session-status` 事件到前端
4. 前端自动刷新笔记内容
5. 关闭终端窗口
6. 显示成功提示

## 七、Phase 2 - 人格商店

### 7.1 新增页面

**路由配置：**
```typescript
{
  path: '/personas',
  name: 'Personas',
  component: () => import('../views/PersonasView.vue')
}
```

**导航菜单：**
在 `src/components/Navigation.vue` 中添加"人格商店"入口

### 7.2 核心功能

**人格列表展示：**
- 网格布局，每个人格显示为卡片
- 显示名称、分类、描述、安装状态
- 支持按分类筛选（全部、技术类、旅游攻略、电影剧集）

**人格详情：**
- 点击"查看详情"弹出对话框
- 显示完整描述、skill 名称、创建时间
- 提供"使用此人格"快捷入口

**人格管理：**
- 从本地 skills 目录导入
- 标记为"已安装"或"未安装"
- 卸载功能（仅从 OmniLink 数据库删除记录，不删除 skill 文件）

**搜索功能：**
- 按名称搜索人格
- 实时过滤结果

## 八、Phase 3 - 女娲造人

### 8.1 创建新人格入口

在人格商店页面添加"+ 创建新人格"按钮，点击后弹出 `PersonaCreator` 对话框

### 8.2 创建流程

**用户输入：**
- 人格名称（如"李子柒"）
- 人格类型（技术类、旅游攻略、电影剧集、其他）
- 可选：补充描述

**调用女娲：**
```rust
pub fn create_persona_with_nuwa(name: &str, category: &str) -> Result<String> {
    // 1. 创建 PTY 会话
    // 2. 启动 Claude Code
    // 3. 输入命令：使用 nuwa-skill，为 "{name}" 生成 perspective skill
    // 4. 等待生成完成（可能需要几分钟）
    // 5. 返回生成的 skill 文件路径
}
```

**保存到数据库：**
- 解析生成的 skill 文件
- 保存到 personas 表
- 标记为"已安装"

**显示结果：**
- 显示生成进度（可能需要几分钟）
- 完成后跳转到人格详情
- 提示用户可以立即使用

### 8.3 智能选择模式

在笔记详情的"人格撰写"对话框中，启用"智能选择" Tab：

1. 用户点击"开始智能重构"
2. 后端调用女娲 skill 分析笔记内容
3. 女娲返回推荐的人格 skill 名称
4. 自动使用该人格执行重构

**Why:** 智能选择模式依赖女娲的分析能力，因此必须在 Phase 3 实现。

**How to apply:** 智能选择的命令格式为：`阅读 {note_path}，根据内容从 {技能目录} 中选择合适的人格 skills，排除女娲技能，当 AI 选择到合适的技能时，返回技能的名称，并携带到下一条消息中：使用 xxx 技能，为 {note_path} 重构文本内容，直接覆盖内容`

## 九、技术依赖

### 9.1 前端依赖

```json
{
  "xterm": "^5.3.0",
  "xterm-addon-fit": "^0.8.0",
  "xterm-addon-web-links": "^0.9.0"
}
```

### 9.2 Rust 依赖

```toml
[dependencies]
portable-pty = "0.8"
tokio = { version = "1", features = ["full"] }
dirs = "5.0"
```

## 十、开发时间线

**Phase 1（2-3 周）：**
- Week 1: PTY 管理 + 终端组件
- Week 2: 人格扫描 + 选择对话框
- Week 3: 集成测试 + Bug 修复

**Phase 2（1-2 周）：**
- Week 1: 人格商店页面 + 数据持久化
- Week 2: 搜索、筛选、详情功能

**Phase 3（2-3 周）：**
- Week 1: 女娲调用逻辑
- Week 2: 创建人格流程
- Week 3: 智能选择模式

**总计：5-8 周**

## 十一、风险与挑战

### 11.1 技术风险

**PTY 跨平台兼容性：**
- Windows 的 PTY 支持与 Unix 系统有差异
- 缓解措施：使用 `portable-pty` crate，它已经处理了跨平台差异

**Claude Code CLI 版本兼容性：**
- 不同版本的 Claude Code 可能有不同的命令格式
- 缓解措施：在文档中明确支持的 Claude Code 版本范围

**终端输出解析：**
- 需要检测 Claude Code 的完成标志
- 缓解措施：使用多种检测方式（提示符、特定输出、超时）

### 11.2 用户体验风险

**生成时间过长：**
- 女娲生成人格可能需要几分钟
- 缓解措施：显示详细的进度提示，允许用户取消

**错误处理：**
- Claude Code 执行失败时需要友好的错误提示
- 缓解措施：捕获常见错误（如 API key 未配置），提供解决方案

## 十二、未来扩展

### 12.1 在线人格商店

- 提供官方维护的人格库
- 用户可以一键下载和安装
- 支持人格评分和评论

### 12.2 人格自定义

- 允许用户编辑人格的 prompt
- 支持创建私有人格
- 人格版本管理

### 12.3 批量重构

- 选择多篇笔记，使用同一人格批量重构
- 后台队列处理
- 进度追踪

## 十三、总结

本设计采用渐进式实现策略，通过三个阶段逐步交付完整的人格系统功能。Phase 1 专注于核心的人格撰写功能，使用内嵌终端调用本地 Claude Code CLI，实现自动化的笔记重构。Phase 2 添加人格商店，提供完整的人格管理功能。Phase 3 引入女娲造人，允许用户自定义生成任意人格。

整体架构清晰，技术方案可行，风险可控。预计 5-8 周完成全部三个阶段的开发。
