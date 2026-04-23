# 人格系统前端适配设计

> 基于 UI 重构（Tailwind v4 + shadcn-vue）后对原人格系统实施计划的调整。后端（Rust）部分保持原计划不变。

## 背景

原计划 `docs/superpowers/plans/2026-04-23-persona-system.md` 编写于 UI 重构之前，前端组件使用纯 scoped CSS。项目已完成向 Tailwind CSS v4 + shadcn-vue 的迁移，因此前端组件的实现方式需要全面调整。

## 设计决策

| 决策项 | 选择 | 原因 |
|---|---|---|
| 核心方案 | PTY + Claude CLI | 保留原计划，可交互、可观察 |
| 人格选择形式 | shadcn Dialog 对话框 | 流畅、不遮挡上下文 |
| 终端展示方式 | 内嵌可折叠面板 | 不遮挡笔记内容，类似 VS Code 终端 |
| 智能选择 Tab | 保留占位 | Phase 3 功能，占位给用户预期 |

## 与原计划的差异

| 项目 | 原计划 | 新方案 |
|---|---|---|
| PersonaDialog 样式 | 纯 scoped CSS 自定义 overlay | shadcn Dialog + Tailwind |
| Terminal 展示 | 全屏覆盖层（fixed overlay） | 内嵌可折叠面板（flow layout） |
| TerminalPanel 组件名 | `EmbeddedTerminal.vue` | `TerminalPanel.vue` |
| 组件结构 | 两个独立 overlay 组件 | Dialog + 内嵌面板，不互相遮挡 |
| shadcn 依赖 | 无 | 需安装 Tabs 组件 |
| 前端依赖 | xterm + xterm-addon-fit | 不变 |

## 组件设计

### PersonaDialog.vue

**职责：** 人格选择对话框，包含手动选择和智能选择两个 Tab。

**使用的 shadcn 组件：**
- `Dialog` / `DialogContent` / `DialogHeader` / `DialogTitle` / `DialogFooter` — 对话框容器
- `Tabs` / `TabsList` / `TabsTrigger` / `TabsContent` — Tab 切换（需新增安装）
- `Card` — 人格卡片（选中态用 Tailwind `ring-2 ring-primary`）
- `Badge` variant="secondary" — "内置"标签
- `Button` default + outline — 底部操作按钮

**布局：**
- 最大宽度 560px，双列网格展示人格卡片
- 手动选择 Tab：人格卡片列表 + 右下角"自定义人格"虚线占位
- 智能选择 Tab：居中占位提示（🚧），底部按钮 disabled

**状态：**
- `personas: Ref<Persona[]>` — 扫描到的人格列表
- `selectedPersona: Ref<Persona | null>` — 当前选中的人格
- `loading: Ref<boolean>` — 扫描中状态

**事件：**
- `@close` — 关闭对话框
- `@confirm(persona, mode)` — 确认选择

### TerminalPanel.vue

**职责：** 可折叠内嵌终端面板，嵌入在 NoteDetail 页面笔记内容下方。

**布局：**
- 折叠状态：一行深色状态条（人格名 + 状态指示灯 + 展开/关闭按钮）
- 展开状态：深色顶栏 + xterm.js 终端输出区域
- 面板通过 `v-if` / `v-show` 控制显隐，嵌入正常文档流

**状态流转：**
- 执行中：黄色脉冲圆点 + 滚动输出
- 已完成：绿色静态圆点 + "查看结果"按钮 → 刷新笔记内容
- 失败：红色静态圆点 + 错误信息 + "重试"按钮

**使用的 shadcn 组件：**
- `Button` variant="ghost" size="sm" — 收起/关闭按钮
- `Badge` — 状态标签

**Props：**
- `sessionId: string`
- `noteId: number`
- `personaName: string`

**事件：**
- `@close` — 关闭面板
- `@completed` — 执行完成
- `@failed(error)` — 执行失败

### NoteDetail.vue 集成

**变更：**
- 启用已有的 disabled "人格撰写"按钮，绑定 `@click`
- 添加 `showPersonaDialog` / `showTerminal` 状态控制
- 在笔记内容下方放置 `<TerminalPanel>`
- 在模板末尾放置 `<PersonaDialog>`

## 数据流

```
用户点击"人格撰写"
  → PersonaDialog 打开
  → 调用 scanLocalPersonas() 加载人格列表
  → 用户选择人格，点击"开始重构"
  → Dialog 关闭
  → 调用 startPersonaRewrite() 启动 PTY 会话
  → TerminalPanel 在笔记下方展开
  → 监听 pty-output 事件，xterm.js 实时渲染
  → 执行完成 → 自动刷新笔记内容
  → 执行失败 → 显示错误 + 重试按钮
```

## 新建文件清单

```
src/components/persona/PersonaDialog.vue   — 人格选择对话框
src/components/persona/TerminalPanel.vue   — 内嵌终端面板
src/types/persona.ts                       — 类型定义
src/composables/usePersona.ts              — API 封装
```

## 修改文件清单

```
src/components/notes/NoteDetail.vue        — 集成人格功能
```

## 新增依赖

**npm：**
- `@radix-vue/radix-vue` 中的 Tabs 原语（通过 shadcn CLI 安装 `tabs` 组件）

**需安装的 shadcn 组件：**
- `tabs`

## 后端

所有 Rust 后端代码保持原计划不变：
- 数据库 schema（personas、terminal_sessions 表）
- 数据模型（Persona、TerminalSession）
- 内置 skills（房琪 kiki、雷探长、花叔、女娲）
- 人格扫描器、数据仓库、PTY 管理器
- Tauri 命令（scan_local_personas、start_persona_rewrite 等）
