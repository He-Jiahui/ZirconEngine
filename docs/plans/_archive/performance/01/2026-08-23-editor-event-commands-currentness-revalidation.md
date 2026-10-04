---
related_code:
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/core/commands
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-core-editor-event-input-transaction-audit-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-commands-routing-search-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/ScopedTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/InputBindingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
doc_type: implementation-evidence
status: static_current_structural_cutover_required_dynamic_blocked
---

# Editor Event / Commands currentness复验（2026-08-23）

## 当前清单

| 模块 | current Rust | 行 / bytes / tests | path+raw SHA256 | currentness结论 |
|---|---:|---:|---|---|
| `core/editor_event/**` | 36/36 | 2,700 / 86,352 / 8 | `2077891e59df157f31e594442002b97aea8a8b029e3936e2a9c72d0e8a94e09f` | 对8月16日36/36逐文件基线完成提交和工作区diff复验；结构结论不变 |
| `core/commands/**` | 17/17 | 4,311 / 141,612 / 31 | `1cd589687193aa10acf3f5a992e594fe9fd13d929d4a53307dfa45d3aa8adb31` | 对8月16日17/17逐文件基线完成提交和工作区diff复验；结构结论不变 |

本轮保留6个用户/其他Session未提交文件，不覆盖、不格式化。复验读取了基线后完整提交diff、当前工作区diff、所有漂移文件及其dispatch/effect/retention/replay owner链；未修改本组Rust源码。

## 漂移逐文件核对

`08094b9b9`对`commands/registry.rs`只有import排序，对`editor_event/listener/registry.rs`只有测试builder调用更新，均无产品算法变化。

当前工作区变化如下：

| 文件 | 当前变化 | 性能/结构结论 |
|---|---|---|
| `commands/defaults.rs` | 增加Save All descriptor和测试 | bootstrap常数增加1；没有改变per-event reverse scan或menu全表扫描 |
| `editor_event/dispatcher.rs` | dispatcher改为associated typed error | 错误保真改进；仍返回完整owned `EditorEventRecord` |
| `editor_event/replay.rs` | 泛型typed replay error | 仍逐条clone并dispatch全部输入record，不读取retention class或replay disposition |
| `editor_event/mod.rs` | 导出typed replay error | re-export，无热点 |
| `editor_event/types.rs` | 增加Save All effect | typed payload，无独立热点 |
| `editor_event/workbench/menu_action.rs` | 增加Save All action | typed payload，无独立热点 |

Save All当前落入`retention_class`默认`DurableReplay`，Replay仍重新dispatch该事件，因此会重复命令反查、shell执行、effect/record构造、失效和审计链。`refresh_workbench_for_event_record`只对`RetainedHost`走effect bridge，所以Replay当前不会实际触发`save_all_documents`；这只是source分支造成的结果，不能替代显式的non-replayable side-effect disposition。

## 结构瓶颈仍开放

- realtime pointer/resize/scroll与semantic command、authoring transaction、UI invalidation、audit/plugin delivery、executable replay仍共享一条宽事件管线。零listener时也会发生sequence/revision、command reverse scan、shell lock、effect/result、record clone、JSON byte traversal和journal工作。
- `begin_event`在执行前推进authoring revision，failed/no-op/raw input都能制造假authoring generation。
- `EditorEventReplay`仍执行传入的全部`EditorEventRecord`；raw input、transient UI、失败和外部side-effect request没有显式fail-closed disposition。
- `SharedEditorEventRecord::new`仍为每条事件完整`serde_json::to_vec`只取长度。改为counting writer被本模块测试明确禁止，因为它仍保留完整JSON遍历，并继续把wire bytes误作typed heap retention。
- 任意无显式operation metadata事件仍锁command registry并对全部descriptor做`descriptor_for_event`线性比较；direct event的常见miss仍为`O(commands)`。
- menu bar七个top-level bucket仍分别全表扫描，在部分caller持有command/shell锁时构造owned rows；extension command batch仍是重复mutation/invalidation而非一次compiled-generation commit。
- palette的shared catalog、rarest-byte posting、single-pass scorer和bounded top-K是正确基线；common/empty query仍为`O(N)`，但不能退回全量candidate materialization或私有无界worker。

## 参考引擎约束

- Unreal `SlateApplication.cpp`直接把pointer move路由到当前widget path/capture/drag状态，不先转换为editor transaction或serialized audit record。Zircon realtime input必须绕开command reverse lookup与replay journal。
- Unreal `ScopedTransaction.cpp`只为显式transaction begin/end；`EditorTransaction.cpp`保存custom change或changed-object状态。这支持只重放已提交semantic operation，不支持重放原始UI事件。
- Unreal `InputBindingManager.cpp`以chord-to-command map和command identity查找；`UICommandList.cpp`以`FUICommandInfo`直接执行action。Zircon应在binding/menu/keymap/remote normalization阶段生成`CommandRoute`，而不是执行后反查任意event。
- Unreal `SlateInvalidationRoot.cpp`把dirty fast path与完整slow path分离。Zircon执行receipt应发布固定invalidation domains，stable/no-op input不推进presentation generation。

这些源码建立ownership和复杂度基线，不提供可移植的耗时或功耗目标。

## 必须执行的结构性优化

1. EditorUI01将`RealtimeInput`直接路由到interaction state；move/resize在frame boundary合并，press/release/cancel保持有序。该路径command registry visits、replay record和audit JSON均为0。
2. Editor08统一发布`CompiledCommandGeneration`，包含command、operation、chord、static event route、menu和palette indexes；所有入口预先生成`CommandRoute::{Command, Operation, DirectEvent}`。
3. Editor03只在成功且changed的authoring commit后推进document revision并生成versioned transaction entry；raw input、transient UI、failure和external side effects显式不可重放。
4. execution返回一个shared typed receipt和fixed invalidation mask；UI与audit借用同一receipt，不再深拷完整record或为内存预算遍历JSON。
5. Editor02把audit/plugin delivery变为可选、count+owned-bytes+deadline有界的派生流；需要wire/persistence时只在最终边界编码一次。
6. Editor12对一批command contributions锁外validate/build全部indexes，按base generation fence一次提交；失败publish为0，stable frame menu/predicate build为0。

## 静态验证与动态门

- `rustfmt --edition 2021 --check`：Event 36/36、Commands 17/17通过；scoped `git diff --check`通过，仅报告现有LF/CRLF提示。
- 四个相关Python契约模块执行16 tests，16/16通过，覆盖bounded runtime consumer pump、palette shared projection/query及paged keyboard路径。
- 未运行Rust/Cargo；associated error、Save All及replay变更仍缺current compiler/test证据。managed validator session已归档，禁止raw Cargo或伪造identity绕过。
- 没有current-source可执行文件，故F4、WPR/xperf、allocator/RSS、input-to-present、package power及RenderDoc parity均未执行。

两棵树继续留在`pending`。动态接受必须覆盖pointer 125/500/1,000 Hz、commands 1/100/10k、listeners 0/1/1k/10k、semantic changed/no-op/failure、replay disposition、menu/palette stable/change与plugin command batch；记录registry visits、JSON/clone bytes、lock wait/hold、revision/invalidation advances、queue bytes/age、CPU/wakeups、RSS、p50/p95/p99及package power。RenderDoc只用于事件cutover后的draw/pixel parity，不能证明CPU路由或replay正确性。
