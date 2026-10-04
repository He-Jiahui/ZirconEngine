---
related_code:
  - zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/57/2026-09-27-virtual-group-single-window.md
tests:
  - zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor57 virtual group single window start

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor57 | Compute the scroll window start once per visible virtual group append and defer item lookup until after clipping, preserving slot and damage behavior | `implemented_pending_validation` | Behavior regression and `EDITOR57_VIRTUAL_GROUP_SINGLE_WINDOW_BENCH_V1` queued for the combined Editor batch. Release p95 targets: ≤ 90% of retired at 48 groups and ≤ 80% at 240 groups. Full Asset Browser product p99 remains pending. |
