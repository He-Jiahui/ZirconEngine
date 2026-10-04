---
related_code:
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots/preview_demand_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/57/2026-09-27-viewport-preview-demand.md
tests:
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots/preview_demand_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor57 viewport-bound asset preview demand

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor57 | Limit preview demand to the measured Asset Browser/Activity scroll window, two overscan rows and offscreen selection; admit selections from both surfaces before visible rows, deduplicate requests, and use the committed Browser pointer generation on scroll | `implemented_pending_validation` | 100,000-item initial/middle/clamped-tail list and thumbnail regression, selected-first order and two-surface dedup, plus Activity folder/unknown-geometry checks queued for the combined Editor batch. Ignored Release marker `EDITOR57_100K_PREVIEW_DEMAND_BENCH_V1` compares all-filtered and viewport UUID candidate stages with five alternating warmup pairs, 31 alternating measured pairs, raw samples, p50/p95/p99, and a pending viewport p95 <= 25% of old p95 target. Backend chrome construction, per-item scheduler work, native present and ED57-G37 product p99 remain pending. |
