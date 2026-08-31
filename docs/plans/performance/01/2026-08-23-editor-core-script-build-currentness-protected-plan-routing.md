---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-core-script-build-currentness-revalidation.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor core script-build性能计划路由（2026-08-23）

本记录通知既有owner，不修改受保护Performance01、`review.md`、`pending.md`或编号计划。

| owner | 必须吸收 |
|---|---|
| Performance01 / PERF-MVP-557 | 更新冻结为5/5、1,589行、52,563字节、26 tests、SHA256 `b4993c5e...8cdda`；删除已过时的无限debounce/无界VecDeque现状，P0改为generation identity、failure preservation和产品接线。 |
| Editor13 | request id与source generation分离；保留one active + latest pending；failure/cancel不得删除更新source；同source合并observers/Play waiter。 |
| Runtime13 | 编译immutable source generation，发布artifact+ledger digest；safe-point binding按artifact/runtime session生成精确receipt并拒绝stale。 |
| Editor14 / Runtime11 | 只用shared bounded job/process/I/O authority；无script私有pool；admission和result pages都有entries/bytes/age/deadline。 |
| Editor04 | Play等待required source对应的accepted binding generation，不以request完成或旧session binding恢复。 |
| Editor17 / PERF-MVP-644 | diagnostics使用count+byte bounded page和batch log ingress；retained ring容量不能替代producer ingress预算。 |

## 账本建议

- `zircon_editor/src/core/script_build/**`继续`static_complete / dynamic_pending /
  product_integration_pending`；外部production caller当前为0。
- 当前纯状态机contract 8/8与Rustfmt 5/5不能替代VM/job/Play/commandlet行为或F4性能证据。
- generation cutover、current managed Cargo和31-run F4 WPR CPU/wait/I/O/RSS/power完成前，不得加入
  `review.md`、提交性能完成commit或发送企微完成通知。

