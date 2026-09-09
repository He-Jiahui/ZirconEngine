---
related_code:
  - zircon_editor/src/core/commands/mod.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
implementation_files:
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/editing/engine/transaction/engine_state.rs
  - zircon_editor/src/core/editing/engine/transaction/scope.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/journal
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Runtime_Editor 插件注册与 EditorOperation 设计计划.md
  - docs/editor-and-tooling/editor-command-workflow.md
tests:
  - zircon_editor/src/core/editing/engine
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editor_event
doc_type: module-detail
---

# 命令、事务与撤销系统

## 概览

Zircon 把“命令入口”和“可撤销编辑”分成两层：

- `EditorCommandRegistry` 描述一个操作何时可见、可用、如何展示、由何种执行器处理。
- `EditorTransactionEngine` 执行会改变作者态的 `EditCommand`，保存 undo/redo、selection、dirty 和 journal。

菜单、快捷键、命令面板、远控和 CLI 使用相同的 `EditorOperationPath`，但不是每个命令都必须产生事务。例如打开设置窗口是命令，却不一定修改文档。

## 注册命令

```rust
use zircon_editor::{
    EditorCommandDescriptor, EditorCommandRegistry,
};
use zircon_editor::core::editor_operation::EditorOperationPath;

let descriptor = EditorCommandDescriptor::operation(
    EditorOperationPath::parse("scene.node.rename")?
);

let mut registry = EditorCommandRegistry::default_workbench();
registry.register(descriptor)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`EditorCommandDescriptor` 可配置：

- 本地化 label/description、category 和 keywords。
- 菜单路径与菜单投影策略。
- 默认 `EditorKeyChord`。
- `WhenClause` 和 Play mode predicate。
- payload schema、headless commandlet route/name。
- remote callable 标志和 required capabilities。
- `EditorCommandExecutionContract`：输入/输出大小、时间和 codec 约束。
- 资产写目标参数，用于命令执行前的写入边界检查。

插件批量贡献应使用 `EditorCommandContributionSet`，并为 `Operation` action 同时注册 operation factory。贡献退休时由注册系统统一撤销，而不是在插件 unload 后留下悬空菜单项。

## 命令可用性

`CommandEvalCtx` 是一次判定的不可变上下文，涵盖项目是否打开、文档类型、选择、Play 状态、焦点、capability 等。`WhenClause` 组合这些谓词。

UI 不应复制判定逻辑；菜单、快捷键和命令面板都调用同一 descriptor 的 `is_enabled`/registry 检查。若需要跨线程读取，使用 `CommandEvalSnapshotHandle` 发布快照。

## Operation 路径

`EditorOperationPath::parse` 校验稳定命名路径。一次调用由 `EditorOperationInvocation` 表示：

```rust
use serde_json::json;
use zircon_editor::core::editor_operation::EditorOperationInvocation;

let invocation = EditorOperationInvocation::parse("scene.node.rename")?
    .with_arguments(json!({"entity": 42, "name": "Light"}))
    .with_operation_group("inspector-rename");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`operation_group` 用来把连续交互合并成一个逻辑事务，例如滑块拖动或连续文本提交。来源由 `EditorOperationSource::{Menu, UiBinding, Remote, Cli}` 标记。

## EditorEvent 与 effect

typed binding 首先规范化为 `EditorEvent`。`EditorEventService`/dispatcher 执行后产生 `EditorEventRecord` 与 `EditorEventEffect`：effect 告诉宿主需要刷新布局、重绘、同步资产或执行 host-only 操作。

关键原则是：事件记录语义，宿主消费副作用。文件选择器、进程启动和 native window 不应在纯 event reducer 中直接执行。

`EditorEventJournal` 支持会话记录；listener registry 可按 filter 订阅有界 delivery；`EditorEventReplay` 用同一语义路径复现事件。Transient hover/focus 等状态通过专用 transient 载荷处理，不应进入永久 undo 历史。

## 可撤销命令

`EditCommand` 的实现需要定义 apply/revert，并可选择 merge、finalize 和 journal codec 行为。命令执行错误携带 `CommandEffect::{Unchanged, Applied}`，让事务引擎知道失败发生前是否已经产生修改并进行补偿。

事务的典型调用：

```rust
use zircon_editor::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId,
};

let engine = EditorTransactionEngine::new(edit_context);
let mut tx = engine.begin(
    "Rename Actor",
    HistoryContextId::Document(document_id),
)?;
tx.add_participant(document_id);
tx.push(rename_command)?;
let transaction_id = tx.commit()?;
```

宿主常通过 operation factory 封装这一步；插件通常不应持有 engine 内部锁。

## History context

`HistoryContextId` 分为：

- `Global`：非单文档作者态。
- `Document(DocumentId)`：普通资产/Scene 文档。
- `PlaySession(PlayInstanceId)`：Play 世界临时编辑。

Play history 是 volatile：不会成为源资产 dirty 保存点，Play session 退休时应调用 `discard_play_history`。Edit 与 Play 的命令路由通过 `EditWorldRoute` 区分 logical/in-process runtime，并可绑定 `GatewaySessionIdentity`。

## Undo/redo 原子性

一条事务包含命令序列、参与文档、选择前后快照、时间帧和 significant 标记。Undo 逆序 revert，Redo 正序 apply。若中途失败，引擎会回滚已完成部分并恢复原 selection；补偿也失败时返回 `RollbackFailed`，调用者必须停止继续编辑并暴露错误。

`HistoryStore` 有容量上限。新事务会清除 redo 分支；若保存点因此不可达，`saved_top_reachable` 变为 false，文档保持 dirty。

## 交互合并

`MergeMode` 与 operation group 用于把高频变更合并。Gizmo 拖拽常用模式是：begin 建立 interactive state，move 更新预览/命令，end 提交一个事务，cancel 恢复起始 transform。不能为每个 pointer move 都创建独立用户可见 undo 项。

## 保存点与 dirty

`HistoryStatus` 提供 `can_undo`、`can_redo`、`dirty`、generation 和当前/saved top。异步保存必须：

```rust
let token = engine.capture_save_token(history)?;
save_document_to_disk()?;
let outcome = engine.mark_saved_if_unchanged(token)?;
```

若保存期间发生新事务，返回不会误标为已保存。`HistoryDirtyBatch` 用 cursor 增量投影多个 history 的 dirty 变化，适合 workbench 更新 tab 标记。

## Durable journal 与恢复

可持久化命令通过 `EditCommandCodecRegistry` 编解码为 `CommandJournalPayload`。`DurableJournal` 按 `JournalDocumentKey` 保存事务，reader 能报告尾部损坏，replayer 在 codec 和文档身份验证通过后重放。

并非所有命令都可 journal。`CommandJournalUnavailable` 表示该命令只有内存 undo，不能承诺崩溃恢复。保存源文件后可 compact 已覆盖的 journal 前缀。

## 限制与错误处理

- History capacity 必须大于零，分页上限由 `MAX_HISTORY_DETAIL_PAGE_SIZE` 限制。
- 事务 scope drop 时若未显式 commit/cancel，错误通过 `take_drop_error` 暴露；生产路径应显式结束。
- Runtime 路由必须固定 gateway identity；endpoint 替换后旧事务不可继续提交。
- Native command executor 受时间、输入/输出字节、codec 和 capability budget 约束。
- UI Asset Editor 另有 session-local undo stack，详见[资产编辑器](asset-editors.md)；它和通用 Scene transaction history 不能混作同一个栈。
