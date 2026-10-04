---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-arranged-visibility-scratch-reuse.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_runtime/src/ui/surface/arranged_visibility.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
tests:
  - zircon_runtime/src/ui/surface/arranged_visibility.rs
  - tools/tests/test_runtime_ui_arranged_visibility_index_performance_contract.py
---

# Arranged Visibility Resolution Scratch Reuse

`UiArrangedVisibilityIndex::rebuild` now retains the iterative resolver's state and ancestor
path buffers. Warm arranged-tree rebuilds reuse both allocations, while a shrink trims capacity
to a two-times-current-node budget. Cloning copies only published visibility data, not scratch.
Published visibility bits and fail-closed behavior are unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M32 | Reuse and bound arranged-visibility resolution scratch | implemented_pending_validation | Rust regression is present for warm capacity/pointer reuse, high-water shrink, clone scratch omission, and visibility preservation (Cargo execution deferred to managed testing); post-change combined Runtime UI/Editor contract batch `86/86`, Python syntax, and scoped Rustfmt pass. New source/plan files have no trailing whitespace; six older malformed lines remain in the shared async log. Managed Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending. |
