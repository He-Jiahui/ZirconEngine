---
related_code:
  - zircon_runtime/src/graphics/material
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# Renderer/material/shader/streaming protected-plan routing evidence (2026-08-15)

## Coordinator decision

The 136/136-file review is complete in
`2026-08-15-renderer-material-shader-streaming-current-architecture-review.md`. The current freeze is
29,960 physical lines, 284 tests and manifest
`ea44d51aa946bfceabd0197e77b6532a1abb7ae4c0e1d84f6045858ec365860b`.

Performance01 requested authorization for the required owner writes on 2026-08-15. The coordinator
returned `protected_plan_definition` for both numbered performance plan definitions and
`outside_registered_child` for `docs/plans/performance/pending.md`. This Session is restricted to
`docs/plans/performance/01/**`; it did not bypass that ownership boundary or touch foreign-dirty
`review.md`.

This is ordinary routing evidence, not a cross-plan `failure-*` handoff. The work is not blocked from
making useful static progress; only the protected owner merges and dynamic product acceptance remain.

## Required owner merges

### Performance main plan

Add `PERF-MVP-636` as P0 and link it to PERF-MVP-357/358/404/623/633/635:

| ID | Priority | Current root cause | Required hard cut | Acceptance summary |
|---|---|---|---|---|
| PERF-MVP-636 | P0 | Material include, shader assembly/prewarm/disk cache, mesh pipeline compiler, graph compiler and render-resource streamer are separate authorities. Two private mesh compiler threads coexist with a deliberately serial prewarm worker; disk variant identity omits source artifact/compiler/backend versions; graph compile repeatedly materializes descriptors and adds a total pass chain; stable frames poll visible resources and model dependencies before synchronous miss work/GPU creation; cache I/O occurs synchronously and from Drop | Plan02 M1/M3, Runtime04/11, Render01/02/08/13/17, Plugins01 and Editor09 hard-cut one generation chain: `AssetCatalog -> ShaderSourceArtifact -> ShaderPermutation -> PipelineSchema -> RhiPipeline -> PreparedRenderAsset`. Use content-keyed single-flight TaskGraph jobs with bytes/count/age/deadline budgets, one RHI PSO owner, event-driven dirty DAG, immutable prepared scene set and shared IDE artifacts. Delete private compilers, serial/direct cache APIs, per-frame ensure polling, per-assembly full-source rebuild and parallel dead DTOs in the replacing milestones | 136/136 static manifest retained pending. Stable scan/hash/parse/assembly/descriptor rebuild/asset probe/load/GPU create/upload=0; changed work near affected closure; duplicate compile/stale hit/main-render-UI wait/Drop I/O=0; queues/RSS bounded; schema near `O(P+A+E)` and resize does not rebuild topology. Current Cargo plus WPR/xperf/GPU timestamps/RenderDoc/energy and functional reload/device-loss gates pass |

Keep the earlier task meanings but strengthen their dependencies:

- PERF-MVP-357: replace “parallel prewarm” with the shared compiler service; source-table schema is
  retained, but `max_in_flight_variants=1` and direct disk writes are deleted.
- PERF-MVP-358: parsed source/DAG artifacts become the single Render08/Runtime04 generation consumed
  by runtime, material, IDE and preview.
- PERF-MVP-404: resource preparation consumes dirty asset generations and RHI upload tickets; stable
  frame dependency polling is prohibited.
- PERF-MVP-623/633: compute/render pipeline compilation consumes the same schema/permutation/RHI PSO
  generations; no local cache may preserve pass-name or full-source keys.
- PERF-MVP-635: RHI owns physical pipeline creation, device compatibility, persistence capability and
  resource lifetime; shader/pipeline code cannot access WGPU device/queue directly.

### Plan02 M1 and M3

M1 must add keyed single-flight compiler/artifact jobs to the shared TaskGraph. Required fields are
domain/priority/dependencies, job and source-byte budgets, resident/cache/I/O budgets, queue age,
cancellation, affinity and deadline shutdown. Delete per-cache compiler OS threads and unbounded
join. A compiler worker process may be introduced later only as one measured isolation domain owned
by this service.

M3 must add these gates after the RDG/RHI packet definitions:

1. `ShaderArtifactGeneration` and `PipelineSchemaGeneration` use interned dense identities and
   affected-only invalidation; dynamic extent is not part of immutable topology identity.
2. `RhiPipelineGeneration` is the only GPU pipeline owner. Compile/create results publish through
   typed tickets and last-good/fallback/error state; submission never waits.
3. `PreparedRenderAssetGeneration` consumes Runtime04 dirty snapshots and publishes immutable frame
   resource handles. Product per-frame `ensure_*` polling and direct asset/GPU work are deleted.
4. cache keys include source artifact, compiler/backend/device compatibility and schema identity;
   persistence is explicit async work and destructor I/O is zero.
5. material/plugin/IDE/preview consume the same source DAG. Plugin reload invalidates affected ranges
   and stale editor jobs cancel.

### Global pending index

Update the `zircon_runtime` graphics/resource entry to state that material, pipeline, shader,
resource-streamer and backend current Rust source is statically reviewed 136/136 with the evidence
report above. Keep it pending because current-source Cargo, product WPR/xperf, GPU timestamps,
RenderDoc and energy are absent. Do not update `review.md`.

### Other plan owners

- Runtime04: asset catalog generation, normalized shader-token index, dependency/reverse-dependency
  DAG, dirty event ranges and immutable asset snapshots.
- Runtime11: shared compile/artifact task service, keyed single-flight, quotas, cancellation, affinity
  and deadline shutdown; no per-cache private thread.
- Render01: dense pipeline schema, dynamic frame instance, versioned RDG edges and lock-free atomic
  generation publish; remove total pass chain and repeated descriptor materialization.
- Render02/13: prepared mesh/model/material/texture artifacts and bounded upload application; no
  frame-time asset-manager access or direct GPU creation.
- Render08: canonical shader source/permutation identity, parsed module DAG, compiler tickets, one RHI
  PSO handoff and explicit disk artifact service.
- Render17: source/compile/cache/queue/RSS, schema, asset/upload, GPU timestamp and RenderDoc counters;
  WPR/xperf and energy acceptance.
- Plugins01: catalog-generation shader module contribution, stable VM ABI, affected reload/revoke and
  no Rust shader object across dynamic libraries.
- Editor09: consume shared parse/diagnostic/preview artifacts, coalesce generation storms and cancel
  stale jobs; no UI-thread Naga/WGPU/cache I/O.
- Optimize render/runtime indexes: treat local replace/clone/hash/cache tweaks as post-hard-cut work;
  do not optimize private compilers or per-frame ensure loops as permanent architecture.

## Completion condition

This routing record can be retired only after protected owners merge the plan/pending updates and the
hard cut obtains current-source Cargo plus WPR/xperf/GPU timestamp/RenderDoc/energy evidence. Until
then the module is static-complete/dynamic-blocked and no milestone commit or WeCom completion
message is permitted.
