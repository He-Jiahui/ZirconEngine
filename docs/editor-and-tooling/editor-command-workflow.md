---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/common/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/editor_event/host_adapter.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world/mod.rs
implementation_files:
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_scene/src/world/mod.rs
plan_sources:
  - user: 2026-04-12 扩展 editor 命令系统到删除节点、改父子层级、重命名和 inspector 字段批量提交
  - user: 2026-04-12 将 undo/redo 从整世界快照推进到真正的 EditorCommand/UndoableStack 命令化实现
  - user: 2026-04-12 实现 Zircon Editor Workbench Shell V1
  - user: 2026-05-02 Unity 式编辑器优先补齐计划：Inspector / Component Drawer 接入 Undo/Redo
  - .cursor/plans/基本路线图.md
  - .codex/plans/ZirconEngine Unity 式编辑器优先补齐计划.md
tests:
  - zircon_editor/src/lib.rs
  - zircon_scene/src/lib.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs
  - cargo test -p zircon_editor -- --nocapture
  - cargo test -p zircon_app -- --nocapture
  - cargo test -p zircon_editor --lib inspector_binding_applies_dynamic_plugin_component_fields_with_undo_history --locked --jobs 1
  - cargo test -p zircon_runtime --lib dynamic_plugin_component_instances_report_schema_when_loaded_and_protect_when_missing --locked --jobs 1
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe builtin_viewport_toolbar_play_buttons_dispatch_menu_play_mode_operations --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe menu_action_dispatches_through_runtime_and_sets_scene_dirty_effects --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe editor_operation_registry_exposes_builtin_menu_operations_by_path --nocapture (passed)
doc_type: module-detail
---

# Editor Command Workflow

## Purpose

`zircon_editor` 不再把整个 ECS 世界直接塞进 undo/redo 快照，而是通过 `EditorCommand` 把一次编辑收束为可应用、可撤销、可重做的命令对象。这样 editor UI 可以保持 Slint 宿主职责，只负责维护草稿字段和触发 intent；真正的世界修改则统一通过命令层进入 `zircon_runtime::scene::LevelSystem` 所托管的 `zircon_scene::Scene`。

这一设计直接服务当前路线图里的两个目标：

- editor 对 ECS 世界的修改必须可逆，并且不能把整世界快照当作长期主路径
- inspector、gizmo、场景树这些不同入口触发的编辑，需要落到同一套 undoable 行为模型

## Related Files

- `zircon_editor/src/core/editing/command.rs`: 命令类型、创建/删除/更新节点逻辑
- `zircon_editor/src/core/editor_operation.rs`: menu/toolbar/editor 插件 operation descriptor registry
- `zircon_editor/src/core/editing/history.rs`: undo/redo 栈和 gizmo drag 聚合逻辑
- `zircon_editor/src/core/editing/state/mod.rs`: `EditorIntent` 到命令执行的主入口，维护 inspector 草稿态
- `zircon_editor/src/ui/slint_host/app.rs`: 统一接住项目保存/加载和多窗口 workbench 宿主消息，再驱动命令执行
- `zircon_editor/src/core/host/manager.rs`: 提供布局、view registry、项目 workspace 的 editor 域协调入口
- `zircon_editor/src/ui/workbench/project/mod.rs`: editor project/workspace sidecar 与 level 文档桥接
- `zircon_editor/src/ui/workbench/snapshot/mod.rs`: workbench 与资产面板投影快照
- `zircon_editor/src/ui/slint_host/callback_dispatch/common/dispatch.rs`: template binding 的 operation-first 分派入口
- `zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs`: workbench menu action 到 `EditorOperation` 的桥接
- `zircon_scene/src/world/mod.rs`: 世界层约束，如最后一个 camera 不可删、层级不可成环

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

### EditorOperation 分派

1. Workbench menu item、toolbar button 或 builtin template binding 先携带 stable action id 进入 Slint host dispatcher
2. `operation_path_for_menu_action(...)` 把内置 menu action 映射到 `EditorOperationDescriptor` 路径，例如 `Scene.Node.CreateCube`、`Runtime.PlayMode.Enter`、`Runtime.PlayMode.Exit`、`View.BuildExport.Open`
3. `dispatch_editor_binding(...)` 和 `dispatch_menu_action(...)` 优先调用 `EditorEventRuntime::invoke_operation(...)`
4. operation registry 根据 capability snapshot 和 descriptor 决定该命令是否可见、可调用，以及是否声明 undoable
5. 真正修改场景的 operation 继续进入 `EditorState::apply_intent` 和 `EditorCommand`；播放模式和窗口打开这类不可撤销命令则停在 editor event/runtime 边界处理副作用

这条路径让插件菜单、内置 View 菜单、Scene toolbar 播放按钮和后续插件贡献的 toolbar 命令不再各自解析字符串。

### 普通命令

1. UI 事件在 workbench 中的某个 `View` 里转成 `Message`
2. `EditorApp` 区分宿主类消息和场景编辑类消息
3. 场景编辑类消息进一步转换或调用 `EditorIntent`
4. `EditorState::apply_intent` 根据意图创建对应 `EditorCommand`
5. 命令在构造阶段直接修改 `LevelSystem` 所托管的 `World`
6. 成功后命令进入 `EditorHistory` 的 undo 栈
7. `EditorState::sync_selection_state` 从当前世界回填 inspector 草稿和 orbit target

### Inspector 批量提交

1. 用户编辑 name / parent / translation 字段时，仅更新 `EditorState` 里的草稿字符串；插件动态组件字段会进入 `inspector_dynamic_fields`
2. 在点击 Apply 前，不会即时改动 `World`
3. `ApplyInspectorChanges` 把内建字段解析成目标 `NodeEditState`，把动态组件字段解析成 `ComponentPropertyPath + ScenePropertyValue`
4. `EditorCommand::update_node` 和 `EditorCommand::set_scene_property` 在命令层做合法性校验
5. 如果同一次 Apply 同时修改了节点字段和插件字段，命令会收束进 `BatchEditorCommand`
6. 只有整个状态都可应用时，才一次性写回世界并进入历史栈

这让 inspector 改动成为真正的“批量提交”，而不是每个字符都产生一个世界 mutation。

### Gizmo 拖拽

1. `BeginGizmoDrag` 让 `EditorHistory` 记录拖拽起点状态
2. 拖拽过程中的世界变换由 viewport 控制器实时更新，服务于预览
3. `EndGizmoDrag` 比较拖拽起点和当前节点状态
4. 如果节点确实发生变化，收束为一个 `UpdateNodeCommand`
5. undo/redo 因此与 inspector 或场景树命令共享同一历史模型

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
- toolbar Play/Stop binding 会分派成 `Runtime.PlayMode.Enter` / `Runtime.PlayMode.Exit`
- workbench menu action `CreateCube` 会通过 operation runtime 进入 undo stack
- editor operation registry 暴露内置 menu/view/play-mode operation descriptor

`zircon_scene/src/lib.rs` 当前覆盖：

- 递归删除返回完整子树记录
- `NodeRecord` roundtrip 恢复实体
- `set_parent_checked` 拒绝层级环
- transform 传播与 render extract 保持一致

## Plan Sources

- 用户要求 editor 继续补齐删除节点、改父子层级、重命名和 inspector 字段批量提交
- 用户要求把 undo/redo 从整世界快照升级为真正的 `EditorCommand/UndoableStack`
- 用户要求 Unity 式编辑器优先补齐 Component Drawer 实际编辑 UI 和插件卸载后的保护/降级诊断
- 当前路线图要求 editor 只维护编辑器域状态，世界修改通过 scene/core 边界下沉到可复用运行时层

## Open Issues Or Follow-up

- inspector 目前只批量提交 translation，rotation / scale 仍是后续扩展项
- inspector 每个字段的字符级输入仍是草稿更新；只有 ApplyBatch 会形成一次 undoable batch
- 层级编辑当前通过 parent id 字段完成，后续可以升级成场景树拖拽重排，但底层仍应复用同一命令模型
- 多选批量编辑还未落地
