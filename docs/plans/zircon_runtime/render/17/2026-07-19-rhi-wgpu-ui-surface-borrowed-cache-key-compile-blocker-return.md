---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-07-19
summary_slug: rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
plan_link_mode: child_record_only
---

# rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-07-19-rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker.md](../../runtime/12/fixed-2026-07-19-rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker.md)
- 摘要：Runtime12 M4 canonical compilation may resume past the Render17 ui-surface borrowed-key blocker. The separate Render17 pairwise-overlap batching failure remains open and is not absorbed by this return.
