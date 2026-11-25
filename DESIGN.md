# Aedt 设计基线

Aedt 桌面应用的设计约定。改动界面时先读这份，再动 `src/ui`。

## 平台

- 本轮交付 Desktop：macOS 用 `cargo run` 或 `nsh run --target macos`，GPUI 后端用 `nsh run --target gpui`。
- Desktop 窗口隐藏系统标题栏（`TitleBarStyle::Hidden`），标题栏高度用平台默认值；红绿灯等系统控件保留，侧栏顶部用 `title_bar().top` 预留，聊天 header 右侧用 `title_bar().right` 预留 Windows/Linux 的窗口按钮。
- 移动端、Web、TUI 未纳入本轮；布局按桌面密度（32pt 控件、36pt 列表行）搭建，侧栏在窄窗口下应改为可折叠，属于后续工作。

## 颜色与尺度

- 颜色全部来自 `nsh_base::theme()`，只用语义角色：`background`、`foreground`、`card`、`sidebar*`、`muted*`、`accent`、`input`、`ring`、`border`、`primary*`。
- 主题在 `theme.json`（`nsh.json` 的 `theme`），明/暗两套：`primary` 浅蓝 `#DBEAFE` / `#93C5FD`，气泡文字用中性 `#171717`；`secondary`、`accent` 保持中性灰，侧栏不染蓝；`ring` 与 `link` 用可读的蓝色。Markdown 链接用独立的 `link` 深蓝，避免浅蓝链接在正文里不可读。`cargo run` 不读 `nsh.json`，根组件启动时用 `tokens_of` 应用同一份主题；`nsh run` 由 host 读取。原始 hex 只写在主题文件里。
- 圆角来自 `theme().radius` 与 Tailwind 的 `rounded-*` 阶梯：浮起的 composer 用 `rounded_3xl`，胶囊（badge、avatar）与圆形发送键用 `rounded_full()`。
- 尺寸是 app token，集中在 `src/ui/sizes.rs`：侧栏 264、控件 32、列表行 36、头像 28、输入框最小 44、会话列最宽 760（列内左右 24）、圆形发送 28、用户消息最大宽 560。
- 间距走 Tailwind 的 4pt 刻度（`p_3` = 12pt、`gap_2` = 8pt、`p_6` = 24pt）。
- 字号只用 `text_xs`（12）与 `text_sm`（14）；消息正文用 `text_view` 的 `text_size(14)`。
- 编程字体（代码块、行内代码、`font-mono`）默认 `Berkeley Mono, SF Mono, Menlo, monospace`：CoreText 取第一个已安装的族，缺 Berkeley Mono 的机器落到 SF Mono / Menlo，不会退回比例字体。`cargo run` 不读 `nsh.json`，根组件启动时应用主题并设置 `font-mono` token。

## 组件层

功能代码只用 `src/ui` 与布局原语（`view`、`text`、`virtual_list`、`text_view`）：

- `button`：`secondary` 标签按钮与 `icon` 圆形图标按钮，统一的按压反馈、焦点环与禁用态；圆形按钮 28pt，带 6pt 热区。
- `input`：受控多行输入，无边框；焦点由外层 composer 卡片的 `focus_within` 边框承接。
- `list_item::session`：侧栏会话行，selected 由调用方提供。
- `badge`、`avatar`、`separator`、`empty`。

## 布局

- 一级结构：`sidebar`（264pt，`sidebar` 底色，右边框）+ 会话面板（`flex_1`）。
- 内容脊柱：侧栏外边距 12pt、行内再留 12pt，会话标题文字落在 24pt；会话面板 header 用 24pt 内边距。聊天 header 顶部留 16pt；侧栏品牌在系统控制区（`title_bar().top`）下方再留 4pt，红绿灯下方的可见留白与左侧 12pt 内边距接近。品牌与 header 标 `window_drag(true)`，两栏背景各顶到窗口上边。
- 消息柱：消息流与 composer 放在同一个居中列里（最宽 760、列内左右 24）；列内消息内容再左右各缩 12pt，和输入框文字对齐。两根柱到卡片左右边的距离都是 12pt，AI 文本与用户气泡都不越过它们。
- 消息流、会话列表、输入框与代码块隐藏滚动条指示（`scrollbar(false)`），滚轮、触控板和键盘仍可滚动。
- 会话面板：header（标题 + 模型 badge）、`virtual_list` 消息流（底部对齐、垂直 24pt、水平 12pt、16pt 行距；助手消息不带头像与昵称，tokens/时间元信息在正文下方左对齐；用户气泡右对齐到右柱）。
- composer 放在同一列内：卡片顶边直接接消息视口，滚动内容在卡片上沿截断，中间不再留空白；卡片填满列宽，`rounded_3xl`、1pt `border`、`shadow_sm`、`card` 底色；卡片内 12pt 内边距，上面是最小 44pt 的输入框，下面是右对齐的 28pt 圆形发送键。卡片下方 8pt 后保留 20pt 状态栏高度（暂空），再留 8pt 到窗口底边。
- 会话切换用 `dynamic` 按会话 id 重建面板，草稿与滚动位置随之重置；消息与侧栏由 store 的 signal 驱动。

## 动效清单

| 变化 | 工具与 token | 减少动态 |
| --- | --- | --- |
| 按钮 hover | 颜色过渡，`transition_all` + `FAST`(180ms) ease-out | 时长归零 |
| 按钮按下 | 标签按钮缩放 0.97、圆形图标按钮 0.95，同为 `FAST` | 不缩放，颜色立即变化 |
| 会话行 hover / 选中 | 背景 wash，`transition_colors` + `INSTANT`(120ms) | 时长归零 |
| composer 卡片获得焦点 | `focus_within` 边框转 `ring`，颜色过渡 `FAST` | 时长归零 |
| 切换会话 | 面板淡入，`motion().entering(fade)`，`BASE`(240ms) ease-out | 直接显示终态 |
| 发送后的新消息 | 无动效：Mock 回复同步插入，列表保持阅读位置；接入流式模型后补光标呼吸与行进入动画 | — |
| 窗口缩放 | 无动效：原生布局 | — |

每条“无动效”都写明了理由；其余在运行中的应用里逐条核对。

## 尚未做

- 会话重命名、删除、搜索。
- 真实模型接入、流式回复、停止生成。
- 键盘快捷键（⌘↵ 发送）与移动端布局。
