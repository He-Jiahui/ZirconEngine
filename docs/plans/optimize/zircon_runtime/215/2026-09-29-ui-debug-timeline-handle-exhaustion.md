---
title: Runtime215 UI debug timeline handle exhaustion
category: zircon_runtime
report_id: runtime215-ui-debug-timeline-handle-exhaustion-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/215-runtime-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-current-working-tree-review.md
implementation_status: source_candidate_pending_managed_validation
validation_status: rustfmt_and_scoped_diff_check_passed
regression_status: stronger_full_retention_applied_managed_validation_pending
performance_status: correctness_fix_no_performance_claim
---

# Runtime215: prevent UI debug timeline handle reuse

`UiDebugTimelineStore::capture_snapshot` used saturating arithmetic with a
nonzero fallback. Once `next_handle` reached `u64::MAX`, each later capture
reused the same public frame identity. A lookup could then resolve an older
retained frame for a handle that also named a newer capture.

The store now holds its next handle as `Option<u64>`. It can issue the final
`u64::MAX` handle once; checked increment marks the store exhausted, and the
next capture fails before changing retained frames or selection. The stronger
boundary regression now fills a capacity-two store with handles
`u64::MAX - 1` and `u64::MAX`, selects the earlier handle, then verifies an
exhausted capture leaves the complete timeline snapshot unchanged. This covers
both full-window eviction and preservation of a non-latest selection.

## Evidence and remaining gates

- Added `ui_debug_timeline_handle_exhaustion_does_not_reuse_the_final_handle`
  in `zircon_runtime/src/ui/surface/timeline/handle_range_tests.rs`.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` passed.
- The stronger regression is applied in shared source (SHA-256
  `033b05c3acea18285f57c504e2136786091851dcfab41c5158d5499686faa7fb`).
  Scratch rustfmt and source-patch applicability checks passed; the
  LF-normalized applied text matches that reviewed candidate. The regression
  has not run because managed Runtime validation remains pending.
- No Cargo command ran. Managed Runtime tests remain pending; this is a
  correctness fix, with no performance improvement claimed.
