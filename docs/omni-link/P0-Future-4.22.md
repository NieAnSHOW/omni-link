# P0-Future-4.22

### 笔记

#### 数组存储

sqlite 只做元数据的存储

笔记的内容直接存储为 markdown 的文本文件

#### 布局

分为左右

左侧单列展示笔记列表

右侧展示选中笔记详情

#### 技术方向

笔记详情markdown的渲染可直接使用 src/views/ContentView.vue 中的 markdown 编辑器，直接对 markdown 文本文件进行编辑保存
