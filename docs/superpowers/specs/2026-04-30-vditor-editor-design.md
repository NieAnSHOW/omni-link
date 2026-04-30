# Vditor 替换 md-editor-v3 设计

## 目标

将 `NoteDetail.vue` 中的 `md-editor-v3` 替换为 `Vditor`，实现 Typora 风格的即时渲染（IR）编辑体验，同时支持项目已有的亮色/暗色主题切换。

## 背景

当前 `NoteDetail.vue` 使用 `md-editor-v3` v6.4.2，以分屏模式工作（左边 Markdown 源码，右边实时预览）。用户希望获得更沉浸的编辑体验——输入 Markdown 语法时即时渲染为富文本，语法符号在渲染后隐藏，类似 Typora。

经过对比，`md-editor-v3` 没有真正的 IR 模式；Milkdown 的 IR 模式仍不稳定且过于重量级。Vditor 将 IR 模式作为核心功能，且社区成熟（b3log/siyuan 笔记本采用），是最佳选择。

## 设计

### 组件架构

新建 `src/components/notes/VditorEditor.vue` wrapper 组件，封装 Vditor 实例，对外暴露与当前 MdEditor 一致的 v-model 接口。

```
src/components/notes/
├── NoteDetail.vue      (修改：用 VditorEditor 替换 MdEditor)
└── VditorEditor.vue    (新建：wrapper 组件)
```

**VditorEditor.vue 职责：**

- `props: modelValue: string` + `emit('update:modelValue')` — 保持 v-model 语义
- `props: theme: 'light' | 'dark'` — 响应主题切换
- `onMounted`：调用 `new Vditor()` 初始化实例，mode 设为 `'ir'`
- `onUnmounted`：调用 `vditor.destroy()` 清理实例
- `watch(modelValue)`：外部内容更新（如人格重写刷新）同步到编辑器，通过内部标记避免循环触发
- `watch(theme)`：调用 `vditor.setTheme()` 同步暗色模式

### 暗色模式集成

Vditor 通过 CSS class `.vditor--dark` 启用暗色主题，通过实例方法 `setTheme('dark' | 'classic')` 切换。

`VditorEditor.vue` 内部监听 `theme` prop 变化，调用 `vditor.setTheme()` 同步切换。项目已有的 `useTheme()` composable 在 `<html>` 上切换 `.dark` class，Vditor 的 CSS 变量会自动继承，无需额外 CSS hack。

### NoteDetail.vue 集成

**变更点：**

| 位置 | 变化 |
|------|------|
| import | `MdEditor` + 样式 → `VditorEditor` |
| template | `<MdEditor v-model :language>` → `<VditorEditor v-model :theme>` |
| 新增 | `useTheme()` 获取当前主题传给 VditorEditor |
| 删除 | `import 'md-editor-v3/lib/style.css'` |
| 保存逻辑 | 不变，`handleContentChange` → 2 秒防抖 → `handleSave()` 照旧 |

外部内容更新（`handleRewriteCompleted` 通过直接修改 `localContent.value`）仍由 VditorEditor 内部的 `watch(modelValue)` 响应。

### 依赖变化

- 移除：`md-editor-v3`
- 添加：`vditor`
