---
related_code:
  - zircon_editor/src/workbench/autolayout.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/drawer_resize.rs
  - zircon_editor/src/host/slint_host/tab_drag.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/host/slint_host/viewport.rs
  - zircon_editor/src/host/binding_dispatch.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/host/manager/startup.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/workbench/layout.rs
  - zircon_editor/src/workbench/model.rs
  - zircon_editor/src/workbench/reflection.rs
  - zircon_editor/src/workbench/snapshot.rs
  - zircon_editor/src/workbench/startup.rs
  - zircon_editor/src/workbench/view.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor_ui/src/binding.rs
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
implementation_files:
  - zircon_editor/src/editing/state.rs
  - zircon_editor/src/workbench/autolayout.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/manager/startup.rs
  - zircon_editor/src/host/slint_host/drawer_resize.rs
  - zircon_editor/src/host/slint_host/tab_drag.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/host/slint_host/viewport.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/workbench/startup.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/workbench/layout.rs
  - zircon_editor/src/workbench/model.rs
  - zircon_editor/src/workbench/reflection.rs
  - zircon_editor/src/workbench/snapshot.rs
  - zircon_editor_ui/src/binding.rs
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
  - docs/editor-and-tooling/prototypes/editor-workbench-hybrid-shell.html
plan_sources:
  - user: 2026-04-13 JetBrains Hybrid Workbench Shell Spec + Implementation Plan
  - user: 2026-04-14 Slint Workbench 响应式 AutoLayout 与约束求解计划
  - user: 2026-04-14 编辑器启动最近工程与 Welcome 新建工程计划
  - .codex/plans/Zircon UI Editor UI Binding & Reflection Architecture.md
tests:
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/host/slint_drawer_resize.rs
  - zircon_editor/src/tests/host/slint_tab_drag.rs
  - zircon_editor/src/tests/workbench/fixture.rs
  - zircon_editor/src/tests/workbench/view_model.rs
  - zircon_editor/src/tests/workbench/reflection.rs
  - cargo test -p zircon_editor --locked
  - cargo test -p zircon_asset -p zircon_manager --locked
doc_type: module-detail
---

# Editor Workbench Hybrid Shell

## Purpose

这一版 workbench shell 的目标不是继续堆一个“功能都在，但视觉和行为都很散”的宿主，而是把 editor 壳体收束成一套明确的 Hybrid Shell 规范：

- 视觉方向固定为 JetBrains Rider 对齐的 workbench shell
- 默认布局固定为 editor-first，大中台 viewport 占主导
- Pane 语义和空状态固定属于 pane 自己，不属于某个位置
- 布局仍完全由 `WorkbenchLayout` 驱动，用户可以把 pane 挪到任何 drawer/document host
- HTML 原型、`WorkbenchViewModel`、反射树和未来 Slint 宿主必须读同一套语义，而不是各做各的

## Startup Welcome Session

### Session Authority

启动现在不再默认伪造一个 `sandbox-project` 和假 world。编辑器先由 `EditorManager::resolve_startup_session()` 解析结构化启动会话，再决定进入：

- `EditorSessionMode::Project`
- `EditorSessionMode::Welcome`

最近工程配置统一落在 `editor.startup.session`，至少包含：

- `last_project_path`
- `recent_projects`

每次启动都会重新验证最近工程，而不是把上一次的验证结果当权威缓存。失效工程会继续保留在 recent list 中，并在 Welcome 页上显示诊断标签。

### Welcome Page Contract

Welcome 页不是独立 launcher，而是 workbench 内的 exclusive page：

- `editor.welcome` descriptor 由 `EditorManager` 注册并托管
- `EditorStartupSessionDocument` 负责 recent list、draft 和状态消息
- `WelcomePaneSnapshot` 只承载展示数据
- Slint `workbench/welcome.slint` 负责 JetBrains 风格的 Recent Projects + New Project 双栏界面

当前 Welcome 页主操作固定为：

- 打开 recent project
- 移除 recent entry
- 创建 `Renderable Empty` 目录式项目
- 按 `Project Name + Location` 推导现有目录并打开

### Renderable Empty Template

Welcome 创建的新工程不再只是 manifest 骨架，而是立即可打开、可渲染的目录式模板。`EditorProjectDocument::create_renderable_template(...)` 负责脚手架生成，固定写出：

- `zircon-project.toml`
- `assets/scenes/main.scene.toml`
- `assets/materials/default.material.toml`
- `assets/shaders/pbr.wgsl`
- `library/`

创建完成后，host 统一走 `create_project_and_open -> apply_startup_session` 链路，更新 recent list、恢复 layout、替换 runtime world，并关闭 Welcome exclusive page。

## Visual System

### Shell Character

Hybrid Shell 的视觉基调固定为：

- 冷灰底色，连续 IDE 壳体，不做圆角卡片式内嵌窗口
- 蓝色焦点高亮，用于活动 tab、rail 激活态、主按钮和输入焦点
- icon-first rail + 扁平 tab strip，而不是大块按钮矩阵或胶囊 chip
- 中央 Scene/Game 文档区面积最大，明确体现“编辑器优先”
- Inspector 与 Console 走高密度属性表 / 输出面板观感，不做大号占位卡片

HTML 原型在 `docs/editor-and-tooling/prototypes/editor-workbench-hybrid-shell.html` 中作为视觉 oracle。当前运行时已经切到 Slint host；运行时代码只负责跟随这套壳体语义，不把某个宿主实现细节反向定义成视觉规范。

### Stable Shell Chrome IDs

壳层稳定控件命名空间固定为：

- `WorkbenchMenuBar/*`
- `ActivityRail/*`
- `ToolWindow/*`
- `DocumentTabs/*`
- `InspectorView/*`
- `ViewportToolbar/*`
- `StatusBar/*`

这里的 `*` 代表具体 control id。HTML 原型和未来 Slint 组件必须映射到同一套稳定 id，避免“原型一套命名、宿主一套命名、headless 测试第三套命名”。

## Builtin Hybrid Preset

### Default Startup Preset

source-controlled builtin preset 固定为：

- left `Project / Assets / Hierarchy`
- center `Scene / Game`
- right `Inspector`
- bottom `Console`

当前内置比例会刻意压低底部 Console、收窄左右 drawer，让中心 viewport 在默认启动时保持主导。

这个 preset 只定义 builtin startup shell，不定义用户最终布局。真正的布局恢复优先级仍然是：

1. project layout
2. global layout
3. builtin Hybrid preset

### Persisted Layout Authority

`zircon_editor/src/workbench/layout.rs` 中的 `WorkbenchLayout` 仍然是唯一持久化 schema。  
本轮没有引入第二套 HTML-only 或 Slint-only 布局格式。

因此：

- 用户把 `Project` 拖到右边，语义不变，只是 host slot 变了
- 用户把 `Console` 挪到左边，空状态和内容不变
- project/global layout 覆盖 builtin preset 时，仍然走现有 `LayoutManager` 恢复链路

### Named Layout Preset Assets

除了 startup builtin preset，workbench 现在还支持“用户命名 preset”：

- 打开项目后，preset 保存到 `assets/editor/layout-presets/<sanitized-name>.workbench-layout.json`
- preset 文件内部保留原始 `preset_name`，文件名只负责安全落盘
- 未打开项目时，preset 回退到用户配置键 `editor.workbench.presets`
- `EditorManager` 会先枚举项目 preset asset，再并入 global preset 名单，最后在 shell 中渲染可加载列表

这意味着“把 Project 固定放右边”这类偏好不再需要写死进 startup preset；用户可以把它保存成项目资产，让团队项目带着自己的默认壳体走。

## Current Drag And Drop Scope

当前 Slint shell 已经支持真实 pointer-driven tab drag/drop，但范围刻意收窄到“壳体组级别”：

- document tabs 可以拖到 `left / right / bottom / document`
- tool-window header tabs 也可以拖到同样四个宿主组
- drop 到 drawer side 时，会优先落到该侧当前活跃/可见的 drawer slot；如果该侧没有活跃 stack，则回退到 canonical slot
- drop 到 `document` 时，会优先落到当前 active workbench page；若当前主页面不是 workbench，则回退到第一个 workbench page

这一层已经足够支持用户把 `Project`、`Inspector`、`Console`、`Scene/Game` 在默认壳体内重新收纳，而不需要先保存 preset 再看结果。

当前仍未支持的拖放目标是：

- document edge split target
- floating window detach target
- exclusive activity page promote target

这些更细粒度目标仍然保留在 layout/drag model 中，但 Slint 壳体暂时还没有把它们全部暴露为可视 drop zone。

## Current Splitter Scope

当前 Slint shell 已经支持 root-captured drawer splitters：

- left / right / bottom 三个可见 stack 都提供真实 pointer-driven resize
- splitter 释放事件在 shell 根级捕获，不依赖原始 handle 命中区域
- side resize 会把同一侧的全部 drawer slot extent 同步写回，避免 `LeftTop/LeftBottom` 或 `RightTop/RightBottom` 切换时宽度跳变
- 保存 preset asset 时，这些 extent 会直接落入现有 `WorkbenchLayout`

同一轮里，tab drag overlay 也改成 root-captured pointer release。这样鼠标从 tab 拖出后在主内容区任意位置释放，overlay 都会可靠清理，不会残留错误的 drop hint。

## Responsive AutoLayout Solver

这一轮开始，Slint runtime 不再把 `left/right/bottom extent` 当作 UI 侧锚点公式的输入真源。壳体现在明确分成两层：

- Rust `autolayout` 求解器负责根据真实窗口尺寸、`WorkbenchLayout`、descriptor 默认约束、layout/view override，以及当前活动 tab，统一解出 shell 几何。
- Slint `WorkbenchShell` 只消费 frame 结果并渲染，不再自己推导 `document_zone_x`、`right_stack_x`、`bottom_panel_y`。

当前共享求解结果模型是 `WorkbenchShellGeometry`，其中固定包含：

- `window_min_width` / `window_min_height`
- `region_frames`
- `splitter_frames`
- `viewport_content_frame`

当前 host 更新链路也固定为：

1. Slint 导出真实 `shell_width_px` / `shell_height_px`
2. host 监听尺寸变化并设脏 `WindowMetrics`
3. layout/preset/tab/drag/splitter 改动设脏 `Layout`
4. `recompute_if_dirty()` 统一调用 `compute_workbench_shell_geometry(...)`
5. 求解后的 frame 和最小窗体尺寸回灌到 Slint
6. `viewport_content_frame` 再送入 `ViewportInput::Resized`

### Constraint Semantics

当前轴向约束语义已经固定为：

- `min == 0` 表示无最小值
- `max == -1` 表示无最大值
- `preferred` 是显式字段，drawer `extent` 作为主轴 preferred 覆盖
- 放大时只让 `Stretch` 项吸收空间，优先级高者先分配，同优先级再按 weight 分配
- 缩小时所有仍高于 `min` 的项都可压缩，但低优先级先缩，同优先级再按 weight 分摊

默认上：

- `Document` 保持更高优先级和更高权重
- 左/右/底工具区维持中优先级
- 顶栏、宿主栏、状态栏走固定尺寸

### Runtime Resize Behavior

splitter 现在不再在 Slint 里直接计算最终 extent。当前行为改成：

- pointer down 时 host 记录活动 region、起始指针位置和基础 preferred
- pointer move 只更新内存中的 transient preferred，并立刻重算 frame
- pointer up 才把结果提交回 `drawer.extent`

这样带来的结果是：

- 右侧 `Inspector` 和底部 `Console` 的锚点跟随 solver，而不是跟随旧公式
- 拖拽过程中 splitter 命中区与显示边界一致
- viewport 尺寸来自求解后的内容区，而不是从 Slint pane 反推
- 窗体最小尺寸来自所有 region/host chrome 的聚合结果，不能再被缩到把文档区挤没

## Pane Catalog

### Tool Windows

下列 pane 属于 tool-window family，可以驻留任何 drawer stack：

- `Project`
- `Assets`
- `Hierarchy`
- `Inspector`
- `Console`

它们的 content kind 现在由 `ViewContentKind` 显式建模，而不是靠固定 slot 推断。

### Document Pages

文档区固定包含两类长期文档 pane：

- `Scene`
- `Game`

并支持按需创建：

- `Prefab Editor`

`Prefab Editor` 不在 startup layout 中创建占位 tab。只有真的打开 prefab 时才出现实例和标签。

## Close Rules

关闭规则固定为：

- `Scene` 不可关闭
- `Game` 不可关闭
- `Prefab Editor` 可关闭
- tool windows 默认不走 document-style close；显示/隐藏和激活通过 `DockCommand`/rail/menu 驱动

当前实现已经在 `EditorManager::close_view` 对 `editor.scene` 和 `editor.game` 做了 non-closeable 保护，并有测试覆盖。

## Empty State Ownership

空状态属于 pane，不属于 slot。也就是说：

- `Project` 挪到右边，仍显示 `No project open`
- `Hierarchy` 从左边换到 document host，仍然是 hierarchy 的空状态
- `Console` 换到别的位置，也仍然只显示 console 自己的空状态

### Canonical Empty States

固定文案如下：

- `Project / Assets`
  - title: `No project open`
  - primary action: `Open Project`
  - `Recent Projects` 只保留在菜单，不做大按钮墙
- `Hierarchy`
  - no project: `No scene loaded`
  - project open but no nodes: `No nodes in scene`
- `Scene`
  - no project: `No project open`
  - project open but no active scene: `No active scene`
- `Prefab Editor`
  - 不渲染预占位空 tab
- `Inspector`
  - `Nothing selected`
  - 不显示一堆禁用输入框
- `Console`
  - 无输出时显示最近一次状态或 `No output yet`

当前 `WorkbenchViewModel` 已经输出 `PaneEmptyStateModel`，并且 Slint host 直接消费这套空状态语义。
当前壳体实现中，tool window 空状态使用顶部锚定的紧凑消息样式；中央 `Scene/Game` welcome state 继续保持 document-centered。

### Runtime Startup Selection Rule

运行时 startup 现在明确区分两件事：

- shell 仍然可以带着 renderable default level 启动，以保证 viewport / renderer 链路稳定
- 但 `EditorState::new` 必须在 no-project welcome 模式下主动清空 selection

这条规则的直接结果是：

- `Inspector` 在无项目启动时稳定显示 `Nothing selected`
- `Scene/Game` welcome state 不会再因为默认 camera/cube 选中而退化成可编辑状态
- 需要“默认就有选中对象”的测试或编辑路径，必须显式走 `EditorState::with_default_selection(...)`

## Backend-Neutral View Model

`zircon_editor/src/workbench/model.rs` 现在除了旧的 menu/host strip/status bar 视图模型外，还额外输出：

- `tool_windows: BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>`
- `document_tabs: Vec<DocumentTabModel>`
- `PaneTabModel`
- `PaneEmptyStateModel`
- `PaneActionModel`

这让宿主层不需要再根据固定 left/right/bottom 假定 pane 类型，而是可以直接读取：

- 哪个 slot 有哪些 pane
- 当前 active tab 是谁
- 某个 pane 是否 closeable
- 某个 pane 当前该显示正常内容还是空状态

## Fixtures And Prototype Contract

`zircon_editor/fixtures/workbench/` 下的 fixture 现在对齐 builtin Hybrid preset：

- `default-layout.json`
- `view-descriptors.json`
- `view-instances.json`
- `editor-data.json`

这些 fixture 现在表达的是“startup shell 已经准备好，但还没有打开项目”的状态：

- `project_open = false`
- `Scene/Game` tab 已存在
- `Project/Assets/Hierarchy/Inspector/Console` pane 已存在
- `Project` 为左侧 active pane
- `Inspector` 默认右侧打开
- `Console` 底部打开

HTML 原型使用 fixture-shaped data 渲染 builtin preset，并提供 `Project docks right` 的 alternate preset，证明 pane placement 来自 layout JSON，而不是 DOM 写死。

## HTML Skeleton And Slint Mapping

当前 Slint 宿主的组件边界固定为：

- `WorkbenchShell`
- `WorkbenchTopBar`
- `ActivityRail`
- `ToolWindowStack`
- `DocumentWorkspaceHost`
- `WorkbenchStatusBar`
- pane components for `Project`, `Assets`, `Hierarchy`, `Inspector`, `Console`, `Scene`, `Game`, `Prefab Editor`

映射原则：

- Material components 只提供 base controls
- theme layer 负责把它们压到 JetBrains-like 视觉系统
- pane content 读取 `WorkbenchViewModel`，不直接绑定固定 slot
- dock tree 继续由 `WorkbenchLayout` 驱动，不额外引入 Slint-side 布局 schema

## Runtime Status

当前运行时宿主已经切到 Slint：

- `zircon_editor/src/host/slint_host/app.rs` 负责启动 shell、绑定菜单/标签/选择/视口事件
- `zircon_editor/src/host/slint_host/ui.rs` 把 `WorkbenchViewModel` 投影成 Slint 属性模型
- `zircon_editor/src/host/slint_host/viewport.rs` 负责把共享 `wgpu` 纹理导入到 Slint `Image`
- `zircon_editor/ui/workbench.slint` 与 `zircon_editor/ui/workbench/chrome.slint` 提供 JetBrains-like shell chrome 与 pane surface

这一轮已经完成的是：

- Slint-only runtime path，`zircon_editor` / `zircon_entry` 不再依赖 `iced`
- builtin Hybrid preset fixture 与 config-driven HTML prototype
- project-aware layout preset assets with config fallback
- Slint `Window` menu preset entry for save/load/reset
- Slint pointer-driven tab drag/drop across shell host groups
- Slint pointer-driven left/right/bottom splitter resize
- layout-agnostic pane empty states
- document closeability rules
- WGPU27 共享纹理 viewport bridge

仍未完成的是：

- 更完整的 document split / floating workspace 视觉宿主
- viewport toolbar 更丰富的 typed action 接线
- 持续把 Slint 视觉细节逼近 HTML 原型
