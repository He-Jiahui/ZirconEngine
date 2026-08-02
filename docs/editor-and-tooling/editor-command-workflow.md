---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/commandlet/runner.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/commandlet/runner.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/core/commands/menu_model.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/component_drawer.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/world/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/core/editing/intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/component_drawer.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_runtime/src/scene/world/mod.rs
plan_sources:
  - user: 2026-04-12 扩展 editor 命令系统到删除节点、改父子层级、重命名和 inspector 字段批量提交
  - user: 2026-04-12 将 undo/redo 从整世界快照推进到真正的 EditorCommand/UndoableStack 命令化实现
  - user: 2026-04-12 实现 Zircon Editor Workbench Shell V1
  - user: 2026-05-02 Unity 式编辑器优先补齐计划：Inspector / Inspector Customization 接入 Undo/Redo
  - user: 2026-05-03 Full Milestone Android/iOS/WebGPU/WASM hosts and Inspector Customization templates
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - .cursor/plans/基本路线图.md
  - .codex/plans/ZirconEngine Unity 式编辑器优先补齐计划.md
tests:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/import.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/core/commandlet/tests.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - cargo test -p zircon_editor --lib commandlet --locked
  - cargo test -p zircon_app --locked cli
  - zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs
  - cargo test -p zircon_editor -- --nocapture
  - cargo test -p zircon_app -- --nocapture
  - cargo test -p zircon_editor --lib inspector_binding_applies_dynamic_plugin_component_fields_with_undo_history --locked --jobs 1
  - cargo test -p zircon_runtime --lib dynamic_plugin_component_instances_report_schema_when_loaded_and_protect_when_missing --locked --jobs 1
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe builtin_viewport_toolbar_play_buttons_dispatch_menu_play_mode_operations --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe menu_action_dispatches_through_runtime_and_sets_scene_dirty_effects --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe editor_operation_registry_exposes_builtin_menu_operations_by_path --nocapture (passed)
  - cargo test -p zircon_editor --lib workbench_window_menu_exposes_unreal_style_functional_windows --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib editor_operation_registry_exposes_builtin_menu_operations_by_path --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - pending: cargo test -p zircon_editor --lib editor_runtime_exposes_plugin_inspector_customization_surface_for_inspector_lookup --locked --jobs 1
  - pending: cargo test -p zircon_editor --lib editor_snapshot_resolves_enabled_inspector_customization_for_selected_dynamic_component --locked --jobs 1
  - pending: cargo test -p zircon_editor --lib editor_runtime_rejects_inspector_customization_bindings_to_missing_operations --locked --jobs 1
  - 2026-05-04: cargo test -p zircon_editor inspector_customization_adapter_accepts_safe_action_events_beyond_press --locked --jobs 1 --target-dir target-codex-editor-check --message-format short --color never (passed; existing warnings only)
  - 2026-05-13: RUSTFLAGS="-C linker=rust-lld" cargo test -p zircon_editor --lib editor_extension_registry_rejects_legacy_ui_template_documents --jobs 1 (passed, 1 passed)
  - current contract: cargo test -p zircon_editor --lib editor_extension_registry_rejects_non_zui_inspector_customization_documents --locked
doc_type: module-detail
---

# Editor Command Workflow

## Purpose

`zircon_editor` 不再把整个 ECS 世界直接塞进 undo/redo 快照，而是通过 `EditorCommand` 把一次编辑收束为可应用、可撤销、可重做的命令对象。这样 editor UI 可以保持 retained host 职责，只负责维护草稿字段和触发 intent；真正的世界修改则统一通过命令层进入 `zircon_runtime::scene::LevelSystem` 所托管的 `zircon_scene::Scene`。

这一设计直接服务当前路线图里的两个目标：

- editor 对 ECS 世界的修改必须可逆，并且不能把整世界快照当作长期主路径
- inspector、gizmo、场景树这些不同入口触发的编辑，需要落到同一套 undoable 行为模型

## Related Files

- `zircon_editor/src/core/editing/command.rs`: 命令类型、创建/删除/更新节点逻辑
- `zircon_editor/src/core/editor_operation.rs`: menu/toolbar/editor 插件 operation descriptor registry
- `zircon_editor/src/core/editing/engine/transaction.rs`: `EditorTransactionEngine`、`TransactionScope`、命令 apply/rollback 与 transaction commit owner
- `zircon_editor/src/core/editing/engine/history.rs`: `HistoryStore`、transaction records 与按 `HistoryContextId` 分区的 undo/redo 状态
- `zircon_editor/src/ui/workbench/state/editor_state_viewport.rs`: `GizmoTransactionCapture` 与 drag step 命令捕获；结束拖拽时交给 `EditorTransactionEngine` 收束为同一事务模型
- `zircon_editor/src/core/editing/intent.rs`: headless `EditorIntent` 声明 owner；具体执行与 inspector 草稿态由上方列出的 workbench state 模块分别持有
- `zircon_editor/src/ui/retained_host/app.rs`: 统一接住项目保存/加载和多窗口 workbench 宿主消息，再驱动命令执行
- `zircon_editor/src/ui/host/editor_manager.rs`: 提供布局、view registry、项目 workspace 的 editor 域协调入口
- `zircon_editor/src/ui/workbench/project/mod.rs`: editor project/workspace sidecar 与 level 文档桥接
- `zircon_editor/src/ui/workbench/snapshot/mod.rs`: workbench 与资产面板投影快照
- `zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs`: template binding 的 operation-first 分派入口
- `zircon_editor/src/ui/retained_host/callback_dispatch/workbench/menu_action.rs`: workbench menu action 到 `EditorOperation` 的桥接
- `zircon_runtime/src/scene/world/mod.rs`: 世界层约束，如最后一个 camera 不可删、层级不可成环

## Behavior Model

当前命令模型包含五类命令：

- `CreateNodeCommand`
  - 处理普通节点创建和外部 mesh 导入
  - 记录新节点完整 `NodeRecord` 与之前的选择状态
  - `undo` 时删除创建结果，`apply`/`redo` 时按同一记录重建节点
- `DeleteNodeCommand`
  - 以子树为单位删除
  - 记录整棵子树的 `NodeRecord` 列表、删除前选中节点、删除前活动相机，以及删除后应落到的 selection
  - 如果待删集合会移除世界里最后一个 camera，直接拒绝执行
- `UpdateNodeCommand`
  - 统一承载重命名、改父子层级、改 transform 和 inspector 批量提交
  - 记录 `before` / `after` 的 `NodeEditState`
  - `NodeEditState` 目前固定包含 `name`、`parent`、`transform`
- `SetScenePropertyCommand`
  - 通过 runtime scene `ComponentPropertyPath` 写入单个组件属性，当前主要服务插件动态组件 Inspector 字段
  - 捕获属性写入前后的 `ScenePropertyValue`，因此动态组件字段也能进入 undo/redo
- `BatchEditorCommand`
  - 把一次 Inspector Apply 产生的节点更新和多个插件组件属性更新合并成一个历史步
  - `apply` 顺序执行，`undo` 反向执行，selection 使用最后一个子命令的结果

`EditorState::apply_intent` 是 editor 侧权威入口。它不直接绕过命令层修改世界，除了 gizmo 拖拽中的中间帧预览。即便是 inspector 的 Apply，也会先组装成 `UpdateNodeCommand`、`SetScenePropertyCommand` 或二者的 batch，再进入历史栈。

## Control Flow

### Inspector Customization Surface Projection

插件贡献的 `InspectorCustomizationDescriptor` carries the target component type, UI document, controller, optional template id, optional data root, and validated operation bindings as descriptor metadata. The UI document must be a `.zui` component asset; the contribution batch rejects stale `.ui.toml` and `.v2.ui.toml` customization/template documents before they can enter Inspector projection. The live editor runtime filters these customizations through the current capability snapshot before lookup and before `EditorState::snapshot_with_inspector_customizations(...)` builds Inspector data, so disabled plugin capabilities cannot surface custom Inspector controls.

The editor extension contract tests keep that authority aligned: `zircon_editor/src/tests/editor_authoring_extension_descriptors.rs` and `zircon_editor/src/tests/editor_event/runtime/extensions_validation.rs` exercise `register_ui_template` and `register_inspector_customization` without accepting retired `.ui.toml` / `.v2.ui.toml` suffixes as active registrations.

Inspector snapshots and pane payloads preserve inspector customization surface metadata separately from generic dynamic component properties. When a runtime component schema and an enabled customization are both present, the snapshot carries the customization UI document, controller, template id, data root, and operation binding ids into the pane payload. The retained host-contract projection annotates the component header with the template id and UI document so the host can route the row as a custom inspector surface. When the plugin schema or enabled customization is unavailable, `customization_available` remains false, property rows stay disabled, and the warning diagnostic protects serialized component data until the plugin reloads or the required editor capability is enabled.

Inspector customization surface execution is host-mediated rather than native plugin UI embedding. A customization control dispatches a `UiComponentEventEnvelope` targeting the retained host's `component_drawer` event domain, with the dynamic component type in `subject` and the requested editor operation path in `path`. `EditorEventRuntime::dispatch_ui_component_adapter_event(...)` resolves only enabled customizations, rejects operations not declared in the customization bindings, rejects draft-edit events such as `ValueChanged`, and accepts only safe action-style events such as pressed buttons, committed fields, selected options, expansion toggles, and reference navigation actions. Accepted envelopes then invoke the existing `EditorOperation` dispatcher as `UiBinding`. The operation registry still enforces missing handlers, disabled capabilities, undo metadata, and journal recording. The retained adapter returns a component-adapter result for projection refresh, but the mutation authority remains the normal editor operation/command path.

### EditorOperation 分派

1. Workbench menu item、toolbar button 或 builtin template binding 先携带 stable action id 进入 retained host dispatcher
2. `operation_path_for_menu_action(...)` 把内置 menu action 映射到 `EditorOperationDescriptor` 路径，例如 `scene.node.create_cube`、`runtime.play_mode.enter`、`runtime.play_mode.exit`、`view.build_export.open`
3. `dispatch_editor_binding(...)` 和 `dispatch_menu_action(...)` 优先调用 `EditorEventRuntime::invoke_operation(...)`
4. operation registry 根据 capability snapshot 和 descriptor 决定该命令是否可见、可调用，以及是否声明 undoable
5. 真正修改场景的 operation 继续进入 `EditorState::apply_intent` 和 `EditorCommand`；播放模式和窗口打开这类不可撤销命令则停在 editor event/runtime 边界处理副作用

这条路径让插件菜单、内置 View 菜单、Scene toolbar 播放按钮和后续插件贡献的 toolbar 命令不再各自解析字符串。

### 无头 Commandlet 分派

`--run` 是 Editor 可执行体的无头任务投影，而不是 App、Runtime 或二进制入口的第二套命令表。`zircon_app::entry::cli::EditorLaunchArgs` 在 GUI 启动参数之前识别 `--run`，随后只把原始参数交给 `zircon_editor::core::commandlet`。因此 `--run migrate-assets --project <root> --dry-run|--apply` 不会构造 `EditorHostEventController`、窗口或 Workbench。

`migrate-assets` 的唯一描述符位于 `core::commands::default_workbench_commands()`，操作路径为 `asset.migration.migrate_assets`，显式声明 `callable_from_remote=true`、payload schema `editor.commandlet.migrate-assets` 和 `asset.migration` capability。runner 先从该注册表读取描述符并按 `CommandEvalCtx::headless` 校验 capability，随后才调用 `zircon_runtime::asset::migration::migrate_project_assets`。

无论成功或失败，进程都向 stdout 输出同一种 JSON envelope：`command`、`status`、数值 `exit_code`、可选 `migration` 报告与可选 `error`。退出码固定为 0（成功）、1（runtime 任务失败或报告 issue）、2（参数或未知 commandlet）与 3（缺失 capability）。这种区分使自动化能把无效调用、构建裁剪导致的功能不可用和真实迁移失败分别处理，而不需要观察 UI 或解析非结构化 stderr。

Material/Fyrox/JetBrains/Unreal 设计栈里的顶层功能编辑器也走同一条 operation 路径。Workbench `Window` 菜单把 Prefab、Material、UI Asset、Animation、Asset Browser 和 Diagnostics 映射到 `editor.*_window` descriptor，并注册 `window.prefab_editor.open`、`window.material_editor.open`、`window.ui_asset_editor.open`、`window.animation_editor.open`、`window.asset_browser.open`、`window.diagnostics.open`。这些 operation 不直接修改 runtime scene；它们是 editor authoring shell 的窗口打开入口。

### 普通命令

1. UI 事件在 workbench 中的某个 `View` 里转成 `Message`
2. `EditorApp` 区分宿主类消息和场景编辑类消息
3. 场景编辑类消息进一步转换或调用 `EditorIntent`
4. `EditorState::apply_intent` 根据意图创建对应 `EditorCommand`
5. `EditorTransactionEngine` 打开 transaction scope；`TransactionScope::push` 通过 `EditContext` 调用 `EditCommand::apply`，此时才修改 `LevelSystem` 所托管的 `World`；apply 失败会在同一 owner 内回滚并取消事务
6. `TransactionScope::commit` 把非空命令序列提交到 `HistoryStore` 的全局 history context
7. `EditorState::sync_selection_state` 从当前世界回填 inspector 草稿和 orbit target

`EditorState::snapshot_with_inspector_customizations(...)` 读取场景树时使用 `World::node_records()`，而不是 schedule-maintained `World::nodes()` 缓存。原因是 editor 命令在同一交互帧内需要立刻投影刚创建、导入、撤销或重做的节点；runtime ECS 的 `nodes()` 缓存仍遵守 `PostUpdate`/`RenderExtract` 刷新边界，不能作为 live authoring snapshot 的数据源。

### Inspector 批量提交

1. 用户编辑 name / parent / translation 字段时，仅更新 `EditorState` 里的草稿字符串；插件动态组件字段会进入 `inspector_dynamic_fields`
2. 在点击 Apply 前，不会即时改动 `World`
3. `ApplyInspectorChanges` 把内建字段解析成目标 `NodeEditState`，把动态组件字段解析成 `ComponentPropertyPath + ScenePropertyValue`
4. `EditorCommand::update_node` 和 `EditorCommand::set_scene_property` 在命令层做合法性校验
5. 如果同一次 Apply 同时修改了节点字段和插件字段，命令会收束进 `BatchEditorCommand`
6. 只有整个状态都可应用时，才一次性写回世界并进入历史栈

这让 inspector 改动成为真正的“批量提交”，而不是每个字符都产生一个世界 mutation。

### Gizmo 拖拽

1. `BeginGizmoDrag` 先在 `EditorState` 的 `GizmoTransactionCapture` 中记录拖拽起点；此时尚不向 history 提交事务
2. 每个 `DragGizmo` preview step 在 viewport 更新世界后捕获 `last→current` 增量，追加一个已应用命令并推进 `capture.last`
3. `EndGizmoDrag` 只补录最后一个尚未捕获的增量；累积命令为空时不创建 history 记录
4. 存在累积命令时，`EndGizmoDrag` 以 `MergeMode::Ends` 通过 `EditorTransactionEngine` 一次提交到全局 history context
5. undo/redo 因此与 inspector 或场景树命令共享同一 `HistoryStore` 模型

## Design And Rationale

### 为什么删除节点以子树为单位

层级编辑已经从旧 Scene 树迁移到 ECS `Hierarchy` 组件。删除父节点如果不携带子树，将留下悬挂节点或额外修补逻辑。当前实现直接捕获 `subtree_records` 并在 undo 时整棵恢复，保证层级关系和局部变换记录一致回放。

### 为什么 inspector 先保留草稿，再显式 Apply

这条链路解决了两个实际问题：

- 避免用户输入半成品数字时频繁打到 ECS 世界
- 把名称、父节点和位移三个字段压成一个 undo step，而不是三个或更多离散命令

### 为什么约束放在 `World` 而不是只放在 UI

UI 层可以隐藏非法操作，但真正的边界必须在 `zircon_scene::Scene` 守住。当前至少有三个强约束必须由世界层保证：

- 不允许删除最后一个 camera
- 不允许把节点设成自己的父节点
- 不允许通过改父子制造层级环

这样未来 runtime、脚本、自动化工具或不同 editor 宿主复用同一世界 API 时，约束仍然成立。

## Edge Cases And Constraints

- 空名称会在 `normalize_edit_state` / `World::rename_node` 阶段被拒绝
- `parent_field` 允许留空，表示把节点挂回根层
- `parent_field` 如果是不存在的实体 id，会整体拒绝这次 batch apply
- batch apply 失败时不允许部分写入；动态插件字段会先在读路径解析当前值和可写 schema，避免内建节点字段先写入后才发现插件字段不可写
- 插件组件 schema 缺失或卸载时，Inspector 可以显示受保护只读数据，但不会允许该字段进入 `SetScenePropertyCommand`
- 删除节点后，selection 会优先落到父节点，否则回退到当前活动 camera
- undo 恢复删除节点时，会同时恢复删除前的 selection 和活动 camera

## Test Coverage

`zircon_editor/src/lib.rs` 当前覆盖：

- 创建节点的 undo/redo
- mesh 导入的 undo
- gizmo 拖拽收束成单条 transform 命令
- 删除节点可撤销
- 删除最后一个 camera 被拒绝
- 重命名和改父子可撤销
- inspector 名称/父节点/位移字段批量提交
- inspector 因非法 parent 失败时保持原世界不变
- inspector 插件动态组件字段提交进入 undo history，Undo 后恢复 JSON payload
- inspector 插件动态组件 schema 卸载后拒绝字段提交
- toolbar Play/Stop binding 会分派成 `runtime.play_mode.enter` / `runtime.play_mode.exit`
- workbench menu action `CreateCube` 会通过 operation runtime 进入 undo stack
- editor operation registry 暴露内置 menu/view/play-mode operation descriptor
- enabled inspector customizations resolve into selected dynamic component Inspector snapshots
- disabled inspector customization capabilities keep customization metadata hidden and leave dynamic component editing protected
- the retained inspector customization adapter accepts safe action-style events beyond button press while rejecting draft value-change events before operation invocation
- `migrate-assets` commandlet 必须经唯一 registry 注册、dry-run 不写入、apply 使用 Runtime 事务、未知/互斥参数输出 JSON exit 2、缺失 capability 输出 JSON exit 3、Runtime typed error 输出 JSON exit 1

`zircon_runtime/src/scene/mod.rs` 当前覆盖：

- 递归删除返回完整子树记录
- `NodeRecord` roundtrip 恢复实体
- `set_parent_checked` 拒绝层级环
- transform 传播与 render extract 保持一致

## Plan Sources

- 用户要求 editor 继续补齐删除节点、改父子层级、重命名和 inspector 字段批量提交
- 用户要求把 undo/redo 从整世界快照升级为真正的 `EditorCommand/UndoableStack`
- 用户要求 Unity 式编辑器优先补齐 Inspector Customization 实际编辑 UI 和插件卸载后的保护/降级诊断
- 当前路线图要求 editor 只维护编辑器域状态，世界修改通过 scene/core 边界下沉到可复用运行时层

## Open Issues Or Follow-up

- inspector 目前只批量提交 translation，rotation / scale 仍是后续扩展项
- inspector 每个字段的字符级输入仍是草稿更新；只有 ApplyBatch 会形成一次 undoable batch
- 层级编辑当前通过 parent id 字段完成，后续可以升级成场景树拖拽重排，但底层仍应复用同一命令模型
- 多选批量编辑还未落地
