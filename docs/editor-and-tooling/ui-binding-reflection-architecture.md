---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/binding/model/mod.rs
  - zircon_ui/src/event_ui/manager/mod.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/ui/binding/asset/mod.rs
  - zircon_editor/src/ui/binding/dock/mod.rs
  - zircon_editor/src/ui/binding/dock/command.rs
  - zircon_editor/src/ui/binding/draft/mod.rs
  - zircon_editor/src/ui/binding/draft/command.rs
  - zircon_editor/src/ui/binding/selection/mod.rs
  - zircon_editor/src/ui/binding/viewport/mod.rs
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/ui/binding/viewport/codec.rs
  - zircon_editor/src/ui/binding/welcome/mod.rs
  - zircon_editor/src/ui/control.rs
  - zircon_editor/src/ui/reflection.rs
  - zircon_editor/src/tests/ui/binding.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/core/editor_event/journal.rs
  - zircon_editor/src/core/editor_event/replay.rs
  - zircon_editor/src/core/editor_event/transient.rs
  - zircon_editor/src/core/editor_event/host_adapter.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/mod.rs
  - zircon_ui/src/template/document.rs
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/source_sync.rs
  - zircon_editor/src/core/editing/ui_asset/tree_editing.rs
  - zircon_editor/src/core/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs
  - zircon_editor/src/core/editing/ui_asset/command.rs
  - zircon_editor/src/core/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/host/slint_window.rs
  - zircon_editor/src/ui/workbench/event/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/templates/pane_surface_controls.toml
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/ui/templates/asset_surface_controls.toml
  - zircon_editor/ui/templates/startup_welcome_controls.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
implementation_files:
  - zircon_ui/src/binding/model/mod.rs
  - zircon_ui/src/event_ui/manager/mod.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/ui/binding/dock/command.rs
  - zircon_editor/src/ui/binding/dock/codec.rs
  - zircon_editor/src/ui/binding/draft/command.rs
  - zircon_editor/src/ui/binding/draft/codec.rs
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/ui/binding/viewport/codec.rs
  - zircon_editor/src/ui/control.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/core/editor_event/journal.rs
  - zircon_editor/src/core/editor_event/replay.rs
  - zircon_editor/src/core/editor_event/transient.rs
  - zircon_editor/src/core/editor_event/host_adapter.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/mod.rs
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/source_sync.rs
  - zircon_editor/src/core/editing/ui_asset/tree_editing.rs
  - zircon_editor/src/core/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs
  - zircon_editor/src/core/editing/ui_asset/command.rs
  - zircon_editor/src/core/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/workbench/event/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/templates/pane_surface_controls.toml
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/ui/templates/asset_surface_controls.toml
  - zircon_editor/ui/templates/startup_welcome_controls.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
plan_sources:
  - user: 2026-04-13 收束 JetBrains Hybrid Shell 的 UI 事件、反射和宿主契约
  - user: 2026-04-14 UI事件系统不应该直接和slint耦合，而是独立一套调度和绑定系统
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划的首个共享 core 切片
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 把 Container / Overlay / Space 落到 retained layout core，并把 editor host pointer/scroll 输入适配到 UiSurface + UiPointerDispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-15 继续完善 shared navigation dispatcher，并把后续 editor keyboard/gamepad 入口固定到 shared core
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/Editor Event Decoupling And Replay Plan.md
  - .codex/plans/Zircon UI Editor UI Binding & Reflection Architecture.md
  - user: 2026-04-17 Source/Hierarchy/Canvas 的更强选中同步和 source roundtrip 体验
  - user: 2026-04-17 parent-specific slot/layout inspector，补 Overlay/Grid/Flow/ScrollableBox 语义
  - user: 2026-04-17 designer canvas 的可视化 authoring：插入、重排、reparent、wrap/unwrap
  - user: 2026-04-17 Bindings Inspector 的下一版：事件枚举选择、action/payload 结构化编辑
  - user: 2026-04-17 Palette 到真实节点/引用节点创建的落地
  - user: 2026-04-17 结构化 undo/redo，从当前 source-text 级别继续往 tree-command 演进
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/src/tests/ui/binding.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/viewport.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/pane.rs
  - zircon_editor/src/tests/host/slint_event_bridge.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/host/slint_window.rs
  - zircon_editor/src/tests/host/slint_drawer_resize.rs
  - zircon_editor/src/tests/host/slint_tab_drag.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/src/tests/workbench/host_events.rs
  - zircon_editor/src/tests/workbench/reflection.rs
  - cargo test -p zircon_editor --locked
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_ui --lib --locked
  - cargo test -p zircon_ui --locked
  - cargo test -p zircon_ui --offline --verbose
  - cargo test -p zircon_ui shared_core -- --nocapture
  - cargo test -p zircon_editor slint_drawer_resize -- --nocapture
  - cargo test -p zircon_editor slint_viewport_toolbar_pointer --locked
  - cargo test -p zircon_editor --lib shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi --locked -- --nocapture
  - cargo test -p zircon_editor --test workbench_autolayout -- --nocapture
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_
  - cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo
  - cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_
doc_type: module-detail
---

# UI Binding And Reflection Architecture

## Purpose

`nativeBinding`、headless 测试、反射树远控、真实宿主点击，现在都必须汇到同一套 editor shell 协议。  
这份文档只定三件事：

- 哪些 payload 是稳定 editor UI 协议
- 哪些 shell chrome 命名空间是稳定的
- 每类 payload 在 `zircon_editor` 中最终映射到什么宿主事件或状态变更

## Viewport Raw Pointer Route

viewport 外层原始输入现在也归到同一套协议思路里了，虽然它不是 template business control：

- Slint 不再向 host 暴露 7 个分散的 viewport move/down/up/wheel 直接回调
- 宿主只接收统一 `viewport_pointer_event(kind, button, x, y, delta)` pointer fact
- `app/viewport.rs` 负责把这个 fact 还原成 `UiPointerEvent`
- [`callback_dispatch/viewport/bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/bridge.rs) 与 [`callback_dispatch/viewport/pointer_dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/pointer_dispatch.rs) 再用 shared `UiSurface + UiPointerDispatcher` 产出稳定的 `EditorViewportEvent`

这一步让 viewport 原始指针输入也满足“宿主上传事实，shared dispatcher 归一化，runtime 执行 typed 事件”的统一契约，而不是继续保留一条 editor-only 的旁路输入链。

## Layer Boundary

### `zircon_ui`

`zircon_ui` 现在是共享 UI 权威层，不再只承载 binding/reflection：

- layout primitives
  - `StretchMode`
  - `AxisConstraint`
  - `ResolvedAxisConstraint`
  - `BoxConstraints`
  - `DesiredSize`
  - `LayoutBoundary`
  - `UiAxis`
  - `UiContainerKind`
  - `UiLinearBoxConfig`
  - `UiScrollState`
  - `UiScrollableBoxConfig`
  - `UiVirtualListConfig`
  - `UiPoint` / `UiSize` / `UiFrame`
  - `compute_layout_tree(...)`
- retained UI tree and invalidation
  - `UiTree`
  - `UiTreeNode`
  - `zircon_ui::tree::UiDirtyFlags`
  - `zircon_ui::tree::UiLayoutCache`
  - `UiInputPolicy`
- hit-test and surface state
  - `zircon_ui::tree::UiHitTestIndex`
  - `zircon_ui::tree::UiHitTestResult`
  - `UiSurface`
  - `UiFocusState`
  - `UiNavigationState`
  - `UiPointerEvent`
  - `UiPointerRoute`
  - `UiNavigationRoute`
  - `UiNavigationDispatcher`
  - `UiNavigationDispatchEffect`
  - `UiNavigationDispatchResult`
  - `UiPointerDispatcher`
  - `UiPointerDispatchEffect`
  - `UiPointerDispatchResult`
  - `UiVirtualListWindow`
  - `UiRenderExtract`
- binding AST
  - 现在落在 `binding/model/*` 子树，而不是单个 `model.rs`
- binding codec
- 反射树快照与 diff
- route id 注册与调用
  - 现在落在 `event_ui/manager/*` 子树，而不是单个 `manager.rs`
- transport-neutral request/subscribe control plane

这意味着 binding/reflection 只是 `zircon_ui` 的一部分。共享布局、`HorizontalBox` / `VerticalBox` / `ScrollableBox` 容器、visible-range invalidation、命中、clip 链检查、surface/render extract，以及 pointer/navigation dispatcher 权限也已经下沉到这个 crate；editor 侧只保留 docking/workbench 语义和 editor-only payload。共享布局基础的细节单独记录在 [Shared UI Core Foundation](../ui-and-layout/shared-ui-core-foundation.md)。

### `zircon_editor::ui`

`zircon_editor::ui` 负责 editor-only 协议：

- `EditorUiBinding`
- `EditorUiBindingPayload`
- `SelectionCommand`
- `AssetCommand`
- `WelcomeCommand`
- `DraftCommand`
- `DockCommand`
- `ViewportCommand`
- `InspectorFieldBatch`
- editor reflection adapter / control service

这些 typed payload 现在不再继续堆在单一 `binding.rs` 里，而是固定拆成目录化 command/codec 子树：

- `binding/asset/*`
- `binding/dock/*`
- `binding/draft/*`
- `binding/selection/*`
- `binding/viewport/*`
- `binding/welcome/*`

这次收束里，`DockCommand`、`DraftCommand`、`ViewportCommand` 的 command+codec 支撑层已经补齐，`viewport/*` 还进一步拆出 tool / transform space / projection / display / grid / orientation 这些 typed value codec 子模块。这样 `zircon_editor::ui` 的稳定协议面可以继续增长，而不需要重新把所有 editor binding 解析塞回一个大文件。

### `zircon_editor`

`zircon_editor` 现在通过 `editor_event` 子系统独占 editor 行为执行权：

- semantic event normalization
- event dispatch / execution
- transient UI state tracking
- reflection rebuild
- event journal + replay
- menu dispatch
- layout mutation
- selection propagation
- inspector batch commit
- asset open/reveal intent dispatch
- viewport input dispatch

同时，`zircon_editor::workbench::autolayout` 不再拥有底层 `AxisConstraint` / `StretchMode` / `PaneConstraints` 的定义权。它只负责把 `WorkbenchLayout`、descriptor 默认约束和 region override 映射成共享 `zircon_ui` 约束，再把求解结果投影回 editor shell frame。

`zircon_editor::ui` / `zircon_ui` 不再拥有 editor 行为闭包。  
它们只保留 typed binding、route metadata、reflection schema、codec 和 query/control primitives。

热路径现在固定为：

1. Slint / headless / MCP 输入先被适配成 `EditorEventEnvelope` 或 `EditorUiBinding`
2. `EditorEventRuntime` 归一化成 `EditorEvent`
3. 同一个 dispatcher 路径执行状态变更
4. runtime 记录 journal record
5. runtime 用 `EditorState + EditorManager + EditorTransientUiState` 重建 reflection snapshot

因此 `nativeBinding`、Slint callback 名称、route id 都只是外层 transport / adapter 输入，不再是语义层日志格式。

同一轮 manager 边界整治里，editor shell 的 descriptor catalog 和 layout host bookkeeping 也被压回结构化入口：

- `zircon_editor/src/core/host/manager/builtin_views/mod.rs` 现在只保留 builtin view catalog wiring，activity-view / activity-window descriptor construction 与 welcome / UI asset editor descriptor 拼装已经分层下沉
- `zircon_editor/src/core/host/manager/layout_hosts/mod.rs` 现在只保留 layout host wiring，active tab lookup、document host traversal、workbench root repair、builtin shell repair 不再混在同一个 manager 脚本里

当前 viewport toolbar 也已经并入这条 shared dispatch 主链：

- Slint toolbar 不再为每个按钮暴露独立 direct callback 路径，而是统一走 shared pointer 点击入口
- [`viewport_toolbar_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs) 负责 surface-local hit route 归一化
- [`callback_dispatch/shared_pointer/viewport_toolbar.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs) 与 [`callback_dispatch/viewport/toolbar_control.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/toolbar_control.rs) 把 route 转回 typed `ViewportCommand` 或 cycle/toggle 语义
- runtime journal 中记录的仍是统一 `EditorViewportEvent`，而不是某个 Slint-only callback 名称

同一轮里，transient pane surface action 也进入了 template/runtime authority：

- [`pane_surface_controls.toml`](/E:/Git/ZirconEngine/zircon_editor/ui/templates/pane_surface_controls.toml) 定义 builtin `PaneSurface/TriggerAction`
- [`callback_dispatch/pane/surface_control.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs) 用 `BuiltinPaneSurfaceTemplateBridge` 把 `control_id + action_id` 重组回 canonical `MenuAction`
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 不再暴露 root `menu_action(action_id)` callback；Scene/Game empty-state 和 Project overview 的 `Open Assets` 现在只上传 generic `pane_surface_control_clicked(control_id, action_id)`

这意味着 `PaneActionModel` 虽然仍然携带 `menu_action_binding(...)`，但 Slint compatibility shell 不再把这类动作直接当 handwritten callback ABI 透传出去。

当前有一条刻意保留的例外 seam：

- `WelcomeCommand`

它现在已经拥有稳定 `WelcomeSurface/*` binding 命名空间和统一 `dispatch_welcome_binding(...)` 解析，但仍停在 host-owned `WelcomeHostEvent`，没有被伪装成 runtime 已接管的 `EditorEvent`。原因是 `EditorStartupSessionDocument`、recent project 验证、create/open project 流程以及 exclusive welcome page 生命周期还由 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 和 [`startup/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/startup/mod.rs) 持有。当前架构边界是：

- 模板 authority 决定 welcome control 语义
- 宿主 authority 决定 startup session 执行与页面生命周期

## Editor Event Runtime

`zircon_editor/src/core/editor_event/` 当前包含：

- `types.rs`
  - 定义 canonical `EditorEvent`、`EditorDraftEvent`、`EditorEventRecord`、`EditorEventResult`、`EditorEventUndoPolicy`
- `runtime.rs`
  - `EditorEventRuntime` / `EditorEventDispatcher`
  - 统一拦截 `InvokeBinding`、`InvokeRoute`、`CallAction`
- `transient.rs`
  - hover / focus / pressed / drawer resize / drag projection
- `journal.rs`
  - session-local event record 存储
- `replay.rs`
  - recorded `EditorEvent` 重新走同一 dispatcher path
- `host_adapter.rs`
  - Slint / headless 输入到 normalized event envelope 的薄适配器

当前 canonical log record 固定保存：

- `event_id`
- `sequence`
- `source`
- normalized `EditorEvent`
- `before_revision` / `after_revision`
- emitted effects
- undo policy
- structured success / failure result

## Slint Host Adapter Path

桌面宿主现在也必须遵守和 headless / reflection 相同的 runtime authority：

- `zircon_editor/src/ui/slint_host/app.rs`
  - `SlintEditorHost` 持有 `EditorEventRuntime`
  - 大多数 `ui.on_*` callback 只负责采集桌面输入、必要的瞬态宿主 bookkeeping、提交 dispatch、消费 effect
  - 不再直接持有 editor 行为语义，也不再在 callback 里直接改 `EditorState` / `EditorManager`
- `zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs`
  - 把 raw Slint callback 输入收敛成 `EditorEventEnvelope` 或直接语义化 `LayoutCommand`
  - 统一走 `runtime.dispatch_envelope(...)`
  - viewport pointer/scroll 现在先经过 `SharedViewportPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`，再映射成 `EditorViewportEvent`
  - `AssetSurface/*` 已经通过 builtin template bridge 直接落到 typed runtime dispatch
  - `WelcomeSurface/*` 已经通过 builtin template bridge 落到 typed `WelcomeHostEvent`，再由宿主执行 startup session 逻辑
- `zircon_editor/src/ui/slint_host/event_bridge.rs`
  - 把 `EditorEventRecord.effects` 映射成宿主展示侧需要的 `SlintDispatchEffects`
  - 宿主只消费 `presentation_dirty / layout_dirty / render_dirty / sync_asset_workspace` 之类的结果，不重新解释语义
- `zircon_editor/src/ui/slint_host/drawer_resize.rs`
  - 把 splitter group 级别的桌面手势换成 `LayoutCommand::SetDrawerExtent`
  - `begin_drawer_resize(...)` 现在只上传 pointer 坐标；真正的 `left / right / bottom` target group 由统一 shell pointer bridge 在 shared `UiSurface + UiPointerDispatcher` 上解析
- `zircon_editor/src/ui/slint_host/shell_pointer.rs`
  - `WorkbenchShellPointerBridge` 把 shell drag target 与 splitter target 收口到同一棵 `UiSurface`
  - drag route 与 resize route 共用 hit-test / clip / capture 状态，但通过两个 mode-specific dispatcher 解释 editor-only 语义
- `zircon_editor/src/ui/slint_host/tab_drag.rs`
  - tab drop 继续把 resolved group 翻译成 editor-only `ViewHost`

保留在宿主侧但不进入当前 canonical event catalog 的内容仍然只有 adapter 级职责：

- raw `InputManager` pointer / keyboard forwarding
- shell geometry 和 viewport texture bridge
- asset/resource polling refresh
- startup draft / recent project 生命周期

随着 `zircon_ui` 已经拥有 `ScrollableBox` 和第一版 `UiPointerDispatcher`，后续宿主接线的目标也更明确：

- 宿主先把 pointer / scroll 输入适配成 shared `UiPointerEvent`
- 共享 `UiSurface + UiPointerDispatcher` 先完成 route、stacked target 语义、capture 和默认 scroll container 响应
- keyboard / gamepad 输入后续也必须先适配成 `UiNavigationEventKind`，再由 `UiSurface::dispatch_navigation_event(...)` 走 shared `UiNavigationDispatcher`
- shared navigation fallback 现在先于宿主生效：`Next` / `Previous` 走共享 tab order，`Right` / `Down` 在无焦点时从首个 focusable 节点开始，`Left` / `Up` 从末尾 focusable 节点开始，`Activate` / `Cancel` 不做隐式焦点跳转
- 只有 editor-only 的 docking / menu / inspector / viewport payload 再继续上送到 `EditorEventRuntime`

这一层现在已经开始进入真实接线阶段，但范围仍然是刻意收窄的：

- viewport pointer/scroll callback 已经改成先走 shared dispatcher
- menu、hierarchy、asset、drawer resize、tab drag 仍然保持各自的薄 adapter，不把 editor-only payload 混进 `zircon_ui`
- 更细粒度的 docking transient 仍然属于后续迁移面，但 group 级别 shell pointer hit-test / dock target route 已经进一步收口到统一 shell bridge

这一条“后续迁移面”现在已经向前推进了一段：

- workbench tab drag 的 `left / right / bottom / document` target route 不再由 `workbench.slint` 本地 `drag_target_group` 公式决定
- Slint shell 新增 `update_drag_target(x, y)` callback，只把 pointer 位置交回 host
- host 通过 `WorkbenchShellPointerBridge` 在 shared `UiSurface` 上同时维护 drag target retained 节点和 splitter target retained 节点
- overlap 区域通过 `UiPointerDispatchEffect::{Handled, Passthrough}` 决定 side/bottom 归属
- splitter `Down` 会在 shared dispatcher 里触发 capture，后续 `Move / Up` 即使移出 splitter hit bounds 也继续路由到同一 target
- Slint 现在只消费 host 写回的 `active_drag_target_group`，以及 host 内部基于同一 shell bridge 解析出的 resize group

这意味着 editor shell 的 dock target hit-test 已经开始进入 shared-core-first 路线，而不是继续把 pointer route 留在宿主 UI 壳里。

## Stable Shell Chrome Namespaces

这一轮固定的 shell chrome 命名空间如下：

- `WorkbenchMenuBar/*`
- `ActivityRail/*`
- `ToolWindow/*`
- `DocumentTabs/*`
- `InspectorView/*`
- `ViewportToolbar/*`
- `StatusBar/*`
- `AssetSurface/*`
- `WelcomeSurface/*`
- `PaneSurface/*`

这些名字的作用不是替代 activity instance id，而是给壳层 chrome 提供稳定协议面。  
例如：

- `WorkbenchMenuBar/OpenProject`
- `ActivityRail/ProjectToggle`
- `InspectorView/ApplyBatchButton`

## Editor Payload Surface

`zircon_editor::EditorUiBindingPayload` 当前固定包含：

- `PositionOfTrackAndFrame`
- `MenuAction`
- `SelectionCommand`
- `AssetCommand`
- `WelcomeCommand`
- `DraftCommand`
- `DockCommand`
- `ViewportCommand`
- `InspectorFieldBatch`
- `Custom`

其中 `MenuAction` 和 `InspectorFieldBatch` 继续保留原形态；workbench shell 相关的新行为全部进入 typed command family，而不是继续堆 stringly-typed custom payload。

### Draft / Live Edit

`DraftCommand` 当前承担“只更新 live snapshot、不立即触发持久化执行”的编辑语义。已经落地的 typed draft 面有两类：

- `DraftCommand::SetInspectorField { subject_path, field_id, value }`
- `DraftCommand::SetMeshImportPath { value }`

它们统一归一化成 canonical `EditorEvent::Draft(EditorDraftEvent::...)`，并且遵守以下边界：

- draft 只更新 runtime/editor snapshot 与 reflection surface
- draft 不会隐式触发 `ApplyInspectorChanges`
- draft 不会隐式触发 mesh import、asset sync 或 render side effect
- draft 属于 `NonUndoable`，不会混入 editor history 的持久化命令语义

## Workbench Shell Event Contract

### Menu

- source: `WorkbenchMenuBar/*`
- payload: `MenuAction`
- dispatch entry: `dispatch_workbench_binding`
- result: `WorkbenchHostEvent::Menu`

menu 继续承载：

- project open/save
- layout save/reset
- undo/redo
- create node
- open view

其中 `MenuAction::OpenProject` 现在不再直接在 runtime 里假定“已有 project path 并立即打开项目”。当前语义是：

- runtime 记录 canonical menu event
- runtime 发出 `EditorEventEffect::PresentWelcomeRequested`
- `SlintDispatchEffects` 把它翻译成 host-level present-welcome 标志
- 宿主再调用现有 `present_welcome_surface(...)`

这样 `WorkbenchMenuBar/OpenProject` 已经回到了统一 template/menu/event 链，而 startup session 的执行权仍保持在宿主。

同一条规则现在也明确约束 `WorkbenchMenuBar/ResetLayout`：即使 builtin template 最终会触发 layout reset，它在 journal 里的 canonical event 仍然必须是 `MenuAction::ResetLayout`，而不是 template bridge 自己提前改写成 `LayoutCommand::ResetToDefault`。这样旧手写 Slint 菜单入口、reflection/nativeBinding 菜单入口、以及新的 template-host 菜单入口，才能在 same-fixture parity 测试里产出同构 `EditorEventRecord`。

### Docking And Tool Window Shell

- source: `ActivityRail/*`, `DocumentTabs/*`, `ToolWindow/*`
- payload: `DockCommand`
- dispatch entry: `dispatch_docking_binding`
- result: `LayoutCommand`

`DockCommand` 当前覆盖：

- focus/close view
- attach view to drawer/document
- detach to window
- activate drawer tab
- activate main page
- set drawer mode
- set drawer extent
- save/load preset
- reset to default

rail click、drawer tab 激活、stack 展开/折叠都应继续走这条 typed docking path。

当前 runtime 里，Slint `Window` 菜单上的 preset 条目仍通过宿主菜单回调进入 `LayoutCommand::SavePreset/LoadPreset`，但稳定协议层已经把它们定成 `DockCommand::SavePreset/LoadPreset`。这保证了 headless / reflection / nativeBinding 可以直接走 typed contract，而不是复制一套字符串分派。
当前 runtime 里，Slint `Window` 菜单上的 preset 条目仍来自 legacy menu callback，因为 builtin template 还不会实例化每个动态 preset item；但字符串归一化已经不再停在 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs)。现在的真实链路是：

- `dispatch_menu_action(...)`
  -> [`host_adapter::slint_menu_action(...)`](/E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/host_adapter.rs)
  -> `EditorEvent::Layout(LayoutCommand::SavePreset/LoadPreset)`
  -> `SlintDispatchEffects.active_layout_preset_name`
  -> 宿主只消费 effect，更新当前 preset 选择

因此这里保留下来的只是动态菜单项来源，而不是 `app.rs` 的本地字符串 special-case。稳定协议层仍然是 `DockCommand::SavePreset/LoadPreset`，headless / reflection / nativeBinding 仍可直接走 typed contract，而不用复制一套前缀解析。

同一轮里，scene 空态动作也从宿主 label fallback 收进了同一条 menu/runtime 链：

- `PaneActionModel("Open Scene" / "Create Scene")` 现在直接携带 `menu_action_binding(&MenuAction::OpenScene/CreateScene)`
- `slint_host/ui.rs` 不再需要根据按钮文案硬编码 `"OpenScene"` / `"CreateScene"`
- `dispatch_menu_action(...)` 会把它们和其他 menu action 一样送进 `EditorEventRuntime`
- 当前 runtime 仍只返回占位状态 `"Scene open/create workflow is not wired yet"`，但占位语义已经在 runtime，而不是 `app.rs`

当前真实 Slint tab drag/drop 仍是宿主内部回调，不是反射公开协议：

- Slint tab 释放时先落成内部 `drop_tab(tab_id, target_group)` 回调
- `target_group` 只表达 shell 级别的 `left / right / bottom / document`
- `target_group` 的来源现在是 host-owned `active_drag_target_group`，不再是 Slint 自己算出来的字符串属性
- 宿主再把它映射到现有 `AttachView` / 条件性 `SetDrawerMode(Pinned)` 语义

这样做的原因是：pointer drag 本身是本地桌面手势，不适合作为当前这轮的远控协议表面；真正对外稳定的 attach 语义仍然由 `DockCommand::AttachViewToDrawer` 和 `DockCommand::AttachViewToDocument` 承担。

这里现在多了一条明确的宿主规范：`dispatch_tab_drop(...)` 只会在目标 drawer 当前为 `Collapsed` 时补发 `SetDrawerMode(Pinned)`；如果目标 drawer 已经 `Pinned` 或 `AutoHide`，route dispatcher 只记录 `AttachView` 并保留现有模式。这样 shared pointer route -> normalized drop route -> typed layout dispatch 的 journal 不会因为宿主兼容层而多出冗余 reopen event。

同理，当前 Slint splitters 也先走宿主内部回调：

- `set_drawer_extent(target_group, extent)` 只表达 shell 级别的 `left / right / bottom`
- 宿主把 group fan-out 到对应 drawer slots，再落成 `LayoutCommand::SetDrawerExtent`
- 稳定协议层继续把这类行为定义为 `DockCommand::SetDrawerExtent`，而不是公开桌面手势细节

这样可以把高频 pointer resize 保留在本地宿主里，同时让 headless / reflection / binding roundtrip 仍然对齐同一个 typed docking 语义。

### Selection Sync

- source: hierarchy / scene related controls
- payload: `SelectionCommand`
- dispatch entry: `dispatch_selection_binding`
- apply entry: `apply_selection_binding`
- result: `SelectionHostEvent` or `EditorIntent::SelectNode`

当前已经落地的 typed selection command 是：

- `SelectionCommand::SelectSceneNode`

这条链路的意义是：层级树点击、viewport 点击、headless 绑定调用，最终都收敛到同一条 selection intent，而不是一个地方直接改状态、另一个地方发字符串消息。

### Asset Intent

- source: project/assets related controls
- payload: `AssetCommand`
- dispatch entry: `dispatch_asset_binding`
- result: `AssetHostEvent`

当前已经落地的 typed asset command 是：

- `AssetCommand::OpenAsset`
- `AssetCommand::ImportModel`

这里先锁协议，不强行在这一轮补完完整 asset browser 逻辑。关键约束是：asset open/reveal/import 不能继续走匿名字符串解析。
其中 `AssetCommand::ImportModel` 当前明确走的是“runtime 归一化 + host effect 请求”边界：

- runtime 归一化成 canonical `EditorEvent::Asset(EditorAssetEvent::ImportModel)`
- runtime 只产出 `EditorEventEffect::ImportModelRequested`
- 真正的文件复制、asset import、resource resolve 和 mesh 注入仍然由宿主 effect 消费侧执行

### Welcome Startup Intent

- source: `WelcomeSurface/*`
- payload: `WelcomeCommand`
- dispatch entry: `dispatch_welcome_binding`
- result: `WelcomeHostEvent`

当前 welcome typed command 覆盖：

- project name edit
- location edit
- create project
- open existing project
- open/remove recent project

这一层当前刻意停在 host event，而不是继续伪装成 `EditorEvent`。原因不是协议还没成形，而是 startup session 的执行权还在宿主：

- `WelcomeCommand` 负责稳定 surface 协议
- `WelcomeHostEvent` 负责 typed host dispatch
- `SlintEditorHost` 负责 `EditorStartupSessionDocument` 生命周期与 `EditorManager` 调用

### Inspector Commit

- source: `InspectorView/ApplyBatchButton`
- payload: `InspectorFieldBatch`
- dispatch entry: `dispatch_inspector_binding`
- apply entry: `apply_inspector_binding`

`InspectorFieldBatch` 仍然是 inspector 唯一允许的持久化属性编辑入口。  
selection 改变可以刷新 inspector subject，但真正提交属性改动仍然只允许这一条 batch path。

### Viewport

- source: viewport surface / viewport toolbar
- payload: `ViewportCommand`
- dispatch entry: `dispatch_viewport_binding`
- apply entry: `apply_viewport_binding`
- result: `ViewportInput` or viewport feedback

当前已经落地的 typed viewport path 覆盖：

- pointer move
- left/right/middle press/release
- scroll
- resize

当前桌面宿主里的 viewport pointer/scroll 已经不是直接由 Slint callback 手写映射成 `EditorViewportEvent`：

- Slint callback 先生成 shared `UiPointerEvent`
- `SharedViewportPointerBridge` 在最小 retained `UiSurface` 上完成 target、capture 和 route 派发
- 只有命中 viewport 的结果才继续映射成 editor runtime 的 `EditorViewportEvent`
- release 会复用当前 capture / cursor 状态，即使光标已移出 viewport hit bounds 也不会丢失 `Up`
- callback dispatch 还补齐了 `dispatch_viewport_command(...)` / `viewport_event_from_command(...)` 这条 typed helper path，保证 toolbar/state 级别的 `ViewportCommand` 也会落回同一套 runtime 语义

viewport toolbar 的 typed command 空间已经固定属于 `ViewportCommand`，即使某些具体 toolbar action 还没完全接入 runtime state，也不能再退回 `Custom("TranslateTool")` 这种匿名协议。

当前 runtime 还承担了 viewport gizmo drag 的 editor 语义状态：

- begin / drag / end 仍由桌面宿主采集 pointer 输入
- 是否进入 gizmo drag 由 runtime 内部 `dragging_gizmo` bookkeeping 决定
- scene 变换、render dirtiness、journal record 都继续走统一 dispatcher path

## Reflection Contract

`zircon_editor/src/ui/workbench/reflection/mod.rs` 现在作为结构入口，把 workbench snapshot 和 view model 投影拆成 activity descriptor、activity collection、typed route registration 与名称映射几个独立子模块。

反射树中固定暴露：

- menu item node
- page node
- drawer node
- floating window node
- activity node

activity node 上暴露的动作必须来自 typed payload：

- menu item -> `MenuAction`
- focus/detach -> `DockCommand`
- inspector apply -> `InspectorFieldBatch`
- inspector live edit -> `DraftCommand.SetInspectorField`
- assets mesh import path edit -> `DraftCommand.SetMeshImportPath`
- assets import submit -> `AssetCommand.ImportModel`
- scene/game pointer actions -> `ViewportCommand`

远控现在可以通过 `CallAction`、`InvokeRoute` 或 `InvokeBinding` 进入这些动作，但执行不再发生在 `EditorUiControlService` 闭包里。  
`register_workbench_reflection_routes` 只注册 stub route metadata，真正执行统一回到 `EditorEventRuntime::handle_control_request(...)`。

## Reflection Rebuild Inputs

runtime 每次事件执行后只允许从三类输入重建 reflection：

- `EditorState` 的稳定 editor 数据快照
- `EditorManager` 的 workbench/layout/view 实例快照
- `EditorTransientUiState` 的 hover/focus/pressed/resizing/drag 状态

这意味着：

- 反射树不再依赖 live Slint tree 查询 transient state
- `CallAction` / `InvokeBinding` 返回的结果来自真实 editor 行为，而不是 preview JSON
- replay session 可以重建同一条 reflection surface，而不是只能回放一堆 widget callback 名字

## Hot-Path Rule

实现约束保持不变：

- 正常 UI 热路径不依赖字符串 parse
- route id 和 `nativeBinding` 最终进入同一 handler
- string formatting/parse 只保留给稳定协议、测试和远控

这也是为什么 shell 相关行为需要拆成 `DockCommand / SelectionCommand / AssetCommand / ViewportCommand`：它们既能提供稳定 wire format，又不会逼迫宿主在高频路径上反复做字符串分派。

## UI Asset Editor Structured Authoring Route

`UI Asset Editor` 这一轮也被钉在同样的 typed session/host seam 上，而不是回退到字符串 callback：

- [`UiDesignerSelectionModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/reflection.rs) 现在是 Source、Hierarchy、Canvas 共享的 selection payload；[`reconcile_selection(...)`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/session.rs) 会在 parse/tree edit 后按稳定 `node_id` 重建 parent、mount 和 sibling multi-select
- [`UiAssetEditorCommand::TreeEdit`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/command.rs) 已支持附带 `next_selection`；[`UiAssetEditorUndoStack`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/undo_stack.rs) 记录的是 `source + selection` snapshot，因此 undo/redo 恢复的是结构化 authoring state，而不只是 source text
- [`binding_inspector.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/binding_inspector.rs) 现在把 `UiBindingRef` 投影成 `event kind + action kind + payload entries` 三段式 inspector；事件来自 `UiEventKind`，动作和 payload 来自 `UiActionRef`
- manager 和 Slint host 只转发 palette index、binding index、payload key 以及 `canvas.reparent.*` / `palette.insert.*` 这类稳定 action id；真正的 AST 变更始终由 [`UiAssetEditorSession`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/session.rs) 执行
- Source 模式和 Design 模式仍共用同一条 session authority：parse 成功才刷新 preview/reflection，失败时保留 last-good preview 和 roundtrip diagnostics

## Current Coverage

当前已经有自动化覆盖的点：

- `EditorUiBindingPayload` roundtrip
  - `MenuAction`
  - `InspectorFieldBatch`
  - `SelectionCommand`
  - `AssetCommand`
  - `WelcomeCommand`
  - `DraftCommand`
  - `DockCommand`
  - `ViewportCommand`
- representative shell bindings
  - `WorkbenchMenuBar/OpenProject`
  - `ActivityRail/ProjectToggle`
  - `InspectorView/ApplyBatchButton`
- `editor_event` runtime normalization equivalence
  - Slint adapter input
  - `EditorUiBinding`
  - reflection `CallAction`
- live draft dispatch
  - inspector field draft
  - mesh import path draft
- host-effect asset dispatch
  - import model request
- session-local journal serialization / replay
- transient reflection projection
  - hovered
  - focused
  - pressed
  - drawer resizing
- `zircon_editor` host dispatch
  - menu dispatch
  - selection dispatch + apply
  - asset dispatch
  - docking dispatch
  - inspector dispatch + apply
  - viewport dispatch + apply
- desktop Slint callback adapters
  - menu action
  - hierarchy selection
  - asset search
  - viewport pointer/scroll 先走 shared `UiSurface + UiPointerDispatcher` bridge
  - workbench shell pointer route 先走 host-owned `WorkbenchShellPointerBridge`
  - host page activation 与 welcome surface builtin template dispatch 已补 same-fixture parity
- Slint effect bridge
  - `EditorEventRecord.effects` -> `SlintDispatchEffects`
  - layout / render / presentation / asset sync fan-out
- workbench reflection route registration
  - menu
  - docking
  - inspector
  - viewport
  - runtime-backed `CallAction`
- UI asset editor session/host routes
  - source/hierarchy/canvas selection roundtrip
  - structured binding event/action/payload editing
  - palette insert / wrap / unwrap / reparent / outdent
  - source-selected block projection and undo/redo selection snapshots

## Validation Status

这次文档同步时重新跑过的验证命令：

- `cargo test -p zircon_ui shared_core -- --nocapture`
- `cargo test -p zircon_ui --locked`
- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor draft_command_bindings_parse_into_typed_payloads_instead_of_custom_calls --locked`
- `cargo test -p zircon_editor --lib draft_inspector_binding_normalizes_and_updates_live_snapshot --locked`
- `cargo test -p zircon_editor --lib draft_mesh_import_path_binding_normalizes_and_updates_live_snapshot --locked`
- `cargo test -p zircon_editor --lib inspector_draft_field_dispatch_updates_live_snapshot_without_scene_side_effects --locked`
- `cargo test -p zircon_editor --lib mesh_import_path_edit_dispatch_updates_live_snapshot_without_backend_sync --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_model_projects_menu_and_activity_descriptors --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_routes_mark_activity_actions_as_remotely_callable --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_call_action_dispatches_typed_draft_actions --locked`
- `cargo test -p zircon_editor --lib asset_command_binding_roundtrips_for_import_model --locked`
- `cargo test -p zircon_editor --lib asset_import_binding_normalizes_to_runtime_host_request --locked`
- `cargo test -p zircon_editor --lib builtin_asset_surface_import_model_dispatches_host_request_from_template --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_call_action_dispatches_asset_import_action --locked`

后续在同一工作区继续推进后，又额外通过了：

- `cargo test -p zircon_editor --locked`
- `cargo test --workspace --locked`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_`
- `cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo`
- `cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_`

当前重新确认或由现有测试树持续覆盖的关键点包括：

- `UiNavigationDispatcher` 已经能从 focused route 冒泡，并在未聚焦时回退 root handler
- navigation handler 返回 `Focus(UiNodeId)` 时，会把 focus handoff 收口回 shared `UiSurface`
- unhandled navigation 在 shared core 上已经有 canonical fallback，不再要求 editor host 自己维护 tab order 或无焦点方向键起点
- viewport pointer/scroll callback 已经先经过 shared `UiSurface + UiPointerDispatcher`
- shared pointer capture 后移出 viewport hit bounds 仍会把 `Move` / `Up` 派回 viewport
- workbench shell drag target route 已经先经过 shared `UiSurface + UiPointerDispatcher`
- workbench shell splitter route 现在也先经过 `WorkbenchShellPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`
- `UI Asset Editor` 的 Source、Hierarchy、Canvas 选中现在通过稳定 `node_id` 和 `UiDesignerSelectionModel` 做同源恢复
- `UI Asset Editor` 的 bindings inspector 现在走 `UiEventKind + UiActionRef + payload` 的结构化编辑，而不是宿主私有 callback 字符串
- tree-command undo/redo 现在会一起恢复 source block 定位与 inspector subject
- Slint `workbench.slint` 不再本地拥有 `drag_target_group` 公式，只保留 `active_drag_target_group` 展示态和 `update_drag_target(...)` 回调
- Slint callback adapter 通过 runtime dispatch 执行
- `CallAction` / `InvokeBinding` 继续命中真实 editor behavior
- global default layout 的 empty skeleton 现在会在 bootstrap 时被接受，并补回 builtin shell view placement
- `drawer_resize.rs` 与 `callback_dispatch.rs` 的测试 helper 只在 `cfg(test)` 下编译，不再给工作区构建引入死代码告警

## Remaining Work

这一轮已经完成了主线桌面宿主接线：

- `slint_host/app.rs` 的 menu / docking / hierarchy / inspector / asset / viewport callback 已经改成 adapter + runtime dispatch
- `InvokeBinding` / `CallAction` / Slint callback 现在可以对齐到同一个 normalized event path
- reflection rebuild 输入已经收敛到 `EditorState + EditorManager + EditorTransientUiState`

剩下的工作主要是扩展 catalog，而不是继续拆 ownership：

- viewport toolbar / gizmo 的更多 typed action 并入 normalized event catalog
- asset browser / project explorer 的更完整 action catalog
- MCP / network transport adapter 直接消费 runtime journal 和 reflection surface
- 非 scene editor-domain undo 继续在当前 journal / undo metadata 上扩展

也就是说，这一轮已经把“谁拥有执行权”“日志记录什么”“反射动作如何执行”定死了；后续主要是把更多 editor 行为填进同一 dispatcher，而不是再把语义退回 UI 库。
