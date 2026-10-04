---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-editor-pointer-surface-delta-receipts.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-viewport-toolbar-pointer-surface-reuse.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs
tests:
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/surface_contract.rs
---

# Viewport Toolbar Stable-Topology Cache Pruning

`ViewportToolbarPointerBridge::sync` previously built a surface-key `BTreeSet`
and retained both per-surface caches for every changed layout, including a
frame-only update whose ordered keys were already proven unchanged. Unknown
surface keys are rejected before cache mutation, and a key add/remove/reorder
already selects the explicit topology branch. Cache pruning now remains in that
topology branch, while stable geometry proceeds directly to its exact frame
delta.

This removes the stable-path key-set allocation/construction and both cache
retention scans without weakening topology cleanup or pointer route ownership.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E32 | Restrict Toolbar surface-key cache pruning to explicit topology changes | implemented_pending_validation | The source-contract condition was RED before the move and GREEN after it; scoped rustfmt and diff checks pass. Editor performance-contract batch passed 548/548. Managed Rust compilation and Windows Release allocation/time p50/p95/p99 evidence remain pending, so no product latency target is claimed. |

## Coordinator dispatch log

- 2026-09-10: coordinator accepted the sealed Runtime/Editor static-format batch as
  ticket `ccd5411532074737832a87c26bf5414b`. The snapshot includes this Toolbar
  source/test pair; it is intentionally not polled while independent repair work
  continues.
