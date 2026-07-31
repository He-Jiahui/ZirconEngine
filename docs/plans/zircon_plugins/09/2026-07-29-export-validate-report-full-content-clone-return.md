---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-07-29
summary_slug: export-validate-report-full-content-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
plan_link_mode: child_record_only
---

# export-validate-report-full-content-clone 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-07-29-export-validate-report-full-content-clone.md](../../performance/01/fixed-2026-07-29-export-validate-report-full-content-clone.md)
- 摘要：默认 validate report 已硬切到 compact schema v2，完整内容只由显式 schema v1 artifact 输出；Runtime bin 10/10、Plugins09 integration 2/2 与 Python consumer/CLI 87/87 通过。Performance01 可恢复后续规模矩阵，PF-M1 其他 catalog/profile 工作不由本记录关闭。
