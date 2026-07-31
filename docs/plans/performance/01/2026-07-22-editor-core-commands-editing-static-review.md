---
related_code:
  - zircon_editor/src/core/context
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editing
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
reference_sources:
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/editor_undo_redo_manager.h
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Fyrox/editor/src/message.rs
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ITransaction.h
tests:
  - zircon_editor/src/core/commands/key_chord.rs::tests
  - zircon_editor/src/core/commands/keymap/tests.rs::command_chord_lookup_uses_sorted_binding_search
  - zircon_editor/src/core/editing/command.rs::performance_source_guards
  - zircon_editor/src/core/editing/engine/history.rs::performance_source_guards
  - zircon_editor/src/core/editing/engine/transaction.rs::performance_source_guards
  - zircon_editor/src/core/editing/engine/transaction/operation_group.rs::performance_source_guards
  - current-source Windows Cargo and editor product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core commands、editing与context逐文件性能静态审查（2026-07-22）

## 范围与覆盖

已逐文件阅读`zircon_editor/src/core/context` 3/3、`commands`生产17/17（外部`keymap/tests.rs`只按直接修复补测试，未计生产覆盖）、`editing` 17/17，共 **37/37生产文件**、约4.8k物理行。与上一批合计，`zircon_editor/src/core`当前完成 **144/257** 文件静态阅读；其余113个文件与外部tests继续保留`pending.md`。

受管Cargo lane仍被其他Session占用，没有运行raw Cargo。本切片无GPU提交，RenderDoc不适用；真实viewport/gizmo GPU证据仍归Editor05/Render17，不能用源码守卫替代。

## 已确认的性能形状

- keymap已由controller lifetime持有，旧的per-key TOML parse已修复；但每键仍构造normalized owned chord并线性扫bindings，`chord_for_command`也线性扫。后者本轮利用bindings按command id排序改为binary search；ASCII alias/keyboard normalization删除临时lowercase String，Display直接写formatter而不建`Vec<String>`/join。最终chord dispatch index继续归PERF-MVP-074。
- command palette已经是generation-owned `Arc` catalog和bounded handle window，不再open时深clone全catalog。非空query仍完整执行两遍document fuzzy scan，每遍对enabled行按id回查registry BTreeMap；`to_ui_value`只物化窗口但仍逐字段clone。继续更新PERF-MVP-211，不恢复旧full UiValue入口。
- menu bar每次按7个顶层label分别扫描全registry并新建所有label/id/shortcut；command registry handle以一个Mutex包住mutation、palette/menu projection和dispatch lookup。该工作通常发生在open/recompute，不凭静态形状直接拆锁；先以generation build count和lock hold量化，归Editor08 immutable menu projection。
- transaction engine正确地把client command callback移出state mutex，并以Condvar处理RAII scope等待；history容量固定128，evicted record在锁外finalize。这些是正确基线。
- `CoreEditContext`仍把typed scene selection编码为`serde_json::Value`，每次transaction begin/commit、undo/redo恢复都clone或`from_value`往返；大选择集成本随S增长。`HistoryStore::snapshot`还深clone全部128 records的label、participants与before/after JSON selection，transaction event Vec无明确entry/age drain预算。新增PERF-MVP-549由Editor03收口。
- undo/redo只需event id+label，却过去调用`record.snapshot()`深cloneparticipants与两份selection；本轮改为compact `(TransactionId, String)`。operation group继续同一drag时不再clone稳定group key；Create redo不再为`insert_node_record`做第二次subtree record clone。
- PERF-MVP-063的directional setter与nested cancel pop已成立，但transform/rename/reparent仍统一捕获`NodeEditState{name,parent,transform}`。连续gizmo update每次复制稳定name，merge又clone完整after state；应改为field-specific delta/shared stable fields，而非在UI侧再加缓存。

## 参考引擎核对

- Fyrox command stack保留execute/revert/finalize和固定capacity，record淘汰时显式finalize；Zircon当前锁外finalize方向正确，但不应为UI状态查询深clone宽record。
- Godot `EditorUndoRedoManager`按history保存action/version并以merge mode合并连续编辑；Zirconoperation group保留同一语义，性能目标是只更新changed field和compact history metadata，不改变history路由。
- Unreal `FScopedTransaction`把RAII scope与transaction context分离；Zircon的scope/Condvar owner可保留，但selection、event与diagnostics应使用typed/shared generation而不是JSON/无界Vec。

## 本轮直接止损与动态验收

直接止损：key normalization/format零临时lowercase+parts列表；command-id chord lookup O(log N)；operation-group continuation零key clone；undo/redo event零full record snapshot；Create redo删除第二次NodeRecord clone。源码守卫、scoped rustfmt与diff check通过；current-source Cargo尚未取得lane。

动态门按commands/history/selection **1/100/10k**、drag updates **1/1k/100k**、history records **1/128**记录key allocations/binding probes、palette visits/BTree probes、selection JSON bytes/serde traversals、history snapshot clone bytes、transaction event entries/age、state lock wait和main-thread p95。要求stable key dispatch近O(1)且额外alloc=0、drag每update稳定name/group clone=0、compact status不复制records/selection、event backlog硬有界；command/when/keymap/merge/undo/redo/selection/save/rollback语义及F4产品交互通过后方可进入`review.md`。
