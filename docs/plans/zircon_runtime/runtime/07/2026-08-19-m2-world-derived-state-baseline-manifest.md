Plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
Milestone: M2
Status: deterministic_work_baseline_pending_managed_validation
Files: ["zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs", "zircon_runtime/src/scene/ecs/mod.rs", "zircon_runtime/src/scene/world/performance_diagnostics.rs", "zircon_runtime/src/scene/world/derived_state.rs", "zircon_runtime/src/scene/tests/derived_state.rs", "docs/plans/zircon_runtime/runtime/07/2026-08-19-m2-world-derived-state-baseline-manifest.md"]

# Runtime 07 M2 world derived-state work baseline manifest

## Scope

This slice establishes deterministic work counters for
[`failure-2026-07-22-world-derived-state-full-rebuild.md`](failure-2026-07-22-world-derived-state-full-rebuild.md).
It measures the current hierarchy validity, topology recovery, active propagation, world-matrix
propagation, and node-cache rebuild work before the topology and dirty-frontier hard cutover.

## Required managed gates

- The 1, 1,000, and 100,000-node direct-child fixtures must report stable row and parent-chain
  work counts through `EcsFramePerformanceDiagnostics`.
- A single transform change after synchronization must demonstrate the current full matrix and
  node-cache rebuild cost without rerunning hierarchy validity or active propagation.
- The counters must publish through the existing diagnostic store and core handle path; no wall
  clock threshold may stand in for a work-count assertion.
- The coordinator must compile and run the focused Runtime tests from an approved non-C target
  root and bind the exact source hashes to this manifest.

## Exclusions

- This measurement slice does not add a second hierarchy truth, a parent fallback, a compatibility
  facade, or a full-world clone.
- The later topology generation, merged dirty-root frontier, and incremental node/render/inspection
  consumers remain a separate hard-cutover slice after this baseline is accepted.
