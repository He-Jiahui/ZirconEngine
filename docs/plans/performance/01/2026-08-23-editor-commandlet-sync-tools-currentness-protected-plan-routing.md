---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-commandlet-sync-tools-currentness-and-world-sync-clear-m0.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor Commandlet、Sync与Tools性能计划路由（2026-08-23）

本记录通知现有owner计划吸收合同，不修改受保护`review.md`、`pending.md`或编号主计划。
12/12 Rust文件已current static reconcile，但三组均缺current Cargo与产品剖析，继续保持pending。

| owner | 必须吸收 |
|---|---|
| Editor16 | 一个ProcessArgvOwner一次解析；Commandlet token从同一immutable registry generation直接解析；argv整表复制为0。 |
| Editor02 | WorldSync使用bounded batch/bytes/elapsed continuation；fact不逐条JSON+bus lock；dirty views按tick一次批量publish。 |
| Editor05 | hierarchy只消费generation-bound exact changed items；full rebuild仅显式overflow/recovery；产品帧记录apply budget和lag。 |
| Editor08 | 拆分input capture、modal stack、scene-mode owner；跨资源操作使用组合ticket，删除不相交资源的全局FIFO阻塞。 |
| Editor14 / Runtime11 | backlog prepare/encode可下沉worker；main thread只做bounded apply；所有等待/取消/关闭有deadline和terminal receipt。 |
| Editor01 / Runtime03 | editor tick给WorldSync分配可观测预算并携带continuation；不得在单帧无界drain全部transport backlog。 |
| Runtime10 | ABI drain提供batch generation、encoded bytes、oldest age及bounded cursor；replacement清理不物化废弃token snapshot。 |

## 账本建议

- `zircon_editor/src/core/commandlet`保持pending：静态复核完成，argv/output owner和subprocess规模门未过。
- `zircon_editor/src/core/sync`保持pending：已关闭generation clear临时N-token snapshot，但每tick无界drain、
  per-fact JSON/publish和动态门未过。
- `zircon_editor/src/core/tools`保持pending：队列有界且并发promotion修复已对账，但全局head-of-line、
  per-event clone/publish和F4输入产品门未过。
- 动态里程碑完成前不得提交性能完成commit或发送企微完成通知。

