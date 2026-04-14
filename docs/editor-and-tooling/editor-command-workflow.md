---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/history.rs
  - zircon_editor/src/editing/state.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/host/message.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/workbench/snapshot.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world.rs
implementation_files:
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/history.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/editing/state.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/workbench/snapshot.rs
  - zircon_scene/src/world.rs
plan_sources:
  - user: 2026-04-12 扩展 editor 命令系统到删除节点、改父子层级、重命名和 inspector 字段批量提交
  - user: 2026-04-12 将 undo/redo 从整世界快照推进到真正的 EditorCommand/UndoableStack 命令化实现
  - user: 2026-04-12 实现 Zircon Editor Workbench Shell V1
  - .cursor/plans/基本路线图.md
tests:
  - zircon_editor/src/lib.rs
  - zircon_scene/src/lib.rs
  - cargo test -p zircon_editor -- --nocapture
  - cargo test -p zircon_entry -- --nocapture
doc_type: module-detail
---

# Editor Command Workflow

## Purpose

`zircon_editor` 不再把整个 ECS 世界直接塞进 undo/redo 快照，而是通过 `EditorCommand` 把一次编辑收束为可应用、可撤销、可重做的命令对象。这样 editor UI 可以保持 Slint 宿主职责，只负责维护草稿字段和触发 intent；真正的世界修改则统一通过命令层进入 `zircon_scene::LevelSystem` 所托管的 `World`。

这一设计直接服务当前路线图里的两个目标：

- editor 对 ECS 世界的修改必须可逆，并且不能把整世界快照当作长期主路径
- inspector、gizmo、场景树这些不同入口触发的编辑，需要落到同一套 undoable 行为模型

## Related Files

- `zircon_editor/src/editing/command.rs`: 命令类型、创建/删除/更新节点逻辑
- `zircon_editor/src/editing/history.rs`: undo/redo 栈和 gizmo drag 聚合逻辑
- `zircon_editor/src/editing/state.rs`: `EditorIntent` 到命令执行的主入口，维护 inspector 草稿态
- `zircon_editor/src/host/slint_host/app.rs`: 统一接住项目保存/加载和多窗口 workbench 宿主消息，再驱动命令执行
- `zircon_editor/src/host/manager.rs`: 提供布局、view registry、项目 workspace 的 editor 域协调入口
- `zircon_editor/src/workbench/project.rs`: editor project/workspace sidecar 与 level 文档桥接
- `zircon_editor/src/workbench/snapshot.rs`: workbench 与资产面板投影快照
- `zircon_scene/src/world.rs`: 世界层约束，如最后一个 camera 不可删、层级不可成环

## Behavior Model

当前命令模型包含三类命令：

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

`EditorState::apply_intent` 是 editor 侧权威入口。它不直接绕过命令层修改世界，除了 gizmo 拖拽中的中间帧预览。即便是 inspector 的 Apply，也会先组装成 `UpdateNodeCommand`，再进入历史栈。

## Control Flow

### 普通命令

1. UI 事件在 workbench 中的某个 `View` 里转成 `Message`
2. `EditorApp` 区分宿主类消息和场景编辑类消息
3. 场景编辑类消息进一步转换或调用 `EditorIntent`
4. `EditorState::apply_intent` 根据意图创建对应 `EditorCommand`
5. 命令在构造阶段直接修改 `LevelSystem` 所托管的 `World`
6. 成功后命令进入 `EditorHistory` 的 undo 栈
7. `EditorState::sync_selection_state` 从当前世界回填 inspector 草稿和 orbit target

### Inspector 批量提交

1. 用户编辑 name / parent / translation 字段时，仅更新 `EditorState` 里的草稿字符串
2. 在点击 Apply 前，不会即时改动 `World`
3. `ApplyInspectorChanges` 把草稿解析成目标 `NodeEditState`
4. `EditorCommand::update_node` 在命令层做合法性校验
5. 只有整个状态都可应用时，才一次性写回世界并进入历史栈

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

UI 层可以隐藏非法操作，但真正的边界必须在 `zircon_scene::World` 守住。当前至少有三个强约束必须由世界层保证：

- 不允许删除最后一个 camera
- 不允许把节点设成自己的父节点
- 不允许通过改父子制造层级环

这样未来 runtime、脚本、自动化工具或不同 editor 宿主复用同一世界 API 时，约束仍然成立。

## Edge Cases And Constraints

- 空名称会在 `normalize_edit_state` / `World::rename_node` 阶段被拒绝
- `parent_field` 允许留空，表示把节点挂回根层
- `parent_field` 如果是不存在的实体 id，会整体拒绝这次 batch apply
- batch apply 失败时不允许部分写入，名称、父子和 transform 都必须保持原值
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

`zircon_scene/src/lib.rs` 当前覆盖：

- 递归删除返回完整子树记录
- `NodeRecord` roundtrip 恢复实体
- `set_parent_checked` 拒绝层级环
- transform 传播与 render extract 保持一致

## Plan Sources

- 用户要求 editor 继续补齐删除节点、改父子层级、重命名和 inspector 字段批量提交
- 用户要求把 undo/redo 从整世界快照升级为真正的 `EditorCommand/UndoableStack`
- 当前路线图要求 editor 只维护编辑器域状态，世界修改通过 scene/core 边界下沉到可复用运行时层

## Open Issues Or Follow-up

- inspector 目前只批量提交 translation，rotation / scale 仍是后续扩展项
- 层级编辑当前通过 parent id 字段完成，后续可以升级成场景树拖拽重排，但底层仍应复用同一命令模型
- 多选批量编辑和复合命令事务还未落地
