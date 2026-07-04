---
related_code:
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/intent.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
reference_sources:
  - dev/godot/editor/editor_undo_redo_manager.h
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ITransaction.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
status: planned
---

# 03 命令 / 事务 / 撤销统一框架

本计划落地 00 §6 事实源表的「文档脏态」权威（`saved_top`）与 §5 数据流的「编辑器 → runtime」半程。

## 参照证据（dev/）

**Fyrox 命令内核**（`dev/Fyrox/editor/src/command/mod.rs:85-344`）——trait 直接模板：

```rust
pub trait CommandTrait {
    fn is_significant(&self) -> bool { true }
    fn name(&mut self, context: &dyn CommandContext) -> String;
    fn execute(&mut self, context: &mut dyn CommandContext);
    fn revert(&mut self, context: &mut dyn CommandContext);
    fn finalize(&mut self, _: &mut dyn CommandContext) {}   // 出栈时释放资源
}
pub struct CommandStack { commands: Vec<Command>, top: Option<usize>, max_capacity, debug }
```

`do_command`（:234-283）压栈清游标以上条目并 `finalize`——被 redo 淘汰的命令有确定释放点。`CommandContext` 是 `ComponentProvider`：命令自取所需系统，栈不认识领域。

**godot 多历史路由**（`editor_undo_redo_manager.h:44-65`、`cpp:63-105`）：`History { id, undo_redo, saved_version, undo_stack, redo_stack }`、`Action { history_id, timestamp, action_name, merge_mode, backward_undo_ops, mark_unsaved }`；`get_history_id_for_object` 判定序：远程对象→REMOTE；场景 Node→场景 history；内嵌 Resource→按路径归属场景；否则 pending/GLOBAL。**`saved_version` 即脏态事实源**。

**UE 事务作用域**（`ScopedTransaction.h:11-47`、`ITransaction.h:19-123`）：RAII + `Cancel()/IsOutstanding()`；`FTransactionContext { TransactionId, OperationId, Title, Context, PrimaryObject }`；`ETransactionStateEventType` 7 态事件流；`ContainsPieObjects()`——事务显式感知 PIE 对象拒绝跨世界撤销（04 消费）。

## 现状与证据（zircon，2026-07-05 实读）——三套栈 + 三套合并机制并存

### 1. 场景栈 `EditorHistory`（`core/editing/history.rs:17-124`，`pub(crate)`）

方法面：`push/begin_drag/end_drag/undo/redo/clear/can_undo/can_redo`。`HISTORY_LIMIT=128` 以 `remove(0)` 淘汰（O(n)，收编时顺手修）；双栈式（undo 清空 redo）；`drag_origin: Option<GizmoDragState{node_id, before: Transform}>` 实现 gizmo 拖拽合并（合并机制一）。

命令 `EditorCommand`（`command.rs:12-18`）：**5 变体**（`CreateNode/DeleteNode/UpdateNode/SetReflectedSceneField/Batch`）× **8 构造入口**（`create_node/import_mesh/delete_node/rename_node/set_parent/set_transform/set_reflected_scene_field/batch`）。三个迁移关键事实（v2 未记，此处补强）：

- **「构造即执行」惯用法**：`capture` 系构造器在构造期间就突变场景——`CreateNodeCommand::spawn_node` 当场 spawn（:203）、`UpdateNodeCommand::capture` 当场 `apply_state`（:376）、`DeleteNodeCommand::capture` 当场 `remove_entity_recursive` 并以删除后世界计算 `selection_after/active_camera_after`（:281-283）。现 `apply()` 实为 **redo** 路径。迁往「push 即 apply」的事务引擎必须先纯化构造（见迁移设计）。
- **选中态内嵌命令**：每变体携带 `previous_selected/selection_before/selection_after`，`apply/undo -> Result<Option<NodeId>, String>` 以返回值转移选中。05 SelectionModel 落地后该职责上移事务记录，命令瘦身。
- **不变量守卫在命令内**：`DeleteNodeCommand::capture` 拒删最后一台相机（:272-274）；`normalize_edit_state` 拒空名（:454-460）。迁移时守卫原样保留在命令实现内（不是引擎职责）。

错误通道为 `String`（全族），统一升级 `EditCommandError`。

### 2. UI 资产栈 `UiAssetEditorUndoStack`（`ui/asset_editor/undo_stack.rs:174-439`）

`push_edit/push_edit_with_style_rule_selection/undo/redo -> Option<UiAssetEditorUndoTransition>`、`undo_record/redo_record -> Option<UiAssetEditorUndoReplayRecord>`、`replay_records() -> Vec<UiAssetEditorUndoStackReplayRecord>`——条目含 label、`tree_edit/inverse_tree_edit`、选择与光标转移，**serde 可回放**（journal 化基础已备）。

### 3. 操作层（`core/editor_operation.rs:48-360`）

```rust
pub struct UndoableEditorOperation { display_name: String }   // :48-50 —— 撤销语义空心（仅名字）
pub struct EditorOperationDescriptor {                        // :65-76
    path, display_name, menu_path: Option<String>,
    payload_schema_id: Option<String>, callable_from_remote: bool,
    undoable: Option<UndoableEditorOperation>, event: Option<EditorEvent>,
    required_capabilities: Vec<String>,
}
pub struct EditorOperationRegistry { descriptors: BTreeMap<EditorOperationPath, _> }  // :163-165
pub struct EditorOperationStack { undo_stack, redo_stack: Vec<EditorOperationStackEntry> }  // :260-263
```

`EditorOperationStack::record`（:266-271）按 `operation_id + operation_group` 同名吸收合并（**合并机制三**；机制二是 UI 栈的光标转移折叠）。

三套互不知晓：Ctrl+Z 语义取决于焦点在哪段代码里；历史不持久化；脏标记与历史无关联；三样全锁在 `EditorEventRuntimeState` 大锁内（01 拆解，operation 两件已定迁 `core/editing/` 临时 owner）。

## 目标

1. 单一事务内核 `core/editing/engine/`：`EditCommand`（Fyrox 五方法）+ `HistoryStore` 多历史（godot 路由 + Fyrox 游标）+ `TransactionScope` RAII（UE 形状，Cancel + 五态事件）。
2. 三套栈收编三命令族；物理删除 `EditorHistory`、`UiAssetEditorUndoStack` 私有栈、`EditorOperationStack`；三套合并机制统一为 `MergeMode` + `try_merge`。
3. 撤销路由：焦点文档 → history；跨文档入 Global 锁定参与文档。
4. `saved_top` 脏态事实源（09 `DirtyRegistry` 与 17 autosave 的消费源）。
5. 历史可观测：五态事件（Started/Canceled/Committed/UndoApplied/RedoApplied）入 01 bus（`TransactionMessage` 族已在 01 定义）；历史面板数据源。

## 非目标

- 不做协作合并；不改 `Scene` 数据结构；journal 存储格式属 11、崩溃恢复消费属 17；历史面板外观属 editor_layout。

## 架构设计

### 模块布局

```
zircon_editor/src/core/editing/
  mod.rs                 # 薄声明
  engine/
    command.rs           # EditCommand trait + CommandBox + EditCommandError
    history.rs           # HistoryContextId / HistoryStore / TransactionRecord
    transaction.rs       # EditorTransactionEngine + TransactionScope + MergeMode
    routing.rs           # 对象 → HistoryContextId 判定
    events.rs            # 五态事件 → TransactionMessage 投递
  scene_commands/        # 现 command.rs 五变体迁入（构造纯化后）
  # UI 资产命令族 owner 不变（ui/asset_editor/），仅实现 trait
```

### 关键类型（定稿）

```rust
// engine/command.rs
pub enum EditCommandError {
    TargetMissing { desc: String },     // 现 "missing node ..." String 族归位
    InvariantViolation { desc: String },// 最后相机/空名等守卫
    ReflectError { desc: String },
    ExternalEffect { desc: String },    // UI 资产写盘等 commit 钩子失败
}
pub trait EditCommand: Send {
    fn label(&self) -> &str;
    fn is_significant(&self) -> bool { true }
    fn apply(&mut self, ctx: &mut EditContext) -> Result<(), EditCommandError>;
    fn revert(&mut self, ctx: &mut EditContext) -> Result<(), EditCommandError>;
    fn finalize(&mut self, _ctx: &mut EditContext) {}
    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome { MergeOutcome::Reject }
    fn serialize_journal(&self) -> Option<serde_json::Value> { None }
}
// EditContext：经 01 EditorContext 提供 gateway/documents/selection，命令自取所需（Fyrox ComponentProvider 思路）

// engine/history.rs
pub enum HistoryContextId { Global, Document(DocumentId) }   // Remote 预留（12）
pub struct HistoryStore {
    entries: Vec<TransactionRecord>,  // 事务为条目单位（非裸命令）
    top: Option<usize>,               // Fyrox 游标式（undo 减、redo 增、push 截断+finalize）
    saved_top: Option<usize>,         // godot saved_version 等价；save 成功时 saved_top = top
    capacity: usize,                  // 128 沿用，VecDeque 存储修 remove(0)，设置化（17）
}
pub struct TransactionRecord {
    pub id: TransactionId, pub label: String,
    pub timestamp_frame: u64,          // 帧号，禁墙钟
    pub commands: Vec<CommandBox>,
    pub participants: BTreeSet<DocumentId>,
    pub selection_before: SelectionSnapshot,   // 选中转移上移至此（命令不再各自背）
    pub selection_after: SelectionSnapshot,
    pub significant: bool,
}

// engine/transaction.rs
pub enum MergeMode { Disable, Ends, All }     // godot 三值直译
impl EditorTransactionEngine {
    pub fn begin(&self, label: &str, ctx_id: HistoryContextId) -> TransactionScope<'_>;
    pub fn undo(&self, ctx_id: HistoryContextId) -> Result<(), EditCommandError>;
    pub fn redo(&self, ctx_id: HistoryContextId) -> Result<(), EditCommandError>;
    pub fn is_dirty(&self, ctx_id: HistoryContextId) -> bool;   // top != saved_top
    pub fn mark_saved(&self, ctx_id: HistoryContextId);
}
impl TransactionScope<'_> {
    pub fn push(&mut self, cmd: impl EditCommand + 'static) -> Result<(), EditCommandError>; // 立即 apply
    pub fn set_merge_mode(&mut self, mode: MergeMode);
    pub fn cancel(self);                       // 逆序 revert 已 push 命令
    pub fn commit(self) -> TransactionId;      // drop 未 commit == cancel（RAII）
}
```

### 构造纯化（迁移的第一道工序，逐命令处置表）

「构造即执行」与「push 即 apply」不能共存，否则首次 apply 双重执行。纯化原则：**capture 只读，突变только在 `apply()`**；首次 apply 需要的派生信息（新实体 id、删除后选中）由 apply 首次执行时计算并缓存回命令字段。

| 命令 | 现构造行为 | 纯化后 |
| --- | --- | --- |
| `CreateNodeCommand` | 构造期 spawn 并抓 record | capture 只定 kind/parent 意图；apply 首次 spawn 缓存 `NodeRecord`，redo 走 `insert_node_record`（现 apply 路径） |
| `DeleteNodeCommand` | 构造期递归删除并算 after 态 | capture 抓 `subtree_records` + 相机守卫判定；apply 执行删除并首算 `active_camera_after` |
| `UpdateNodeCommand` | capture 抓 before 后当场 `apply_state` | capture 只抓 before/after 态；apply 调 `apply_state`（现函数原样复用） |
| `SetReflectedSceneFieldCommand` | capture 读 before 后当场 reflect_write | capture 只 `reflect_read`；apply `reflect_write`（`changed=false` 时命令自报 no-op，scope 丢弃） |
| `Batch` | 变体删除 | 事务多 push 取代；调用点 `EditorCommand::batch(...)` 改为同 scope 连续 push |

守卫（最后相机/空名）留在各命令 `apply` 内，失败返回 `InvariantViolation`，scope 收到即整体 cancel。

### 合并统一（三机制 → 一机制）

- 机制一（gizmo 拖拽）：05 拖拽期间持 `MergeMode::Ends` 长事务——begin_drag 开事务、每帧 push `set_transform`（`try_merge` 吸收中间值，Ends=只留首末）、end_drag commit。`GizmoDragState`/`begin_drag/end_drag` 删除。
- 机制二（UI 栈光标折叠）：`UiAssetEditCommand::try_merge` 实现同型折叠（现折叠逻辑迁入）。
- 机制三（operation_group 同名吸收）：操作层 factory 产出命令时把 `operation_group` 映射为同一未 commit 事务（组变即 commit 前组），`EditorOperationStack::record` 合并逻辑删除。

### 三族迁移映射（执行合同）

| 现物 | 迁移形态 | 删除物 |
| --- | --- | --- |
| `EditorCommand` 5 变体 | 纯化 + 实现 `EditCommand`；选中字段族（`previous_selected/selection_*`）整体摘除，`apply/revert` 返回 `Result<(), _>` | `EditorHistory`、`GizmoDragState`、`HISTORY_LIMIT`、`Batch` 变体、String 错误 |
| `SourceEditEntry` | 包装 `UiAssetEditCommand`（tree_edit=apply、inverse=revert）；`serialize_journal` 复用现 serde；写盘效应改 commit 钩子 | `UiAssetEditorUndoStack` 双栈、`undo_record/redo_record/replay_records` 旧入口 |
| `UndoableEditorOperation`（空心） | 描述符字段改 `undoable: Option<OperationCommandFactory>`（产 `EditCommand`）；无 factory 即不可撤销 | `EditorOperationStack` + `EditorOperationStackEntry` |
| 命令内选中转移 | `TransactionRecord.selection_before/after`（引擎在 begin/commit 时向 SelectionModel 抓拍/恢复） | 各命令选中字段 |

### 路由（godot 判定序直译）

1. 显式 `HistoryContextId` → 用之；2. 目标属某文档（场景实体→场景文档、UI 资产节点→资产文档、内嵌子资产→宿主文档）→ `Document(id)`；3. 参与文档 >1 → `Global` 记 participants，undo 时校验在场（缺席提示而非静默失败）；4. 其余 → `Global`。Ctrl+Z：焦点文档 history 非空 → 撤之；否则 `Global`。

### 深度测试

夹具文档 `FixtureDoc` + 两夹具命令接入=实现 trait + routing 条目，engine/ 零改动。07 行为树图文档是真实第二验证者。

## 里程碑

### M1 事务内核

- 切片 1.1：engine/ 五文件；单测矩阵——游标语义（push 截断 redo 段并 finalize）、容量淘汰 finalize、cancel 逆序 revert、drop 即 cancel、嵌套折叠、跨 context 嵌套 Err、MergeMode 三值、`saved_top` 判定与 `mark_saved`、五态事件顺序。
- 切片 1.2：`EditorOperationStack` 自 01 临时 owner 摘除，引擎挂 `EditorContext`（01 占位字段接活）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`；Grep 断言 `EditorOperationStack` 零残留。更新 `docs/zircon_editor/core/editing.md`。

### M2 场景命令族纯化迁移

- 切片 2.1：按处置表逐命令纯化 + 实现 trait；选中字段摘除（过渡期由引擎抓拍顶替）；错误 String→`EditCommandError`；viewport/inspector 调用点改 `TransactionScope`；gizmo 长事务；删 `EditorHistory` 全文件。
- 测试阶段：`core/editing` 既有测试改写后须过；新增：纯化回归（capture 后世界无变化断言——对照旧惯用法的关键新测）、拖拽 100 帧 push→历史 1 条、`set_reflected_scene_field` 撤销往返、最后相机守卫经引擎路径仍拒绝、Grep `EditorHistory` 零命中。

### M3 UI 资产族与操作层收编 + 路由

- 切片 3.1：`UiAssetEditCommand` 迁移；写盘效应 commit 钩子（失败→自动 revert，与 09 保存流程对齐）；删私有栈。
- 切片 3.2：操作层 factory 化 + operation_group→事务组映射 + `EditorOperationStack` 删除；routing.rs + Ctrl+Z 接 08 命令。
- 测试阶段：asset_editor 既有测试迁移后须过；双文档交错撤销互不串扰矩阵；Global 参与文档缺席提示单测；三合并机制统一后各原场景回归（gizmo/光标折叠/组吸收）。

### M4 观测与 journal

- 切片 4.1：五态事件入 bus（01 `TransactionMessage` 族接活）；历史面板数据源；`serialize_journal` 实现度清点（未实现记债）。
- 测试阶段：事件序列断言 + journal 往返；证据记状态节。

## 风险与开放问题

- 构造纯化改变 8 个调用入口的时序语义（调用方此前依赖「构造返回即已生效」）——M2 调用点迁移必须与纯化同切片完成，不允许新旧惯用法并存一帧。
- `set_reflected_scene_field` 整值快照内存成本：M2 保持现状；差量化留待 runtime 反射迁移能力落地评估（触发条件：单值 >64KB 或历史内存超阈，记状态节）。
- UI 资产 commit 钩子失败时 revert 依赖 `inverse_tree_edit` 完备性——迁移时逐类 tree_edit 补逆操作完备性测试。
- `ContainsPieObjects` 等价：PIE session 实体禁入编辑事务——routing 第 2 步按文档归属天然拒绝（PIE 视口非文档），另补显式守卫测试（04 会签）。
