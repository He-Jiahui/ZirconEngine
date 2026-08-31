---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-core-tool-scheduler-liveness-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor core tool scheduler活性受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：将`zircon_editor/src/core/tools/**`更新为current 4/4文件、1,416行、17 tests；注明set清空后single promotion、三值set去树化与topic parse M0静态完成，但Cargo/真实consumer/WPR/allocator/power仍open。本Session不直接编辑受保护ledger。
- Editor08 M3 + Editor05/15：记录三条set transition入口已统一“set queue empty -> promote all available single resources”，并新增三条行为门；真实scene mode/export wizard必须同里程碑接入唯一service，排队状态不得启动process或接管viewport input。
- Editor08 + Editor03/05：按Unreal InteractiveToolManager/EditorModeTools补齐tool generation、active/pending/retired lifecycle、accept/cancel、transaction和input-router owner；不得让`HandleToolRegistry`、scene mode与export各建私有仲裁器。
- PERF-MVP-019 + Editor02：记录内建tool topic parse从1/operation降为1/service；per-event topic/event clone、global bus lock、bounded page与slow subscriber继续由message generation/inbox owner收敛。
- Editor08规模门：保留Q<=64时简单VecDeque；先测`release_all` O(Q^2) bounded cancel path，只有p95超预算才以single-pass retain/rebuild替换，不预建复杂index。
- `docs/plans/performance/review.md`：只有3条Rust行为门在current Cargo执行、Editor05/15产品接线、1M operation/unload矩阵和WPR/allocator/power通过后迁入；该非渲染切片不要求RenderDoc。本轮不迁移、不提交milestone、不发送完成企微。
