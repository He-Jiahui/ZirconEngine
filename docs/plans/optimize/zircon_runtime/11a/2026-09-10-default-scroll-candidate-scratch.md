---
title: Runtime11A Default Scroll Candidate Scratch Reuse
category: zircon_runtime
report_id: Runtime11A-default-scroll-candidate-scratch-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Default Scroll Candidate Scratch Reuse

## Scope

This slice removes the repeated temporary candidate-vector allocation from the unhandled
default nested-scroll route in `UiSurface`. It does not change pointer hit selection,
scrollability rules, scroll offset semantics, dispatch ownership, or the broader Runtime UI
driver and window/input architecture tracked by 11A.

## Implementation

- `UiSurface` owns a runtime-only, serde-skipped `scrollable_candidate_scratch` buffer.
  `apply_default_pointer_scroll` takes the retained buffer, returns it after every success or
  error result, and clears it before retention.
- The existing Runtime `UiRuntimeTreeInteractionExt` now provides
  `collect_scrollable_candidates`. It clears the supplied buffer, validates every routed node
  in order, and preserves the previous enabled/visible/scrollable predicate and order.
  The legacy allocating `scrollable_candidates` API delegates to the same implementation.
- Candidate collection completes before the first `scroll_by` mutation. Consequently, a missing
  later route node still returns the prior `UiTreeError` before an earlier scrollable node can
  mutate, preserving the original full-validation behavior.
- Retained capacity above `MAX_UI_LAYOUT_DISCRETE_VALUE` is discarded after a route. Normal
  warm paths reuse their allocation; pathological route shapes do not permanently inflate the
  live surface cache.

## Regression and Local Evidence

- The tree regression verifies ordered collection, replacement of stale scratch contents, and a
  missing node after a valid scrollable candidate.
- The surface regressions dispatch initial and repeated wheel events to assert stable retained
  capacity. They also prove that a missing node after a valid scrollable candidate returns an
  error without changing that candidate's offset, then verify a later valid route can reuse the
  recovered scratch. The structural guard requires complete candidate collection before the first
  scroll mutation.
- The focused Runtime UI pointer/scroll/hot-path source-contract batch passed `73/73`.
- The final combined Runtime UI and Editor asset projection/watcher batch passed `123/123`
  across 19 Python contract modules.
- Python syntax, scoped `rustfmt --edition 2021 --check`, and scoped `git diff --check` passed.
- The broader 12-module batch ran 83 tests; 82 passed and one unrelated
  `test_runtime_ui_surface_input_publication_pressure` source-path guard failed because concurrent
  Runtime input routing moved from `runtime_ui.rs` into `runtime_ui/input_routing.rs`. Tooling is
  intentionally not changed in this slice.

These checks are source, contract, and operation-count evidence. Managed Runtime Cargo execution
and a Windows Release before/after workload with warm-up, allocation counters, and p50/p95/p99
remain required before performance acceptance can be claimed.

## Remaining Parent Work

11A still owns the unified Runtime UI driver, window lifecycle/input ownership, typed product
dispatch receipts, multi-surface composition, retained layout graph, and product/native timing
acceptance. This record does not close those parent gaps.
