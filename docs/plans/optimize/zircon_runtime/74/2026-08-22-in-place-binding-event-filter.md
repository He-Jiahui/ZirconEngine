# Runtime74 In-Place Binding Event Filter

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-in-place-binding-event-filter.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-045`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`UiSurface::apply_pointer_binding_targets` moved every component event into a newly allocated
retained vector, then discarded the caller's original vector allocation. This occurred on every
non-empty compiled binding dispatch even when every event survived, forcing one allocation plus a
full move of the event batch and preventing the dispatch result from reusing reserved capacity.

## Scope Delivered

- The executor now uses stable in-place `Vec::retain_mut` compaction. Published target events and
  target-free pass-through events retain authored order.
- Missing, stale, mismatched, or rejected compiled endpoints are removed in place while keeping the
  existing rejected reports and publication rules.
- A `UiTreeError` still clears the complete event batch before returning the error, matching the
  previous externally visible failure state.
- The allocating implementation remains available only under `cfg(test)` and calls the same
  single-event processor, so the performance comparison changes only buffer ownership.

## Performance Contract

`pointer_binding_target_in_place_filter_p95_beats_allocating_filter` runs 21 paired samples and
alternates which implementation executes first. Each sample filters 32 batches of 512 real
`UiPointerComponentEvent` values. One event per batch executes a compiled target transaction and
the other 511 follow the target-free pass-through path. The release marker emits both raw sample
arrays and nearest-rank P95 values. External validation must independently recompute P95 and enforce:

- optimized P95 is at least 15% lower than the allocating baseline P95;
- event-buffer allocations per sample fall from 32 to 0, a 100% reduction;
- sample count is exactly 21 and first-run order alternates 11 legacy-first / 10 optimized-first.

Measured values remain pending until the coordinator validation copy completes. No performance or
Cargo pass is claimed in this record yet.

## Regression Contract

- A direct executor test reserves excess event capacity, applies one target-bearing event plus one
  pass-through event, and requires the pointer, capacity, order, and report count to remain stable.
- Existing atomic commit, rollback, stale-endpoint, payload, focus, and target-free fast-path tests
  continue to exercise the same single-event decision owner.

## Remaining Scope

This slice does not remove report-vector allocation, intern binding display strings, add frame
safe-point batching, or implement model subscriptions. Those remain separate Runtime74 work items.
