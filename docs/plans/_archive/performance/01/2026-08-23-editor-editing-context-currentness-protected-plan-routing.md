---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-editing-context-currentness-revalidation.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor Editing与Context性能计划路由（2026-08-23）

本记录只通知现有owner计划吸收合同，不修改受保护`review.md`、`pending.md`和编号总计划。
`core/editing/**`已完成29/29、`core/context/**`已完成5/5 current static reconcile；两者均未动态验收。

## Owner合同

| owner | 必须吸收 |
|---|---|
| Editor00/01 | 建立immutable authoring generation与分阶段Context assembly receipt；只发布完整context，删除callback-shaped mutable authority。 |
| Editor02 | 唯一bus拥有zero-target fast path、bounded fanout/backpressure和compact receipt；producer不重复JSON/projection。 |
| Editor03 | `PreparedEditBatch + inverse + generation commit receipt`；scope/Drop无等待；history同时按entries/bytes/resident/age admission；journal锁外编码。 |
| Editor05 | hierarchy/gizmo/transform走同一atomic batch；parent/name/transform partial mutation和full-state field edit归零；move-only subtree delete消费Runtime preflight ticket并保证至少保留一个camera。 |
| Editor08 | 保留bounded scheduler和construction-time topic cache；逐事件clone/fanout只通过Editor02 bus合同解决，不增加第二队列。 |
| Editor14 / Runtime11 | immutable generation上调度prepare、journal encode和独立I/O；UI同步等待、无deadline Condvar和worker后wait均为0。 |
| Editor17 | logging/recovery作为Context唯一owner；启动/关闭均有bytes/deadline/cancel/terminal receipt，不重复scheduler。 |
| Runtime07 | authoring commit是唯一短mutation owner，发布exact changed ranges；stable frame world lease/query=0。 |
| EditorUI08 | 只消费commit generation affected delta；一个edit对应一个inspection/render generation，无broad rebuild。 |

## 账本建议

- `zircon_editor/src/core/editing`保持pending：34文件复核中的Editing部分虽静态完成，但原子fault、wait、
  retention、WPR和产品门未过。
- `zircon_editor/src/core/context`保持pending：旧constructor blocker可标为source-closed，但F0 stage、fanout、
  current managed tests和产品profile仍未过。
- tool topic只可登记“构造后每publish batch parse `1 -> 0`”；不得登记event clone、bus fanout或总allocation为0。
- move-only delete可登记NodeRecord subtree clone path removed，但
  `failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md`关闭前不得接受该路径。
- 当前Python 22项为16 pass/6 stale contract failures，rustfmt为33/34；不得写成模块test/format全绿。

## 动态前置

先产出approved D/E/F target中的current-source editor executable，再运行至少31个可比F0/F4样本：
WPR/xperf CPU、wait/lock、CSwitch、File I/O、allocator/RSS和package power。报告p50/p95/p99、CI、effect
size和环境。RenderDoc只关联authoring generation到首呈现帧，检查draw/pass/GPU/像素与重复submission，
不验收CPU事务或功耗。全部通过前不创建性能里程碑commit，也不发送企微“完成”量化消息。
