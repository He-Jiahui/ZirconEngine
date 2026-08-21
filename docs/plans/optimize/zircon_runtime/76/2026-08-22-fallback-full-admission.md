# Runtime76 Fallback Full Admission

Plan: docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/76/2026-08-22-fallback-full-admission.md","docs/zircon_runtime/ui/layout/pass.md","docs/zircon_runtime_interface/ui/layout.md","zircon_runtime/src/ui/layout/mod.rs","zircon_runtime/src/ui/template/build/parsers.rs","zircon_runtime/src/ui/v2/surface_tree/parse.rs","zircon_runtime_interface/src/ui/layout/engine.rs","zircon_runtime_interface/src/tests/layout_engine_contracts.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime76-fallback-admission-m0-r1-be5a281c-20260822`
- Source items: `RUL-P0-001` and `RUL-P0-003`
- Acceptance gates: `RUL-GATE-002` and `RUL-GATE-008`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`UiLayoutEngineSelection::select(...)` applied the complete `unsupported_reason(...)`
capability check to the preferred backend, but admitted the fallback backend by family alone. A
fallback could therefore be reported as qualified while lacking requested content measurement or
DPI scaling, or while attempting to handle a Zircon-owned semantic family through Taffy.

The legacy and v2 explicit layout parsers also accepted any non-negative `usize` for grid tracks,
slot coordinates/spans, and virtualization overscan. Those authored values flowed into layout
vectors and index arithmetic without an admission budget.

## Scope Delivered

- Preferred and fallback capabilities now pass through the same complete admission function.
- A fallback is reported as `Fallback` only when family ownership, content measurement, and DPI
  requirements all pass; otherwise the route is reported as `Unsupported`.
- Existing selected-backend and preferred-failure diagnostic fields remain stable, so the change
  does not alter serialized report shape or valid fallback behavior.
- `MAX_UI_LAYOUT_DISCRETE_VALUE` defines one crate-owned upper bound of 4096 for explicit authored
  track counts, slot coordinates/spans, and overscan. The largest checked-in explicit grid uses 16
  tracks, leaving substantial product headroom.
- Both the legacy template builder and v2 surface-tree parser reject larger values before layout
  track vectors are allocated. Negative and non-integer diagnostics retain their prior behavior.

## Deterministic Performance Gate

The ignored release benchmark uses a malicious but memory-safe authored grid count of 65,536 and
performs 64 admission operations per sample. The legacy control parses the unbounded integer and
materializes the downstream `f32` track vector on every operation; the optimized production parser
rejects it before any track-vector allocation.

The gate warms both paths and records 21 alternating legacy/optimized sample pairs, with 11
legacy-first and 10 optimized-first pairs. Its marker includes raw unsorted nanosecond series and
nearest-rank P50/P95 values so the external validator can recompute every percentile. Structural
evidence is fixed at 64 legacy track allocations versus zero optimized track allocations per
sample. Acceptance requires `optimized_p95_ns * 4 <= legacy_p95_ns`, or at least 75% lower measured
P95. Actual timing values remain pending and are not claimed here.

## TDD And Static Evidence

- The new contract test covers three false-positive routes: missing content measurement, missing
  DPI scaling, and a Taffy fallback for Zircon-owned overlay semantics.
- Each case is deterministically `Fallback` on the prior family-only implementation and
  `Unsupported` after the production change.
- Existing valid fallback and unsupported-family tests remain in the same contract suite.
- Legacy and v2 parser regressions accept the exact 4096 boundary and reject 4097 with the authored
  field in the diagnostic. The prior parser deterministically accepts 4097.
- `rustfmt +1.94.1` completed for all five owned Rust files and scoped `git diff --check` completed.
- Focused tests, package checks, and external batch validation are pending. No Cargo pass or
  performance result is claimed.

## Remaining Scope

This closes the symmetric admission portion of `RUL-P0-003` and bounds explicit discrete values in
both template parsers. Responsive metadata paths, direct programmatic layout construction, checked
track/index arithmetic, backend build/version-derived capability matrices, compiled artifact
requirement binding, conformance corpus qualification, and product-scale performance evidence
remain open. The two delivered tasks will be validated together before coordinator submission.
