# Aedt 设计基线

Aedt 桌面应用的设计约定。改动界面时先读这份，再动 `src/ui`。

## 平台

- 本轮交付 Desktop：macOS 用 `cargo run` 或 `nsh run --target macos`，GPUI 后端用 `nsh run --target gpui`。
- Desktop 窗口隐藏系统标题栏（`TitleBarStyle::Hidden`），标题栏高度用平台默认值；红绿灯等系统控件保留，侧栏顶部用 `title_bar().top` 预留，聊天 header 右侧用 `title_bar().right` 预留 Windows/Linux 的窗口按钮。
- 移动端、Web、TUI 未纳入本轮；布局按桌面密度（32pt 控件、36pt 列表行）搭建，侧栏在窄窗口下应改为可折叠，属于后续工作。

## 颜色与尺度

- 颜色全部来自 `nsh_base::theme()`，只用语义角色：`background`、`foreground`、`card`、`sidebar*`、`muted*`、`accent`、`input`、`ring`、`border`、`primary*`。
- 主题在 `theme.json`（`nsh.json` 的 `theme`），明/暗两套：`primary` 浅蓝 `#DBEAFE` / `#93C5FD`；`foreground` 用 `#242529dd`（87% 不透明度的深灰，不用纯黑），聊天正文、气泡文字、会话行标题共用它；用户气泡取 `bubble`（明色 `#EFF6FF`，与 `muted` 同为 L 0.97）；消息元信息行取 `meta`（明色 `#8C8C8C`，比 `muted-foreground` 的 `oklch(0.556 0 0)` 浅一档）；`secondary`、`accent` 保持中性灰，侧栏不染蓝；`ring` 与 `link` 用可读的蓝色。Markdown 链接用独立的 `link` 深蓝，避免浅蓝链接在正文里不可读。`cargo run` 不读 `nsh.json`，根组件启动时用 `tokens_of` 应用同一份主题；`nsh run` 由 host 读取。原始 hex 只写在主题文件里。
- 圆角来自 `theme().radius` 与 Tailwind 的 `rounded-*` 阶梯：浮起的 composer 用 `rounded_3xl`，胶囊（badge、avatar）与圆形发送键用 `rounded_full()`。
- 尺寸是 app token，集中在 `src/ui/sizes.rs`：侧栏 264、控件 32、列表行 32、列表行内 10、头像 28、输入框最小 44、会话列最宽 760（列内左右 24）、圆形发送 28、用户消息最大宽 560。
- 间距走 Tailwind 的 4pt 刻度（`p_3` = 12pt、`gap_2` = 8pt、`p_6` = 24pt）。
- 字号只用 `text_xs`（12）与 `text_sm`（14）；消息正文用 `text_view` 的 `text_size(14)`，行距 `LineHeight::Multiple(1.25)`（`ui::MESSAGE_LINE`，气泡、助手回复与代码块统一）。
- 编程字体（代码块、行内代码、`font-mono`）默认 `Berkeley Mono, SF Mono, Menlo, monospace`：CoreText 取第一个已安装的族，缺 Berkeley Mono 的机器落到 SF Mono / Menlo，不会退回比例字体。`cargo run` 不读 `nsh.json`，根组件启动时应用主题并设置 `font-mono` token。

## 组件层

功能代码只用 `src/ui` 与布局原语（`view`、`text`、`virtual_list`、`text_view`）：

- `button`：`secondary` 标签按钮与 `icon` 圆形图标按钮，统一的按压反馈、焦点环与禁用态；圆形按钮 28pt，带 6pt 热区。
- `input`：受控多行输入，无边框；回车发送、Shift+回车与 Ctrl+回车换行（`submit_behavior(SubmitBehavior::Submit)`，macOS 下 IME 回车上字不会被当成发送）；发送后输入框保留焦点；卡片只是它的外框，聚焦时不换边框色（光标本身就是焦点提示，光标色不指定，用系统默认）。
- `list_item::session`：侧栏会话行，selected 由调用方提供；标题在选中与未选中都用 `foreground`，选中只靠 `accent` 底色表达；NSH 的 `hover:` 排在 `selected:` 之后（`rank::HOVER` 34 > `rank::SELECTED` 32），所以选中行的 hover 底色直接取 `accent`（与选中同色），悬停时不换色。
- `badge`、`avatar`、`separator`、`empty`。
- `icon`：内联 SVG 图标（`copy` 照 Lucide 的 `copy` 几何画，24 单位 viewBox、`stroke="currentColor"`，所以颜色跟调用处的文字色）；用 `svg(ImageSrc::svg_markup(..))`，靠 nsh-core 的 `svg` feature 渲成矢量。

## 布局

- 一级结构：`sidebar`（264pt，`sidebar` 底色，右边框）+ 会话面板（`flex_1`）。
- 内容脊柱：侧栏外边距 12pt、行内再留 10pt，会话标题文字落在 22pt；会话面板 header 用 24pt 内边距。聊天 header 顶部留 16pt；侧栏品牌在系统控制区（`title_bar().top`）正下方开始，不再额外加内边距，红绿灯下方的可见留白与左侧 12pt 内边距接近。品牌与 header 标 `window_drag(true)`，两栏背景各顶到窗口上边。
- 消息柱：消息流与 composer 放在同一个居中列里（最宽 760、列内左右 24）；列内消息内容再左右各缩 12pt，和输入框文字对齐。两根柱到卡片左右边的距离都是 12pt，AI 文本与用户气泡都不越过它们。
- 右侧消息导航：`sizes::RAIL` 32pt 宽的竖列（右侧再留 `RAIL_INSET` 14pt，不贴窗口边），它只与消息区同高（与输入框同一行时不含输入框高度，所以居中于消息区）；每条用户消息一个 2pt 高的短划线（AI 消息没有划；静止时 `border` 色、10pt 长；行上下各 4pt 内边距、行间 0，所以相邻两划中心相距 10pt，且整列都是 hover 热区（滑过时不会掉到空隙里）；指针压上去的那根变长到 18pt、取 `primary_foreground`（气泡文字同色，120ms 过渡），光标变手（`cursor_pointer`）；列表里“当前”的那根也一样长而黑——当前取“可见行中的最后一条用户消息”，由 `on_viewable_items_change` 算（`store::last_user_index`），滚动时跟着走，切会话时初始化为该会话最后一条用户消息；两炷都亮时以 hover 为先；指针那根还会在左侧浮出预览卡（320pt、`card` 底 + 1pt 边框 + 与 composer 相同的柔和阴影：上半是该用户消息前 3 行，有回复时下半是该问题的回复前 4 行，不显示作者或模型标签）；点按会 `scroll_to_index` 把时间线滚到那条消息。列左右各留一个 32pt 的空 gutter，保证中间对话列仍在面板里居中。
- 消息流、会话列表、输入框与代码块隐藏滚动条指示（`scrollbar(false)`），滚轮、触控板和键盘仍可滚动。
- 会话面板：header（标题 + 模型 badge）、`virtual_list` 消息流（底部对齐、垂直 24pt、水平 12pt、16pt 行距；助手消息不带头像与昵称；用户气泡右对齐到右柱）。每条消息下面都有一行 12pt `muted_foreground` 元信息：先是发送/回复时间（`11/6 21:12`，月日不补零），助手再接模型与耗时（`deepseek-flash · 3.2 s`）；用户那行右对齐到气泡右沿，助手那行左对齐，行末跟一个 20pt 的幽灵按钮，里面是 14pt 的复制图标。时间取发送时的本机时钟（`chrono::Local`），示例数据里的时间是写死的。
- 代码块由 `text_view` 自己画（1pt 边框 + 底色）；应用层在根组件用 `TextViewParts` 把 `CodeBlock` 的底色与边框色都设成 `code` token（明色 `#FCFCFC`，比 `muted` 的 `oklch(0.97 0 0)` 更接近页面底色；暗色显式写成同一个 `oklch(0.269 0 0)`），字号用 `sizes::CODE_FONT`（11pt，比行内代码的 13pt 小两号）。
- composer 放在同一列内：卡片顶边直接接消息视口，滚动内容在卡片上沿截断，中间不再留空白；卡片填满列宽，`rounded_3xl`、1pt `border`（`border` 色的 60%，存在感很弱）、两层自定义柔和阴影（0 1px 2px 4%、0 6px 20px 5%，比 Tailwind 的 `shadow_md` 更散更淡）、`card` 底色；卡片内 12pt 内边距，上面是最小 44pt 的输入框，下面是右对齐的 28pt 圆形发送键。卡片下方 8pt 后保留 20pt 状态栏高度（暂空），再留 8pt 到窗口底边。
- 会话切换用 `dynamic` 按会话 id 重建面板，草稿与滚动位置随之重置；重建由 `computed(selected_session_id)` 驱动，所以流式回复写 store 时不会重建面板（否则每来一个 token 都重做面板、重播淡入）。消息与侧栏由 store 的 signal 驱动。

## 动效清单

| 变化 | 工具与 token | 减少动态 |
| --- | --- | --- |
| 按钮 hover | 颜色过渡，`transition_all` + `FAST`(180ms) ease-out | 时长归零 |
| 按钮按下 | 标签按钮缩放 0.97、圆形图标按钮 0.95，同为 `FAST` | 不缩放，颜色立即变化 |
| 会话行 hover / 选中 | 背景 wash，`transition_colors` + `INSTANT`(120ms) | 时长归零 |
| 思考折起 / 展开 | 无动效：分支直接切换，不给高度做动画 | — |
| 导航短划线 hover | 宽度 10pt → 18pt、颜色 `border` → `primary_foreground`，`transition_all` + `INSTANT`(120ms) | 时长归零 |
| 导航预览卡出现 | `motion().entering(fade)`，`BASE`(240ms) ease-out；列的 hover 用信号统一记（滑到相邻划时不丢卡） | 直接显示终态 |
| composer 卡片获得焦点 | 无动效：边框不随焦点变色，输入光标本身就是提示 | — |
| 切换会话 | 面板淡入，`motion().entering(fade)`，`BASE`(240ms) ease-out | 直接显示终态 |
| 发送后的新消息 | 匀速吐字：网络增量先进 `Reveal` 缓冲，每 33ms 出一段（`llm::REVEAL_TICK`）；无逐字淡入（`text_view` 不支持字符级样式），也没有流式光标 | 缓冲一次出完 |
| 窗口缩放 | 无动效：原生布局 | — |

每条“无动效”都写明了理由；其余在运行中的应用里逐条核对。

## 模型

- 只接 DeepSeek 官方：`https://api.deepseek.com`，模型 `deepseek-flash`，key 读环境变量 `DEEPSEEK_API_KEY`（未设置时在回答位置直接显示错误）。没有配置界面，没有模型选择。
- 用 `async-openai` 的 Responses API：DeepSeek 在 `POST /responses` 提供 Responses 格式（https://api-docs.deepseek.com/api/create-response/），请求带 `stream: true` 与 `reasoning.effort: "high"`（DeepSeek 的思考默认开，这里显式要 high），读 `response.output_text.delta` 与 `response.reasoning_text.delta` 两类事件。
- 思维链显示在回复上方：思考进行中（正文还没开始）直接铺开 13pt 文字；正文一开始就折成一行“灯泡图标 + 思考”，点一下展开、再点收起。文字用 `meta` token（13pt、`Multiple(1.25)` 行距），是这个应用里最浅的一档文字；展开状态按消息 id 存在页面的 `open: Signal<Vec<u32>>` 里，切会话时重置。
- 请求跑在 `spawn` 的异步任务里、挂在 detached owner 下（NSH 的任务在 tokio runtime 上下文里轮询，见 `nsh-core-0.7.0/src/element/task.rs:748`），所以切会话不会中断回答。
- 网络任务只把增量写进 `Reveal::pending`（不碰 store）；另一个任务每 33ms 按积压量取 2–40 个字符写进 store，所以渲染与存储都是 ~30Hz，不随 token 频率增长，流结束即退出、不留空转定时器。
- 上下文是整段会话：用户与助手的历史消息按顺序发出。
- 回复结束（或出错）时补上元信息：`11/6 21:12 · deepseek-flash · 3.2 s`。
- 回复流式期间，消息的 `streaming` 为真，`text_view` 用 `MarkdownOptions::streaming` 读取：未闭合的语法按闭合渲染，且只重绘变化的块。

## 尚未做

- 会话重命名、删除、搜索。
- 停止生成、重试、超时与错误细分（目前错误文案直接写在回复位置）。
- 其它键盘快捷键与移动端布局。
