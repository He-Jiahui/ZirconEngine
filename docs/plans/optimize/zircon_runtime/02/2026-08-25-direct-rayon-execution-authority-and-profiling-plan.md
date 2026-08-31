---
status: in_progress
implementation_status: source_execution_authority_complete
validation_status: static_boundary_passed_managed_cargo_and_dynamic_profile_pending
related_code:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/asset/mesh_sdf_cook/cook.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/parallel_encoder_set.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-23-task-terminal-delivery-bounded-dispatch.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99o-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/ParallelFor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h
  - dev/bevy/crates/bevy_tasks/src/slice.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
---

# Runtime02 Direct-Rayon Execution Authority And Profiling Plan

## Status And Scope

This is a current-source research and implementation record, not an acceptance receipt. No Cargo,
WPR, GPU, power, or benchmark workload ran for this report. It does not revise the parent-plan
acceptance status or broaden the Runtime11 Rayon whitelist.

The source boundary is now implemented for the three production consumers that were outside the
Runtime11 core-task owners. The goal was not to replace Rayon with a second scheduler. Rayon
remains a private implementation detail of `core/runtime/tasks/{pool,parallel_for}.rs`; consumers
use an injected, budgeted runtime capability rather than a process-global parallel iterator.

## Current Evidence

The 2026-08-25 baseline found five production source paths containing direct Rayon syntax. The
2026-08-27 source result contains exactly the two intentionally classified core owners and zero
unclassified consumers:

| Classification | Paths | Current source facts |
| --- | --- | --- |
| Allowed implementation owners | `tasks/pool.rs`, `tasks/parallel_for.rs` | Construct/install/yield and mutable-slice chunk execution remain under `TaskPool`. |
| Migrated asset consumer | `asset/mesh_sdf_cook/cook.rs` | Explicit-executor cooking calls neutral `ParallelSliceExecutor::parallel_map_indices`; the standalone path stays serial. |
| Migrated graphics consumer | `graphics/.../graph_execution/parallel_encoder_set.rs` | The injected `TaskPool` owns `parallel_map_indices`, preserving bucket index order. |
| Migrated graphics consumer | `graphics/.../mesh_draw_command_list/builder.rs` | The injected `TaskPool` owns `parallel_map_ordered`, preserving the already-sorted plan order and moving each owned plan exactly once. |

`python tools/tests/test_runtime_job_system_audit.py` now passes 3/3 and enforces that exact
two-owner inventory. The migrations did not change mesh SDF traversal, render-graph topology,
mesh batch sorting, cache lookup/store order, or command merge order. This closes a structural
execution-authority leak only; it is not evidence that the parallel algorithms outperform their
serial paths.

## Algorithm Review

### Mesh SDF cook

`TriangleBvh::build` recursively median-sorts triangle centroids and creates leaves of at most
eight triangles. A nearest-distance query orders child traversal by AABB distance and prunes by
the current best distance. Positive-X parity queries prune AABBs that cannot intersect the ray,
but may visit many leaves for overlapping or elongated geometry. Therefore:

- BVH construction is dominated by repeated partition sorting; its practical cost must be
  measured against actual triangle distributions.
- Per voxel, nearest-distance work is geometry dependent; parity work has a data-dependent
  traversal and can approach the triangle count in adverse overlap cases.
- The existing `voxel_count * triangle_count` admission value is a conservative work budget,
  not a measured time complexity, throughput figure, or proof that voxel work is the bottleneck.
- Working memory is at least the triangle/BVH data plus the final `Vec<i16>` voxel payload. Any
  migration must preallocate exactly that final payload and fill disjoint slices; it must not
  create one output vector or task per voxel.

`import_zmesh`, indexed-model import, and model backfill invoke this work through
`AssetImportContext`, which currently carries no execution-domain capability. Replacing it with
`TaskPools::process_default()` would hide compute-pool ownership in the asset importer and permit
nested parallelism under an I/O worker. That is prohibited.

### Render command encoding and mesh command build

`ParallelEncoderSet` already establishes independent immutable buckets and returns command
buffers in compiled graph topology order. `MeshDrawCommandList` first sorts batches and finishes
all mutable cache preparation before the parallel map, then serially merges cache stores and
commands. Both are stable ordered maps over already-owned immutable inputs, not general task
graphs. Their existing `TaskPool` parameter is the correct owner boundary; direct Rayon only
leaks the implementation detail through the consumer.

## Reference Evidence And Decision

Unreal `ParallelFor.h` derives worker width from worker availability and minimum batch size,
keeps a serial path, and exposes an explicit unbalanced mode for variable work. Its comments also
require profiling around each parallel region and reject blocking task bodies. Unreal
`TaskConcurrencyLimiter.h` separates a bounded queue and concurrency slots from task execution.
`Task.h` makes wait/prerequisite ownership explicit.

Bevy `slice.rs` provides injected-pool read-only and mutable chunk maps. The read-only API returns
results in input chunk order, and its tests cover result aggregation and chunk-index semantics.
`task_pool.rs` owns worker lifecycle instead of allowing arbitrary consumers to create ambient
workers. Zircon's existing source-cubemap path already follows the same repository-local pattern:
the framework owns `ParallelSliceExecutor`; runtime `TaskPool` implements it; a serial executor
can prove equal output in tests.

Decision: extend the existing neutral slice contract with one ordered, owned map operation rather
than introducing consumer-local helpers:

```rust
fn parallel_map_ordered<T, R, F>(&self, items: Vec<T>, map: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync;
```

The final owned signature is a deliberate refinement from the borrowed planning sketch: mesh
plans contain resource-bearing payloads, so consuming the `Vec` avoids an extra clone solely to
cross the framework boundary. The contract promises input order, not completion order.
`TaskPool` implements the multi-item path with private Rayon `into_par_iter().map(...).collect()`;
empty and one-item inputs bypass parallel iterator setup. The neutral default is serial and has a
move-only ordered-output test. The method does not take a global pool, expose Rayon types, or
introduce a second task queue.

This is deliberately smaller than the parent `ExecutionRuntime` design. It does not solve
`TaskScope`, admission, worker inventory, shutdown, cancellation, or dynamic-library unloading;
those are mandatory foundation gates from Runtime02/59/99o, not optional follow-up polish.

## Dependency-Ordered Implementation

1. Completed: inspect the current source manifest and the framework/core/graphics/asset owner
   boundaries without changing the consumer algorithms.
2. Source complete: land `ParallelSliceExecutor::parallel_map_ordered` and the private `TaskPool` implementation.
   Keep `pool.rs` and `parallel_for.rs` as the only Rayon owners. Add no process-default fallback,
   allocation cache, or consumer-specific behavior.
3. Source complete: migrate the two graphics consumers. Preserve graph bucket topology order, the serial
   fallback, sorted mesh batch order, and serial cache-store merge. Remove both direct imports
   before changing the Runtime11 inventory; a map adapter that leaves raw Rayon in place is not a
   migration.
4. Source complete: keep asset SDF behind caller-owned `ParallelSliceExecutor::parallel_map_indices`.
   Standalone and deterministic-tool callers remain serial; import work supplies the explicit
   execution capability and the cook does not discover a process pool.
5. Static complete: the audit is tightened to exactly the two core owner paths without adding any
   consumer to `RAYON_CLASSIFICATIONS`. Managed Rust behavior and product profiling remain pending.

## Test And Boundary Matrix

| Layer | Required evidence before the next layer |
| --- | --- |
| Framework contract | Empty, one item, multiple items, ordered output, serial executor parity, and no Rayon type in the public framework contract. |
| Core task owner | Exact-once mapping, no iterator setup on empty/single input, configured pool affinity, worker-count one serial fallback, and no new pool construction. |
| Parallel encoder | Serial/parallel command-buffer count and topology order equality; empty, one bucket, culled bucket, and error-output ordering. |
| Mesh command builder | Serial/parallel command ordering and cache-stat equality; duplicate cache-key fallback; one-worker fallback; no concurrent cache mutation. |
| Mesh SDF | Serial/explicit-executor bit equality, deterministic hash, budget rejection before compute, two-sided and one-sided parity, degenerate geometry, and bounded maximum payload. |
| Audit | The source audit reports only `tasks/pool.rs` and `tasks/parallel_for.rs`; its module inventory/line-budget risks are independently resolved rather than hidden. |

## Profiling And Power Plan

No profiler is started until the coordinator allocates a run identity and a writable artifact
directory below `D:\ZirconBuilds`. Local `C:` output is prohibited. A managed Windows run must
capture the exact source manifest, build profile, CPU topology, GPU adapter/driver, power mode,
input corpus hashes, task-pool configuration, and output artifact hashes.

1. Establish serial and current-authority baselines for SDF using 16/64/128 dimensions and a
   triangle-count corpus; admit 256 only when existing voxel/byte/work budgets permit it. Record
   triangle count after degeneracy filtering, voxel count, `two_sided`, wall time, CPU time,
   peak working set, allocations, and sampled BVH node/leaf work. Do not treat the 100% worst-case
   work budget as measured query cost.
2. Record graphics CPU command encoding with fixed pass/bucket and mesh-batch matrices in serial
   and parallel modes. Collect frame p50/p95/p99, command-buffer order, task queue age, worker
   utilization, context switches, allocator activity, and GPU submission timing separately.
3. Use WPR CPU/context-switch/heap sampling only on the coordinator-managed Windows lane. If WPR
   cannot start, preserve its exact error and collect only the permitted process and diagnostic
   counters. Do not infer stack profiles from wall time.
4. Power comparison requires the same plugged/battery and performance-mode state, a warm-up,
   steady workload interval, hardware telemetry provenance, and repeated runs. Without those
   inputs this plan reports no power claim. Cross-engine comparisons require matching workload,
   hardware, quality settings, and artifacts; no current Zircon result is comparable to Unreal
   or Bevy.

## Exit Criteria And Current Blockers

The explicit runtime-owned joinable executor foundation now exists at source level, so the bounded
consumer migration proceeded while the event-loop acceptance lane remained blocked. Completion
still requires managed Cargo and product validation for the affected graphics and asset paths,
plus the measurement records above. The zero-unclassified direct-Rayon source audit is complete.

Until those dynamic gates complete, this document records source implementation progress. It is
not a performance result, power measurement, acceptance receipt, commit candidate, or
authorization to change the parent-plan acceptance status.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| 结构调研 | 三类 consumer 算法、Unreal/Bevy 执行边界与 profile gate | completed | 2026-08-25 | 本报告 `Algorithm Review` / `Reference Evidence And Decision` |
| 执行 owner 收敛 | Mesh SDF、parallel encoder、mesh command builder 移除直接 Rayon | source_complete | 2026-08-27 | `ParallelSliceExecutor::{parallel_map_indices, parallel_map_ordered}`；JobSystem 静态审计 3/3 |
| 动态验收 | Cargo 行为、Windows WPR/WPA、功耗和同硬件参考引擎对照 | pending | - | 需受管构建/profile lane；当前无性能或功耗结论 |
