---
title: Animation State Machine Pose Capability Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-animation-state-machine-pose-capability-current-source-algorithm-performance-review.md
---

# Animation State Machine Pose Capability Current-Source Protected Plan Routing

## Review ledger status

State-machine, blend-space, layer, GPU-skinning, IK and mask production sources completed current-worktree static review: **50/50 Rust files**, fingerprint `f1f45e34bdf267a817b006a0fe991794e09b27dd7748384f9fbfd035a2ccc392`. Protected ledgers remain unchanged pending managed Rust and dynamic product evidence.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| BlendSpace2D compile is at least O(N^4); query rebuilds hull maps | Plugins13 + Editor82 + Runtime08c | Adopt validated Delaunay precompute, adjacency/hull artifact and cached-triangle query. |
| State/layer runtime clones maps, strings and full poses for rollback/intermediates | Plugins13 + Runtime08c + Editor76 | Compile one indexed program and use tentative dense state/pose-arena commit. |
| GPU skinning has no renderer caller and clones whole palettes | Plugins13 + Runtime09b | Bind versioned dirty-range palette extraction to the renderer or mark unavailable. |
| IK inbox was removed; remaining solvers have no product owner | Plugins13 + Runtime08c + Runtime08a | Compile pose-modifier nodes into one animation/physics schedule before advertising integration. |
| Mask compiler and layer weights are separate paths | Plugins13 + Editor82 | Produce and consume one immutable dense mask artifact. |
| No significance/LOD/time-budget policy | Plugins13 + Runtime65 + Editor69 | Add measured skip/interpolate/reduced-work policy with correctness gates. |

## Acceptance routing

Implementation order is capability truth -> validated BlendSpace2D artifact -> indexed state/layer program -> tentative commit -> scheduled modifier/skinning integration -> budget and dynamic qualification. No local math optimization closes missing product reachability.

Dynamic acceptance records input scales, compile/query/update/evaluate p50/p95/p99, scale slopes, allocations/bytes, cached-triangle steps, pose-arena misses, transition/event deferrals, palette dirty bytes/uploads, CPU, GPU, wakeups, RSS and power. RenderDoc is required only once visible skinned draws exist. No milestone commit or WeCom completion message is warranted yet.

