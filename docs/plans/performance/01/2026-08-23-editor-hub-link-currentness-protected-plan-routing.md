---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-hub-link-currentness-and-single-read-m0.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor Hub link性能计划路由（2026-08-23）

本记录通知既有owner吸收currentness，不修改受保护账本、Performance01主计划或编号owner计划。
`PERF-MVP-643`目前只存在于2026-08-16 routing文档，尚未进入Performance01主计划，不能视为已分派。

| owner | 必须吸收 |
|---|---|
| Performance01 | 新增/合并`PERF-MVP-643`为P0；当前冻结6/6、790行、27,772字节、4 tests、SHA256 `7211487a...47cc4`。 |
| Editor10 | `ProjectAuthority/SessionGuard` commit先于recent persistence；history失败只产生retry/diagnostic receipt，不回滚可用项目；复用选中项目的authoritative validation。 |
| Editor14 | 提供唯一ordered projection lane；按canonical key latest coalesce，绑定entries/bytes/age/deadline/cancel/retry/shutdown flush，不建Hub专用pool。 |
| Editor16 | record/remove改为typed intent；跨进程lease有限；发布一个`HubRecentProjectsGeneration`；保留event-driven focus、one-shot handshake和Editor17 liveness边界。 |
| Render17 | F0/F1分别标记read/decode、row probe、intent admission、lease wait、merge/encode/write、retry和UI generation apply。 |

## 账本建议

- `zircon_editor/src/core/hub_link/**`继续pending。single-read M0只在registry存在时把read前文件系统调用
  从2降到1；无限等待、主线程transaction coupling与重复probe仍未验收。
- current managed Cargo、two-process contention、31-run F0/F1 WPR CPU/RSS/file-I/O/power均为必需门。
- 动态里程碑完成前不得加入`review.md`，不得提交性能完成commit或发送企微完成通知。

