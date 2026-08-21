Plan: docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
Milestone: P1-7 profile path admission and P2-3 periodic diagnostics projection
Status: completed

# Runtime03 current diagnostics snapshot and profile path admission

## Delivered

- Profile export session IDs are reduced to one portable child basename. Empty/dot-only names,
  separators, trailing dots and Windows device basenames can no longer escape or invalidate the
  configured output root.
- Periodic dynamic-session logging collects a current-only diagnostic projection. It preserves
  path, unit, current, EMA, minimum and maximum while omitting retained history, subsystem tags and
  the global profile timeline.
- Derived render/physics/animation series are recorded through the authoritative diagnostic store;
  full diagnostic/tool snapshots remain available to their existing callers.

## Performance evidence

- Full store copy passes per collection: `2 -> 1`, a `50%` reduction.
- Periodic profile samples cloned per tick at default limits: at most `20,992 -> 0`, a `100%`
  reduction.
- Periodic retained diagnostic measurements cloned per tick at the reviewed 541-series, 64-history
  scale: at most `34,624 -> 0`, a `100%` reduction.
- Series traversal/formatting, path filtering, delta publication, rate/byte budgets and dropped
  reports remain open under P2-3. Atomic multi-artifact publication and a capture manifest remain
  open under P1-7.

## Validation

- Coordinator copy `3a1de0a8d2fe47e9b809ea4b355f2c84`, input manifest
  `dee829a080d5519f05d6cbe5c8d8a96e3b931a523375251f9d1a57ccd937bd29`, ran the current source as
  receipt `4a381be1df1e43caaf4c07345c15e2a9`.
- The shared profile-session basename contract passed `2/2`; the Runtime03 current-snapshot/profile
  export regressions passed `2/2`.
- The same receipt passed the Runtime, Runtime Interface and Editor all-target feature checks in
  one 917.188-second stage. Its later Pester failure occurred after these gates and did not alter
  their source snapshot or result.
- This record completes only the delivered profile-path and current-only projection slice. The
  traversal, filtering, publication-budget and capture-manifest work listed above remains open.
