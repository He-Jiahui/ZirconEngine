---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-protected-derived-component-authority.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
---

# Runtime1021: constant-space dirty projected reads

Status: `candidate_static_review_complete_managed_validation_pending`.

## Scope

Dirty world-matrix projection now folds transforms while walking from the queried node toward its parent. Each local matrix is prepended to the accumulator, preserving root-to-child composition order without retaining a lineage vector. Dirty active projection uses the same parent walk shape. Both paths use Floyd slow/fast pointers to detect a multi-node cycle with constant auxiliary storage and fail closed (`None` for a world matrix and `false` for active-in-hierarchy).

`parent_for_read` remains the edge authority. A self-parent, missing parent, or absent parent component ends the readable chain, preserving the existing behavior: world projection returns the current node's local transform and active projection returns that node's `ActiveSelf` value. A missing queried entity still returns no world matrix.

The slice removes heap scratch from these dirty projections. It does not introduce a derived-value cache, change derived-component ownership, or modify authoring and mutation APIs. Each query still scans its ancestor chain. Floyd detection adds parent probes, so time remains O(depth) with a larger constant factor; auxiliary space is O(1). This does not satisfy Runtime62 RSH-G19's O(1) query requirement.

## Behavior and performance gates

`projected_reads.rs` now covers dirty reparent projection against post-flush values, a 128-node transform and active chain, self-parent and missing-parent termination, and fail-closed two-node cycles. These checks exercise observable results rather than source spelling.

The Windows-only ignored integration profile uses a real test-binary `GlobalAlloc` wrapper around `System`. In Release it samples dirty `world_matrix` and `active_in_hierarchy` queries at depths 1, 32, and 1,024, with 31 batches of 256 calls per query and depth. It emits raw batch elapsed times, per-query latency samples, p50/p95/p99, allocation counts, and allocated bytes; it asserts zero allocations and zero allocated bytes for every measured batch. Setup, sample collection, and reporting occur outside the allocation window.

No Release profile result or managed Rust test result is claimed here. The profile has not been run. Its depth set does not cover the Runtime62 100K/1M scale gate, and this slice defines no latency ceiling or before/after baseline. RSH-G19 and G24 remain open; zero-allocation evidence alone cannot close either gate.

## Validation and source attribution

`rustfmt --check` and scoped tracked-file `git diff --check` passed. Direct final-newline and trailing-whitespace checks passed for every source, documentation, manifest, and inverse artifact; the inverse also passes `git apply --check`. Cargo was not run. The Batch U asynchronous validation request remains admission-unknown and has no build or test result; it is not acceptance evidence for Runtime1021. The exact Batch U preimage SHA-256 values and this candidate's postimage hashes are sealed in `.codex/state/session-coordinator/async-validation-batches/2026-09-28-runtime1021-dirty-projected-read-source-manifest.json`; the reversible source/document inverse is `docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime1021-dirty-projected-read-streaming.inverse.patch`. Preexisting changes in `derived_state.rs` were preserved. Coordinator lease and attribution receipts remain pending because coordinator status and tooling were outside this slice.
