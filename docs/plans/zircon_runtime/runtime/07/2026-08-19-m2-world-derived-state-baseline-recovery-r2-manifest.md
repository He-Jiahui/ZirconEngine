# Runtime07 F459 M2 Deterministic Baseline Recovery Manifest

Plan: `docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
Milestone: `M2`
Status: `deterministic_work_baseline_pending_managed_validation`

This r2 manifest replaces no prior artifact. The r1 M2 binding remains immutable and
snapshot-stale after a concurrent shared-source change, so this manifest binds the current
source state for a fresh managed validation run.

Files: [
  "zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs",
  "zircon_runtime/src/scene/ecs/mod.rs",
  "zircon_runtime/src/scene/world/performance_diagnostics.rs",
  "zircon_runtime/src/scene/world/derived_state.rs",
  "zircon_runtime/src/scene/tests/derived_state.rs",
  "docs/plans/zircon_runtime/runtime/07/2026-08-19-m2-world-derived-state-baseline-recovery-r2-manifest.md"
]

## Scope

- Publish deterministic derived-state counters through the existing ECS frame diagnostics store.
- Exercise direct-child hierarchies at 1, 1,000, and 100,000 entities without a wall-clock
  acceptance threshold.
- Record current full-work behavior for a single transform mutation: hierarchy validation and
  active propagation remain zero while world-matrix and NodeCache work remain whole-world.
- Preserve the r1 stale manifest and all foreign worktree edits unchanged.

## Required Evidence

- `rustfmt --check` and scoped `git diff --check` for the listed files.
- Managed Windows Cargo test evidence for the derived-state fixture family, using an approved
  non-C target root.
- The follow-up topology/dirty-root implementation must use these counters as before data and
  may not report an optimization until the affected-row values are measured.

## Explicit Non-Claims

- This is not the generation-owned dense topology or dirty-frontier implementation required by
  PERF-MVP-459.
- It does not claim an FPS, p95, power, allocation, or clone-byte improvement before a managed
  runtime profile has produced those values.
