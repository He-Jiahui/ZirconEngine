---
status: implementation_complete_core_min_validated_default_validation_blocked
date: 2026-08-22
owner_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
related_code:
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order_tests.rs
  - zircon_runtime/src/core/runtime/tests/registration/behavior/module_order.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md#perf-mvp-325
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/bevy/crates/bevy_ecs/src/schedule/graph/dag.rs
---

# Runtime02 Module Graph Iteration Performance Plan

## Problem and Baseline

The frozen runtime module graph already has a single-generation `Arc` cache in
`CoreHandle`. Its graph walks were still structurally recursive:

- module activation ordering recursed through dependencies;
- service activation ordering recursed through dependencies;
- activation and dependent closure collection recursed through the frozen maps.

For a dependency chain of depth `D`, each path consumed `O(D)` native stack.
The existing borrowed-name module index and index stack removed normal-path
temporary names, but did not remove stack overflow risk. Descriptor cloning in
`CoreHandle::frozen_module_graph` is a separate `PERF-MVP-322` arena-ownership
scope and is not counted as fixed by this work.

## Reference-Grounded Decision

Unreal loads module descriptors by declared loading phase and preserves the
phase-ordered lifecycle boundary. Bevy stores one cached topological result and
recomputes only when its graph becomes dirty. Zircon freezes registration before
lifecycle work, so its immutable graph cache is the corresponding generation
boundary; no registration-time incremental edge path exists after the freeze.

The replacement is explicit-frame DFS, not a Kahn ready queue. The existing
contract visits roots in `(init_level, descriptor index)` order and each service
dependency in lexical order. An unconstrained Kahn queue can reorder unrelated
ready nodes. Explicit frames retain the exact current order and cycle path while
removing native recursion.

## Algorithm Contract

| Path | Normal storage | Work | Error behavior |
|---|---|---|---|
| Module sort | borrowed `HashMap<&str, usize>`, `Vec<Option<VisitState>>`, frames bounded by depth | `O(M log M + E)` because roots retain `(init_level, descriptor index)` ordering | materialize names only for a detected cycle |
| Service sort | declaration-order frozen lists for validation, then compact lexical edge indices with borrowed name index and depth-bounded frames | `O(S + E + sum(d_v log d_v))`, where `d_v` is a service out-degree | materialize names only for a detected cycle |
| Activation/dependent closure | borrowed `HashSet<&str>` plus iterative worklist | `O(V + E)` | output still owns only the requested ordered names |

The stable cycle position is found by scanning active frames only after a back
edge. Normal paths neither reserve a frame buffer proportional to the whole
graph nor retain a cycle-position table. Services preserve declaration order
through validation, then construct sorted compact edge indices for traversal,
matching the former per-visit lexical sort without changing diagnostic priority.

## Evidence Plan

1. Preserve module order, missing-edge, init-level, and exact cycle diagnostics.
2. Add service lexical-order and activation/dependent closure regressions.
3. Run module and service `100,000`-node deep-chain regressions. Success proves
   frame depth is heap-managed and native-stack depth is constant.
4. Run the managed Windows `zircon_runtime` lib-test gate, then capture the
   same workload under WPR/xperf profiling. Record elapsed time, peak working
   set, native stack depth, graph visits, and allocation counts where the
   profiler exposes them.

`wpr.exe` and `xperf.exe` are available on this Windows host. At record time the
managed compatible `zircon_runtime` pool was busy with job
`7da5ac7b887a4ad58bf0294954efd0c3`; at that initial planning point no Cargo
process, p95, RSS, or power result had been claimed. The later core-min gate
and process-sampling evidence are recorded below. Full-default-feature dynamic
evidence remains blocked pending the UI repair and a host policy that permits
WPR recording.

## Validation Status

- Complete: source formatting, whitespace validation, and independent static
  review found no P0/P1/P2. The review also corrected the documented costs to
  `O(M log M + E)` for stable module roots and
  `O(S + E + sum(d_v log d_v))` for lexical service edges.
- Complete: new regression coverage specifies declaration-order service
  diagnostics, stable lexical activation, iterative closures, and module/service
  `100,000`-node deep chains.
- Complete: managed Windows core-min validation job
  `011cb2878e544e5bba17966191096c11` ran `cargo build` and the filtered
  `cargo test` successfully with exit code `0`. Its D-drive target used
  `--no-default-features --features core-min`; the full test crate built in
  9m 34s and then ran the `module_order` filter.
- Blocked externally: managed Windows validation job
  `0cc224e1cbb74c0b962706fd30111c49` compiled with the D-drive coordinator
  target but failed before tests ran because the UI worktree has 29 unrelated
  lib-test errors. The precise handoff is
  `failure-2026-08-22-module-graph-validation-ui-blockers.md`.
- Pending after that repair: rerun the filtered test, then collect WPR/xperf
  full-default-feature evidence before milestone closeout. No power comparison
  or cross-engine equivalence claim is made before a supported system trace.

## Performance Evidence

The core-min test executable from the successful managed gate was run as a
profiling workload, once per fresh process, with each `100,000`-node test name
as the only harness filter. The host had concurrent workspace activity, the
binary used Cargo's unoptimized test profile with debug info, and process
startup is included. These are regression baselines, not release performance
or a comparison with Unreal or another engine.

| Workload | Samples | Median | p95 (nearest rank) | Maximum observed working set |
|---|---:|---:|---:|---:|
| Module activation deep chain | 11 | 914.570 ms | 1158.739 ms | 60,919,808 B (58.098 MiB) |
| Service activation deep chain | 11 | 1592.514 ms | 2202.621 ms | 74,858,496 B (71.391 MiB) |

The first measurement attempt was discarded because `Start-Process` does not
retain `PeakWorkingSet64` after process exit. The retained measurements poll
the child `WorkingSet64` at roughly 5 ms intervals, so reported working sets
are conservative observed peaks rather than allocator totals. All 22 sampled
processes exited successfully.

WPR `GeneralProfile.light` plus `Power.light` was attempted in memory mode with
`D:\ZirconBuilds` as its temporary target, but Windows rejected the performance
policy with `0xc5585011`; `wpr -status` confirmed that no recording remained.
Therefore no ETW trace, allocation count, native-stack sample, or attributable
power value exists. The installed `wpr.exe` and `xperf.exe` are sufficient only
after the host policy is enabled. The old recursive implementation has no safe
`100,000`-depth baseline because it would consume the native stack, so the
quantified result establishes stack-safe completion rather than a speedup ratio.
