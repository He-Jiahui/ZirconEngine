---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/preview.rs
  - zircon_editor/src/ui/host/editor_asset_manager/preview/visibility_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-27-preview-scheduler-visible-set-removal.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/preview/visibility_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor04 preview scheduler visible set removal

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04 | Remove the private write-only resident visible set while preserving invisible admission, dirty tracking, the 64-job cap and job-token behavior | `implemented_pending_validation` | Invisible-request/token and scheduler-footprint regressions authored before the minimal removal; scoped Rustfmt, diff and text checks passed. Grouped managed compile/test and ignored Windows Release `EDITOR04_PREVIEW_VISIBLE_SET_100K_BENCH_V1` pending: 100,000 matching dirty UUIDs per pair, five alternating warmup pairs, 31 alternating measured pairs, raw paired samples and p50/p95/p99, request-sweep p95 <= 80% of legacy. Legacy retains 100,000 visible UUIDs; current scheduler has no resident visible set. This does not accept manager completion/refill, Editor04 catalog memory/frame gates or Editor57 native G37 p99. |
