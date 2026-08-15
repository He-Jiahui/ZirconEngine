---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/popup_stack.rs
  - zircon_runtime/src/ui/surface/render/popup_position.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/theme/editor_workbench_strict.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/src/core/commands/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/retained_host/popup_anchor_metrics.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
plan_sources:
  - user: 2026-08-11 参考 UnrealEngine 与 MagicaVoxel 收敛 .zui 设计思想、样式、触发和布局逻辑
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
  - docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
tests:
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime/runtime_pseudo_state.rs
  - zircon_runtime/src/ui/tests/focus_navigation/modal_popup.rs
  - zircon_runtime/src/ui/tests/render_popup_menu.rs
  - zircon_runtime/src/ui/tests/render_popup_options.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/workbench/layout/editor_design_token_contracts/chrome_theme.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_toolbar_breakpoints
doc_type: plan
---

# 计划 12：Unreal / MagicaVoxel `.zui` 设计与工作台收敛

> 状态：执行中（2026-08-11）
> 目标：以 Unreal Slate/Starship 为架构主标准，以 MagicaVoxel 为高密度视觉与工作流次标准，收敛 Zircon 编辑器 `.zui` 的描述思想、语义样式、触发逻辑、停靠布局与自适应策略。
> 约束：本计划编排展示层收敛，不复制计划 01/02/04/06/08/09/11 已拥有的输入、布局、主题、组件、工作台和后缀契约；需要改变基础契约时回到既有 owner，并以硬切方式更新 live caller。

## 1. 执行边界与优先级例外

- 所有编辑器作者态 UI、工作台布局选择、面板开合和命令投影仍归 `zircon_editor::ui`；`zircon_runtime_interface::ui` 只持有可复用的样式、组件状态和布局描述契约，不把编辑器状态写入 runtime world。
- `.zui` 是声明式展示与交互映射资产，不成为业务状态机、命令执行器或绝对坐标脚本。业务命令和 `can_execute / checked / visibility` 由 Rust command/action owner 决定。
- 当前 MVP 计划仍处于推进期；本计划属于用户在 2026-08-11 明确要求的 UI 收敛例外。执行只改 UI/Editor 展示链和必要的通用 UI 契约，不借机扩展未完成的运行时功能。
- 当前共享工作区存在大量并行 Session。执行前按精确路径获取 lease；`ui_asset_editor.zui`、其 workspace projection、UI Asset Editor Rust owner 和 action bar 在本计划全部里程碑中永久为 scan-only，即使其原 Session 后续释放 lease，本计划也不得编辑或迁移这些路径。
- 每个里程碑完成 testing stage、独立审查和计划状态记录后，使用 coordinator 的 milestone commit；该提交作为同一里程碑的企业微信通知来源，不重复发送同一 SHA。

## 2. 当前基线与主要问题

2026-08-11 对当前工作树的只读盘点：

| 项目 | 当前事实 | 收敛要求 |
|---|---|---|
| `.zui` 规模 | `zircon_editor/assets/ui/editor` 下 248 个 `.zui` | 通过共享 token、组件 recipe 和治理测试改变整体风格，不逐页复制样式 |
| 样式主表 | `editor_workbench_strict.zui` 1411 行、169 条 rule、83 个十六进制颜色 | 拆清语义 token、组件 recipe、壳层 selector；允许 viewport 场景色作为明确例外 |
| 顶部工具栏 | 383 行、31 条 route、75 个固定尺寸声明 | 按命令优先级分组，图标命令进入 overflow；稳定高度但不固定整组总宽 |
| 主工作区 | activity rail / 左抽屉 / viewport / 右抽屉单层横排 | 以 dock region 和 splitter contract 表达；viewport 保持最小可用尺寸，侧栏按优先级收缩或折叠 |
| 工作台窗口 | 资产仍有固定 `position`，runtime 已有 popup stack/placement/focus，editor retained host 又维护 open-time anchor/clamp | runtime 成为 generic popup 几何、stack 与 focus 的唯一 owner；editor 只提交 trigger/policy 并投影 open state |
| 视觉值 | 268 个 hex，其中 token owner 外 239 个；5262 个 `Fixed` stretch | 原始颜色只允许 token owner/图像式 viewport；固定尺寸只允许图标、chrome、行高等稳定格式控件 |
| 状态 | hover/pressed/selected/focused 在 stylesheet 与 props 中重复 | 状态优先级集中到 selector/painter；focus-visible 是正交轮廓，不冒充 selected fill |
| 触发 | `UiBindingRef.id/event/route/action` 与 `UiActionRef.route/action` 可并存，且页面 route 与 Rust action id 有同义分叉 | editor command 只以 `UiActionRef.action` 为身份并解析到既有 `EditorCommandRegistry`；local binding id 与 native event 保留各自职责 |
| live root | `workbench_shell.zui` 与 `workbench_window.zui` 同时注册，组件化窗口是当前新主链 | 只在 live componentized projection 上推进，保留 host shell 仅承担宿主接线职责 |

### 2.1 根因

1. token、组件 recipe、页面布局和业务状态混在同一层，导致同一个按钮状态在 token、primitive props 和全局 stylesheet 重复定义。
2. `.zui` 大量使用 `min = preferred = max + Fixed` 模拟截图，失去 measure/arrange 与窗口缩放能力。
3. route 只描述“点击后发字符串”，缺少 Unreal `FUICommandInfo + FUIAction + FUICommandList` 那样的命令描述、可执行性和层级路由统一入口。
4. popup 的打开状态、绝对坐标和可见性在资产中并列，触发控件几何、屏幕边界、focus scope 没有形成单一链路。
5. 全局风格表按具体页面类名持续追加，产生大量近似色和状态分支，无法形成稳定的工具型桌面产品语言。

### 2.2 基线扫描口径

- 范围固定为 `zircon_editor/assets/ui/editor/**/*.zui`，2026-08-11 共 248 个文件；`editor_workbench_strict.zui` 位于该目录外，单独统计为 1411 行、169 条 rule、83 个 hex。
- `hex` 是正则 `#[0-9A-Fa-f]{6}(?:[0-9A-Fa-f]{2})?` 的匹配数，不是去重色值数；268 个匹配减去唯一 token owner `editor/theme/editor_tokens.zui` 的 29 个匹配，得到页面/组件侧 239 个。原先未定义口径的“219 个内联颜色”不作为治理基线。
- `Fixed` 是同一文件集上 `\bFixed\b` 的词法匹配数，当前为 5262；它用于发现候选点，不等同于 5262 个违规，因为图标、chrome、row height 等稳定格式控件可以进入 family allowlist。
- M2/M5 的 guard 必须复用上述范围与表达式并输出逐文件清单，使数量变化可以审计；最终违规判定仍由 token/viewport/data-visualization/brand allowlist 进行结构化分类。

## 3. 参考证据与采用结论

### 3.1 Unreal Engine：主标准

| 参考文件 | 证据 | Zircon 采用 |
|---|---|---|
| `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateStyle.h` | `FSlateStyleSet` 提供命名、类型化 style/property registry 与 parent fallback | `.zui` token 保持语义命名；组件按 family recipe 解析，不由页面直接拼颜色和 padding |
| `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateStyleRegistry.h`、`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/AppStyle.h` | 中央 style registry + application-wide singleton accessor | 单一 editor workbench theme owner；页面只引用 token/recipe |
| `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/StyleColors.h` | Background/Panel/Header/Hover/Primary/Select 等语义色与 theme change 广播 | palette 改为中性层级 + 少量语义角色，避免青蓝色值在 selector 中扩散 |
| `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/ToolBarStyle.h` | toolbar 把 button/toggle/separator/icon size/padding/overflow 作为成组 recipe | 顶栏/面板头/viewport toolbar 各有明确 chrome recipe 和 overflow 规则 |
| `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h`、`dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UIAction.h`、`dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h` | descriptor 与 execute/can-execute/check/visibility 分离，并支持 command list hierarchy | `.zui` 只绑定 action id；Rust 统一 command catalog、状态与快捷键；同一命令不在多处复制逻辑 |
| `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/LayoutExtender.h`、`dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/LayoutService.h` | 以 area/splitter/stack/tab 扩展点构造并版本化保存布局 | 工作台从固定三列改为 region/splitter/preset；坏布局回退到版本化默认值 |
| `dev/UnrealEngine/Engine/Source/Editor/EditorStyle/Private/StarshipStyle.cpp` | Recessed/Header/Input 语义面、0-4px 圆角、低 padding、选中 Primary、明确 tab/dock 状态 | 采用安静的中性深灰、细分隔线、紧凑行高、单一蓝色 selection/focus 信号 |
| `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Test/SlateUserCursorLockTest.cpp` | 提供明确几何与边界组合的测试先例；它不是 popup 或 docking 的直接测试 | Zircon 采用该测试组织方式，并由自身 popup/focus、autolayout 与 splitter tests 直接覆盖相应行为 |

### 3.2 MagicaVoxel：次标准

| 参考文件 | 证据 | Zircon 采用 |
|---|---|---|
| `dev/MagicaVoxel-0.99.7.2-win64/config/ui/editor.sty` | 独立 style 文件、`-h/-s/-d` 状态后缀、30-70 中性灰表面、极少蓝/橙强调 | 用少量可辨识状态替代装饰性卡片和同色系层层描边；危险/警告使用暖色语义色 |
| `dev/MagicaVoxel-0.99.7.2-win64/config/ui/top.ui` | 32px 顶部行、左 mode、中项目名、右常用图标命令和 shortcut hint | Zircon 顶栏收敛为低高度 command strip；主项目上下文与运行控制保持稳定位置 |
| `dev/MagicaVoxel-0.99.7.2-win64/config/ui/editor.ui` | Layer/VBox/HBox 组合、左/中/右区、可折叠/resize panel、popup 独立层 | `.zui` 用容器、region、splitter 和 overlay 描述结构；不把 popup 混进普通流布局 |
| `dev/MagicaVoxel-0.99.7.2-win64/ui_reimplementation_guide.md` | retained view tree、文本 DSL、behavior/render delegate 分离、measure/arrange、事件到命令字符串 | 保留 `.zui` 文档 + Rust runtime 的分层，但把命令字符串提升为有 catalog 约束的稳定 action id |

实际运行参考确认了其核心不是“像素复刻”，而是信息密度、持续可见的 viewport、低装饰分区和按任务展开的工具面板。本计划不复制 MagicaVoxel 的专用 voxel 工具或私有二进制实现。

### 3.3 Slint：声明式实现交叉校验

- `dev/slint/ui-libraries/material/src/ui/styling/material_style_metrics.slint` 证明尺寸应由集中 metrics 提供。
- `dev/slint/ui-libraries/material/src/ui/components/state_layer.slint` 证明 hover/press/focus 应由共享状态层处理，键盘激活和 pointer 状态共用组件行为。
- `dev/slint/ui-libraries/material/src/ui/components/menu.slint` 把 popup、menu item、selected 和 close policy 分层；`dev/slint/tests/cases/elements/popupwindow_position.slint` 与 `dev/slint/tests/cases/focus/popupwindow_focus.slint` 覆盖动态位置和焦点恢复。
- Zircon 不采用 Material 的大圆角、48-56px 移动端密度和 ripple；只采用其声明式属性、组件行为、测试分层方式。

## 4. 目标设计契约

### 4.1 `.zui` 描述职责

`.zui` 可以声明：

- 组件树、稳定 `control_id`、语义 class、slot 和 import；
- min/preferred/max、stretch、alignment、dock region、splitter、overlay anchor；
- token/recipe 引用和有限的初始展示状态；
- `event -> action_id` 映射、tooltip 文案与可访问名称。

`.zui` 不可以拥有：

- 文件保存、编译、播放等业务分支；
- `can_execute`、checked、visibility 的重复权威状态；
- 屏幕绝对 popup 坐标、手工碰撞修正或焦点恢复算法；
- 页面私有的 hover/pressed/disabled 色值；
- 为截图而固定的整组宽度和窗口主区域高度。

### 4.2 样式层级

```text
editor theme tokens
  -> semantic surface/text/accent/status roles
    -> component family recipe (toolbar, panel header, row, field, popup)
      -> pseudo-state overlay (hover, press, selected/open, focus-visible, disabled)
        -> page class only for domain-specific composition
```

- 表面角色：`window / chrome / panel / recessed / input / popup / hover / selection`。
- 文本角色：`primary / secondary / disabled / inverse`。
- 强调角色：selection/focus 使用单一冷蓝；success/info/warning/error 各自保留语义色，不能用 accent 冒充。
- 形状：工作台 chrome 和面板为 0px；输入/按钮 2-4px；popup 最多 4px；不使用装饰性 pill/card。
- 状态输出是“一个 primary visual state + 若干独立 overlay”，不能压成单个互斥枚举后丢失信息。
- primary fill/foreground 的确定顺序固定为：`disabled > loading > drop-target > dragging > pressed > open > checked > selected > hovered > normal`；同一控件多状态并存时取最左者，因而 open/checked/selected 的平局也有确定结果。
- `focus-visible` 是独立 outline overlay，在 selected/checked/open fill 之上仍必须可见；普通 pointer focus 不画该 outline。drop indicator、validation/error indicator 也作为独立 overlay 合成，不得覆盖 primary fill 的身份。

### 4.3 命令与触发

```text
.zui event
  -> stable action id
    -> scoped command list
      -> descriptor(label/icon/chord/type)
      -> action(execute/can_execute/check_state/visibility)
        -> state projection back to retained UI
```

- pointer、keyboard、menu、command palette 调用同一个 action id。
- menu item 不再用页面私有字符串决定 enabled/checked；由 command projection 注入。
- tooltip 由共享 delayed-open policy 管理；disabled command 仍可显示原因，但不触发 execute。
- 现有 schema 的 survivor contract 固定为：`UiBindingRef.id` 只做资产内 binding identity、诊断和 target assignment；`UiBindingRef.event` 只表示 native `UiEventKind`；`UiBindingRef.action.action` 是唯一 editor command identity，并解析到既有 `zircon_editor::core::commands::EditorCommandRegistry`。
- M4 完成迁移后，editor-command binding 禁止 `UiBindingRef.route` 和 `UiActionRef.route`，也不新建 UI-local command catalog；非 editor-command 的 runtime navigation route 只有在 owner 明确且 guard allowlist 登记后才可保留。

### 4.4 Popup 与 focus 唯一所有权

- `zircon_runtime::ui::surface::popup_stack`、`surface::render::popup_position` 和 runtime focus/input routing 是 generic popup 的唯一实现 owner：依据实时 trigger frame 定位，执行 flip/clamp，维护 nested stack、outside click、Escape、focus trap/restore。
- editor command/menu/palette owner 只请求 `open/close + placement policy + trigger identity`，并把 runtime 返回的 open/checked/focus projection 写回 retained UI；不得再计算 generic popup 的绝对 x/y。
- M4 硬切时删除 `.zui` 的 open-time absolute `position/popup_anchor_*` 写入路径，并删除或缩减 `zircon_editor/src/ui/retained_host/popup_anchor_metrics.rs` 及 workbench callback 中重复的 generic anchor/clamp/dismiss 逻辑。editor 只可保留确有领域含义的策略常量，不能保留第二套几何算法。
- runtime-interface 只承载可序列化 placement/trigger/policy contract，不承载 editor command 或窗口状态；popup 关闭后的 focus target 由 runtime stack 记录的触发控件决定。

### 4.5 布局与自适应

- 固定格式控件可以使用稳定尺寸：图标按钮、chrome 高度、row height、separator、status item。
- 工具组、抽屉、面板和文档区必须使用 min/preferred/max + stretch，禁止三值相等后再固定整组。
- 主区优先级固定为：viewport/document > active inspector/scene tree > auxiliary drawer > low-priority toolbar labels。
- 宽度不足时顺序为：缩短文本 -> 图标化低优先命令 -> 收入 overflow -> 折叠辅助 drawer；不得压缩 viewport 到不可交互尺寸。
- popup、toast、command palette 使用 overlay/portal，不参与主 VBox/HBox 的测量。
- splitter 的视觉线可为 1px，但 pointer hit target 不小于 8px；layout preset 版本化持久化并支持 default fallback。
- 响应式判定唯一复用 `zircon_editor::ui::workbench::autolayout::WorkbenchLayoutTier`。输入先按 `ResolutionContext::logical_extent(physical_width, scale_factor)` 转为 logical px，再用 `EditorDensityTokens` 的 `breakpoint_ultra_width=480`、`breakpoint_narrow_width=640`、`breakpoint_wide_width=1260` 产生 Ultra/Narrow/Regular/Wide；测试同时覆盖 1.0/1.25/1.5/2.0 DPI 的等价 logical-width 结果。
- 当前验收 oracle 固定为：普通窗口最小 640x420 logical px、Ultra 最小 420x360；document/viewport 宽度不得低于可用主区的 `minimum_document_width_fraction=0.5`；Ultra/Narrow 自动折叠 right drawer，Regular/Wide 不自动折叠。若 token 值在 M2 经审查调整，测试读取 token 字段而不复制常量，但断言上述 tier 关系与 document 优先级不变。
- collapse/restore 状态由 editor workbench layout model 和 `LayoutPresetPersistenceStore` 唯一拥有；以 `LAYOUT_PRESET_PERSISTENCE_VERSION` 版本化，缺失或版本不匹配回退 `Authoring` preset。窗口跨 tier 后，自动折叠不能覆盖用户持久化的 drawer/preset，恢复到足够宽度时按持久化状态恢复。

## 5. 依赖顺序与里程碑

### M1 — 参考基线、架构和治理范围

- 固化本计划的 Unreal 主参考、MagicaVoxel 次参考、Slint 交叉证据和当前 `.zui` 量化基线。
- 明确计划 04/08/09/11 的 owner 关系、MVP 例外、并行 Session 避让和每里程碑提交/企业微信规则。
- Testing stage：计划 frontmatter/path/链接审计、plan output audit、`git diff --check`；独立 reviewer 核对证据与范围未越过 runtime/editor 边界。
- Exit evidence：coordinator validation batch + M1 milestone commit。

### M2 — 语义 token、chrome metrics 与状态 recipe

- 稳定 `EditorDesignTokens` 的 palette/chrome/density/state role 分层，收束当前进行中的 chrome/density 拆分，消除同义值和 editor/runtime 两份默认值漂移。
- 把工作台主色改为中性深灰层级、蓝色 selection/focus、暖色 warning/error；从 `editor_workbench_strict.zui` 移除可由语义 token/recipe 表达的私有色。
- 为 toolbar/panel-header/row/field/popup 建立共享 family recipe；focus-visible 与 selected 状态分离。
- 添加 token round-trip、cascade alias、state priority、focus-visible 正交性、raw color allowlist 的 focused tests/guards。
- Testing stage：runtime-interface token/style focused tests、runtime v2 pseudo-state tests、editor theme contract tests；随后做 scoped Cargo check/test batch。
- Exit evidence：M2 validation batch、独立 review、计划状态行、milestone commit/企业微信。

### M3 — 工作台壳层与高密度布局

- 重构 live `workbench_window.zui` 组件链：25-32px chrome、紧凑 panel header/status bar、低装饰分区、viewport-first 主区。
- 顶栏按 project/context、transform tools、run、layout 分组；移除固定整组总宽，低优先 command 进入 overflow。
- M3 只改变 command 的分组、可见性和 overflow 投影，必须保留当前 binding 行为与 action identity；不得在 M3 提前改写 route/action 字符串，命令身份硬切只在 M4 完成。
- `workbench_main_band.zui` 改为可持久化 region/splitter 约束，统一 activity rail/left/right/bottom drawer 的 min/preferred/max 和 collapse priority。
- 迁移 scene tree、inspector、viewport、console/status 等高频壳层；通过共享 primitive 影响全部模块，但不改其他 Session 持有的 UI Asset Editor 专属资产。
- 添加窗口宽度档、稳定 chrome 尺寸、viewport 最小面积、drawer collapse/restore 和文本不溢出的布局测试。
- Testing stage：editor componentized workbench、toolbar breakpoint、autolayout/geometry 和 template asset focused tests；运行产品启动 smoke。
- Exit evidence：M3 validation batch、独立 review、桌面/窄窗口截图、milestone commit/企业微信。

### M4 — 命令、popup、focus 与触发一致性

- 统一 toolbar/menu/context-menu/command-palette 的 action id 与既有 `EditorCommandRegistry` descriptor；按 4.3 survivor contract 硬切 editor-command binding，移除 `UiBindingRef.route`/`UiActionRef.route` 及所有同义 caller，不保留兼容 shim 或第二 catalog。
- 把 popup 从窗口绝对位置改为 trigger-frame anchor + flip/clamp；统一 outside click、Escape、nested popup、disabled item、focus trap/restore。
- 将 play/stop、tool mode、layout、module overflow 的 checked/visibility 投影回组件状态；不由资产硬编码选中态。
- 为 pointer/keyboard/menu/command palette 同命令、边缘 popup、无 focusable popup、nested modal 和 disabled command 增加测试。
- Testing stage：runtime focus/popup/event routing、editor workbench menu/command callback、command palette focused tests和产品交互 smoke。
- Exit evidence：M4 validation batch、独立 review、交互验收截图/记录、milestone commit/企业微信。

### M5 — `.zui` 家族迁移、治理闸口与产品验收

- 按 primitives -> shell -> core modules -> secondary modules 的顺序迁移仍绕过 token/recipe 的资产；viewport 场景颜色、数据可视化色和插件品牌色必须进入显式 allowlist。
- 添加治理闸口：禁止页面新增原始交互色、重复状态 recipe、主区域绝对 popup anchor、无理由固定整组尺寸、未注册 action id，以及 editor-command binding 中出现 `UiBindingRef.route`/`UiActionRef.route`。
- UI Asset Editor 的 `ui_asset_editor.zui`、workspace projection、Rust owner 与 action bar 永久 scan-only：M5 可以报告其治理结果，但无论 lease 是否释放都不得编辑、自动迁移或纳入本计划提交。
- 清理 `editor_workbench_strict.zui` 中失效 selector，验证 248 个 editor `.zui` 可解析、import 可达、control id/action id 无冲突。
- 使用真实 Zircon Hub 窗口做桌面/窄窗口截图与像素非空检查，核对无重叠、文本不裁切、popup 不出屏、viewport 始终可用。
- Testing stage：`.zui` 全量静态治理、runtime/editor focused suites、Windows-native 产品构建/启动和最终 scoped regression batch。
- Exit evidence：M5 validation batch、独立 review、产品截图、计划终态记录、milestone commit/企业微信。

## 6. 风险与拒绝条件

- 若基础 style/popup/focus failure 已由其他编号计划拥有，创建 cross-plan failure handoff；不在 editor 资产中添加兼容分支或视觉掩盖。
- 若某个 fixed size 属于真实稳定格式控件，保留并在 guard allowlist 中按组件 family 说明；治理不得机械禁止所有 `Fixed`。
- 若 token 变更导致 viewport 图像、数据可视化或插件品牌辨识度下降，回退该具体语义映射，不回退整体中性工作台层级。
- 若视觉截图正常但 keyboard/popup/focus 测试不完整，里程碑不接受。
- 若测试通过但 live componentized projection 未使用改动资产，里程碑不接受。
- 若当前工作树的未归属改动无法证明来源，不把它静默纳入 milestone commit；先通过 coordinator attribution/lease 确认归属。

## 7. 状态与产出记录

每个里程碑 testing stage、独立审查和 coordinator acceptance 完成后记录一次；实现切片不单独写产出记录。

子计划 current-source manifest 的 `Status: completed` 只表示该份 source output record 已达到 coordinator pre-bind 所需的结构完整性，不表示里程碑已接受。里程碑 acceptance 只以当前 immutable manifest 的 managed validation、独立 reviewer `0 Critical / 0 Important`、coordinator gate 和 milestone commit 为准；在这些条件齐备前，manifest 必须另列 `Acceptance: pending_independent_review`，本表状态也不得写“完成”。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
