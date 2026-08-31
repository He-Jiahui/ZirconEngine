---
title: Runtime02 Mesh SDF Executor and BVH Capacity
category: zircon_runtime
report_id: Runtime02-mesh-sdf-executor-capacity-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime02 Mesh SDF Executor and BVH Capacity

## Scope

This bounded Runtime02 slice keeps Mesh SDF execution behind a caller-owned
`ParallelSliceExecutor` and removes avoidable BVH construction growth. The default cook path is
serial, while importers that own an explicit execution capability can use the ordered executor
path. No process-global Rayon pool is exposed to asset code.

## Implementation

`cook.rs` has no ambient Rayon import and routes the executor overload through
`parallel_map_indices`; serial and explicit-executor cooks retain bit-identical payloads. The BVH
builder now reserves the exact source-triangle upper bound before filtering degenerate triangles,
then reserves the filtered triangle count for `triangle_order`. Triangle filtering, median split
ordering, node bounds, signed distance output, and budget rejection are unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Ambient Rayon imports in Mesh SDF cook | 1 | 0 |
| BVH triangle storage growth | geometric growth while collecting | one upper-bound reservation |
| Triangle-order storage growth | iterator collection growth | one exact reservation |
| Serial/executor output | separate implementations | equal payload and source hash |
| Release p95 | dynamic evidence pending | executor p95 <= 2x serial guard; values from coordinator |

The ignored release test prints `RUNTIME02_MESH_SDF_EXECUTOR_BENCH_V1` with alternating serial
and explicit-executor p95 samples, voxel budget, capability ownership, and the ambient-Rayon
reduction. This is a bounded comparison guard; no power or cross-engine claim is made.

## Validation

- The source contract first failed for the missing BVH reservations, then passed 3/3 after the
  implementation.
- Scoped Rustfmt and `git diff --check` passed for the two Rust files.
- Functional coverage includes serial/executor equality, deterministic self-validation, budget
  rejection, and the no-ambient-Rayon/executor-boundary contract.
- Managed release command:
  `cargo +1.94.1 test -p zircon_runtime --locked --lib --release runtime02_mesh_sdf_ -- --include-ignored --nocapture --test-threads=1`
- Commit integration, terminal p95 values, record finalization, and WeCom delivery remain
  coordinator-owned.

## Remaining Parent-plan Work

The explicit import execution capability, both graphics consumer source migrations, and the
zero-unclassified direct-Rayon static audit are now complete. Runtime02 still requires managed
Cargo behavior evidence and the Windows profiling/power matrix. This record does not close those
parent-plan dynamic gates.
