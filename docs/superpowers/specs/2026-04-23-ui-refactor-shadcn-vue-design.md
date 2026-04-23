# UI 重构设计文档：迁移到 shadcn-vue + TailwindCSS

**日期：** 2026-04-23  
**状态：** 设计阶段  
**范围：** 全量 UI 重构

## 概述

将 OmniLink 的 UI 系统从纯 scoped CSS 迁移到 shadcn-vue + TailwindCSS，采用一次性迁移策略，使用 shadcn-vue 默认主题。

## 目标

- 统一的设计系统和组件库
- 更好的可维护性和开发体验
- 现代化的 UI 工具链
- 按需引入组件，控制包体积

## 技术栈变更

### 移除
- 所有组件的 `<style scoped>` 块
- 自定义样式系统
- 当前主色调 `#6366f1`（改用 shadcn-vue 默认主题）

### 新增
- **TailwindCSS 4.x** - 工具类 CSS 框架
- **shadcn-vue** - 基于 Radix Vue 的组件库
- **class-variance-authority (CVA)** - 组件变体管理
- **clsx + tailwind-merge** - 类名合并工具

## 架构设计

### 项目结构调整

```
src/
├── components/
│   ├── ui/                    # shadcn-vue 组件（自动生成）
│   │   ├── button.vue
│   │   ├── card.vue
│   │   ├── dialog.vue
│   │   ├── input.vue
│   │   ├── badge.vue
│   │   ├── skeleton.vue
│   │   ├── toast.vue
│   │   ├── dropdown-menu.vue
│   │   ├── tabs.vue
│   │   └── textarea.vue
│   ├── LinkCard.vue           # 业务组件（使用 ui/ 组件重写）
│   ├── AddLinkDialog.vue
│   ├── TagInput.vue
│   ├── ContentEditor.vue
│   ├── MarkdownEditor.vue
│   ├── LinkCardSkeleton.vue
│   ├── Toast.vue
│   ├── AppLayout.vue
│   ├── ScrollToTop.vue
│   ├── StickyHeader.vue
│   ├── NotesList.vue
│   └── NoteDetail.vue
├── lib/
│   └── utils.ts               # shadcn-vue 工具函数（cn 等）
├── assets/
│   └── index.css              # Tailwind 入口文件
└── views/
    ├── LinksView.vue
    ├── ContentView.vue
    ├── TagsView.vue
    ├── SettingsView.vue
    └── NotesView.vue
```

### 新增配置文件

- `tailwind.config.js` - TailwindCSS 配置
- `components.json` - shadcn-vue 配置
- `postcss.config.js` - PostCSS 配置

## 组件迁移映射

### shadcn-vue 组件清单

按需安装以下组件：

```bash
npx shadcn-vue@latest add button card badge input dialog skeleton toast dropdown-menu tabs textarea
```

### 组件对应关系

| 现有组件 | 迁移方案 | shadcn-vue 组件 |
|---------|---------|----------------|
| `LinkCard.vue` | 重写 | `Card` + `Badge` |
| `AddLinkDialog.vue` | 重写 | `Dialog` + `Input` + `Button` |
| `Toast.vue` | 替换 | `Toast` |
| `TagInput.vue` | 重写 | `Input` + `Badge` |
| `LinkCardSkeleton.vue` | 重写 | `Skeleton` |
| `ContentEditor.vue` | 重写 | `Textarea` + `Button` |
| `MarkdownEditor.vue` | 保持 md-editor-v3，外层用 Tailwind | - |
| `AppLayout.vue` | 纯 Tailwind 布局类 | - |
| `ScrollToTop.vue` | 重写 | `Button` + Tailwind 定位 |
| `StickyHeader.vue` | 纯 Tailwind 粘性定位 | - |
| `NotesList.vue` | 重写 | `Card` |
| `NoteDetail.vue` | 重写 | `Card` + `Button` |

### 视图层迁移

所有视图（LinksView、ContentView、TagsView、SettingsView、NotesView）的样式全部改用 Tailwind 工具类，移除所有 `<style scoped>` 块。

## 实施流程

### 阶段 1：环境准备

1. 安装 TailwindCSS 及相关依赖
   ```bash
   npm install -D tailwindcss postcss autoprefixer
   npx tailwindcss init -p
   ```

2. 运行 shadcn-vue 初始化
   ```bash
   npx shadcn-vue@latest init
   ```
   - 选择 Vue 3 + TypeScript
   - 选择默认主题
   - 配置组件路径为 `src/components/ui`

3. 配置 Vite 支持 PostCSS（通常自动配置）

4. 创建 `src/assets/index.css`
   ```css
   @tailwind base;
   @tailwind components;
   @tailwind utilities;
   ```

5. 在 `src/main.ts` 中导入样式
   ```typescript
   import './assets/index.css'
   ```

### 阶段 2：shadcn-vue 组件安装

按需安装所有需要的组件：

```bash
npx shadcn-vue@latest add button
npx shadcn-vue@latest add card
npx shadcn-vue@latest add badge
npx shadcn-vue@latest add input
npx shadcn-vue@latest add dialog
npx shadcn-vue@latest add skeleton
npx shadcn-vue@latest add toast
npx shadcn-vue@latest add dropdown-menu
npx shadcn-vue@latest add tabs
npx shadcn-vue@latest add textarea
```

或一次性安装：
```bash
npx shadcn-vue@latest add button card badge input dialog skeleton toast dropdown-menu tabs textarea
```

### 阶段 3：组件迁移（一次性完成）

按以下顺序重写所有组件：

**第一批：基础 UI 组件**
- `Toast.vue` - 使用 shadcn-vue Toast 替换
- `LinkCardSkeleton.vue` - 使用 Skeleton 重写
- `ScrollToTop.vue` - 使用 Button + Tailwind 定位

**第二批：表单和交互组件**
- `TagInput.vue` - 使用 Input + Badge 重写
- `AddLinkDialog.vue` - 使用 Dialog + Input + Button 重写
- `ContentEditor.vue` - 使用 Textarea + Button 重写

**第三批：业务组件**
- `LinkCard.vue` - 使用 Card + Badge 重写
- `NotesList.vue` - 使用 Card 重写
- `NoteDetail.vue` - 使用 Card + Button 重写

**第四批：布局组件**
- `StickyHeader.vue` - 纯 Tailwind 粘性定位
- `AppLayout.vue` - 纯 Tailwind 布局类

**第五批：视图层**
- `LinksView.vue` - 移除 scoped CSS，改用 Tailwind
- `ContentView.vue` - 移除 scoped CSS，改用 Tailwind
- `TagsView.vue` - 移除 scoped CSS，改用 Tailwind
- `SettingsView.vue` - 移除 scoped CSS，改用 Tailwind
- `NotesView.vue` - 移除 scoped CSS，改用 Tailwind

**第六批：根组件**
- `App.vue` - 移除 splash screen 的 scoped CSS，改用 Tailwind

### 阶段 4：清理和验证

1. 删除所有组件中的 `<style scoped>` 块
2. 验证所有页面功能正常
3. 检查响应式布局
4. 测试所有交互功能（点击、输入、弹窗、导航等）
5. 验证 Tauri IPC 调用不受影响

### 阶段 5：优化

1. 配置 Tailwind 的 content 选项优化打包体积
   ```javascript
   // tailwind.config.js
   module.exports = {
     content: [
       './index.html',
       './src/**/*.{vue,js,ts,jsx,tsx}',
     ],
     // ...
   }
   ```

2. 检查是否有未使用的 shadcn-vue 组件
3. 确认所有 Tailwind 类名正确应用
4. 验证打包体积在预期范围内（预期增加 50-80KB gzipped）

## 技术细节

### TailwindCSS 配置要点

**tailwind.config.js 关键配置：**
- `content` 路径需包含：`./index.html`, `./src/**/*.{vue,js,ts,jsx,tsx}`
- 保持 shadcn-vue init 生成的主题配置（使用默认主题）
- 可选：启用 dark mode

### 组件编写规范

**使用 cn 工具函数合并类名：**
```typescript
import { cn } from '@/lib/utils'

// 示例
<div :class="cn('base-classes', props.className)">
```

**移除所有内联样式：**
- 不再使用 `style="..."`
- 全部改用 Tailwind 工具类

**响应式设计：**
- 使用 Tailwind 响应式前缀：`sm:`, `md:`, `lg:`, `xl:`
- 当前的瀑布流布局（masonry）改用 Tailwind 的 `columns-*` 工具类

### 兼容性处理

**Tauri 特殊处理：**
- App.vue 的 splash screen 动画改用 Tailwind 的 `transition-*` 和 `animate-*`
- 确保 Tailwind 的 JIT 模式在 Tauri 开发环境正常工作

**第三方组件：**
- `md-editor-v3`（MarkdownEditor）保持不变，只调整外层容器样式
- `highlight.js` 样式不受影响
- 如有样式冲突，使用 Tailwind 的 `@layer components` 隔离

### 类型支持

shadcn-vue 组件已包含完整 TypeScript 类型定义，无需额外配置。

### 性能考虑

**打包优化：**
- TailwindCSS 会自动 purge 未使用的样式
- shadcn-vue 组件按需引入，不会增加太多体积
- 预期打包体积增加约 50-80KB（gzipped）

**开发体验：**
- Vite HMR 支持 Tailwind，修改类名即时生效
- 无需重启开发服务器

## 风险和应对措施

### 主要风险

**1. 视觉一致性风险**
- **风险：** 采用 shadcn-vue 默认主题后，UI 风格会与当前版本有明显差异（主色调从 `#6366f1` 变为 shadcn-vue 默认色）
- **影响：** 用户可能需要适应新的视觉风格
- **应对：** 迁移完成后进行全面的视觉验证，确保新风格符合预期

**2. 功能回归风险**
- **风险：** 重写所有组件可能引入功能缺失或行为变化
- **影响：** 可能导致某些交互失效或行为异常
- **应对：**
  - 迁移时对照原组件逐一验证功能点
  - 重点测试交互逻辑（点击、输入、弹窗等）
  - 验证 Tauri IPC 调用不受影响
  - 测试所有路由跳转和状态管理

**3. 第三方组件兼容性**
- **风险：** md-editor-v3 可能与 Tailwind 样式冲突
- **影响：** Markdown 编辑器显示异常
- **应对：** 使用 Tailwind 的 `@layer components` 隔离第三方组件样式

**4. 开发周期风险**
- **风险：** 一次性迁移工作量大，可能影响其他功能开发
- **影响：** 延迟其他功能的开发进度
- **应对：**
  - 在独立分支进行迁移
  - 迁移期间暂停其他 UI 相关开发
  - 预留充足的测试时间

### 回滚方案

如果迁移后发现重大问题：
1. Git 回退到迁移前的 commit
2. 评估问题是否可以快速修复
3. 如无法修复，暂时使用旧版本，重新规划迁移方案

### 验收标准

迁移完成的标准：
- ✅ 所有页面功能正常运行
- ✅ 所有交互逻辑与原版一致
- ✅ 响应式布局在不同屏幕尺寸下正常
- ✅ 无 console 错误或警告
- ✅ 代码中无残留的 scoped CSS
- ✅ 打包构建成功，体积在预期范围内（增加 50-80KB gzipped）
- ✅ Tauri 应用正常启动和运行
- ✅ 所有 Tauri IPC 调用正常工作

## 迁移策略总结

- **范围：** 全量重构
- **组件引入：** 按需引入
- **迁移方式：** 一次性迁移
- **主题：** shadcn-vue 默认主题
- **实施方案：** 标准 shadcn-vue 集成（方案 A）

## 后续工作

迁移完成后，下一步将进入实现计划编写阶段，详细规划每个组件的具体实现步骤。
