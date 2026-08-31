---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-core-asset-currentness-refactor-and-builtin-base-m0.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor core Asset性能计划路由（2026-08-23）

本记录只通知现有owner计划吸收合同，不改受保护`review.md`、`pending.md`和编号总计划。
`core/asset/**`已完成38/38 current static reconcile和builtin base M0，但动态验收未通过。

## Owner合同

| owner | 必须吸收 |
|---|---|
| Editor03 | delete/rename/move只提交undoable typed intent并消费generation receipt；不得直接拥有filesystem或Runtime registry rollback。 |
| Editor09 | 删除Editor完整ProjectManager/第二reference graph truth；mutation只按Runtime affected delta更新catalog/details/preview，full refresh=0。 |
| Editor10 | 把delete/relocation preflight升级为promotable generation ticket；admission与commit不重复全拓扑query/sort/clone。 |
| Editor12 / Plugins01 | builtin base升级为immutable shared base + plugin overlay generation；一次batch validation/materialization/publish，删除逐candidate完整clone/replay。 |
| Editor14 / Runtime11 | explicit save、Save All、autosave使用同一bounded streaming coordinator；queued/running/result resident bytes、age、deadline、cancel和UI affinity全部显式。 |
| Runtime04 | 唯一immutable project/asset generation store；stable reverse/ordered index和paged query；删除`ProjectManager: Clone`、deep snapshot及mutation candidate full clone。 |
| Frameworks01 | source/meta/registry/resource mutation只允许一个durable transaction owner，具备exact rollback ownership和restart recovery。 |

## 账本建议

- `zircon_editor/src/core/asset`保持pending；38/38 static review不等于动态验收。
- builtin base只可记录：process build=1；后续每次`with_builtins` contribution build/validate/apply
  `26 -> 0`。registry deep clone仍存在，不得登记为zero allocation或模块完成。
- 新refactor方向可保留Runtime-owned topology/fail-closed策略，但Editor锁内clone/sort、重复preflight、
  Runtime ProjectManager candidate clone和mutation后full refresh必须在接受前消失。

## 动态前置

current-source managed Windows editor可运行后，执行WPR/xperf CPU/File I/O/waits/locks/CSwitch、
allocator/RSS与package power；至少31个可比F1/F4 save/import/delete/rename样本并报告p50/p95/p99/CI/
effect size。RenderDoc仅用于thumbnail/Browser/首帧GPU parity，不能验收CPU topology、filesystem或锁。
这些门通过前不创建性能里程碑commit，也不发送企微“已完成”量化消息。

