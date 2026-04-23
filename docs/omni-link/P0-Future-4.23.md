# P0-Future-4.23

### 人格系统

通过“蒸馏”而来的人物 skills ，例如：技术向的内容交由完“huashu-perspective” skills 完成、旅游向交由 “XXXX”skills  完成

#### 人格商店

内置 “房琪 kiki”、“雷探长”、“花叔” 、“女娲” 

“女娲” skill 作为核心技能，不可被直接调用

#### 人格类型

技术类、旅游攻略、电影剧集类

#### 与笔记的功能集合

src/components/notes/NoteDetail.vue 笔记详情中预留了 “人格撰写” 的操作按钮，点击时弹出选择窗口“智能”“手动选择”

智能：交由AI选择与文本内容相匹配的人格

手动选择：由用户自己选择指定的人格

#### 如何使用人格？

核心思想是借用用户端本地已存在的就 AI cli工具，例如：claude code、Gemini Cli、openCode、codex，拉起cli 后，当用户选择了“智能”，发送消息：阅读 XXXX.md，根据内容从 {技能目录} 中选择合适的人格 skills，排除 “女娲” 技能，当 AI 选择到合适的技能时，返回技能的名称，并携带到下一条消息中：使用 xxx 技能，为 xxx.md 重构文本内容，直接覆盖内容；

当前 MVP 版本优先适配 claude code

