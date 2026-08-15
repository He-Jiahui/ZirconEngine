---
related_code:
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/interaction_extract
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
primary_reference:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneUpdateCommandQueue.h
secondary_references:
  - dev/bevy/crates/bevy_render/src/extract_plugin.rs
  - dev/bevy/crates/bevy_render/src/sync_world.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/Fyrox/editor/src/scene_viewer/mod.rs
measurement_tooling:
  - tools/mvp/Build-RenderExtractProfilingInputs.ps1
  - tools/mvp/New-RenderExtractScaleProject.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - tools/mvp/Write-RenderExtractBaselineReport.ps1
  - Windows Performance Recorder (WPR)
  - Windows Performance Analyzer (WPA)
status: proposed_pending_windows_baseline
created_at: 2026-08-12
---

# Editor Scene Extract Ownership And Measurement Plan

## Scope And Current Facts

This is a design and measurement gate for the F2/F4 scene-to-editor-viewport path. It is not an
accepted optimization record. No current-source profiling executable, product trace, GPU capture,
power trace, or before/after measurement exists at this point.

The current call chain is:

```text
EditorState::render_frame_submission
  -> SceneViewportController::build_render_snapshot
  -> World::build_viewport_render_packet
  -> World::clone + registry/storage projection rebuild
  -> World::build_prepared_viewport_render_packet
  -> full mesh collection + sort + viewport packet
  -> ViewportInteractionExtract::new
  -> Arc<[RenderMeshSnapshot]> copy on interaction-cache miss
  -> renderer submission -> renderer-visible spatial snapshot
```

`Scene` is a type alias for `World`. `World::clone` copies entity and component-facing state, then
rebuilds entity and component projections before render extraction. The editor packet path invokes
that clone for every render submission. A world-generation change additionally copies the complete
`Vec<RenderMeshSnapshot>` into the editor interaction cache. A pointer-cache miss can independently
call `build_render_packet`, repeating the same full extraction before it installs an interaction
extract. These are code facts, not a measured hot-path attribution.

The renderer already publishes a generation-bound `RenderVisibleSpatialQuerySnapshot` after a
successful submission. Editor pointer routing adopts it when its world generation matches, but
pre-submission interaction rebuilding still owns a full render-mesh copy. The existing visible
query is therefore a useful boundary to preserve, not proof that the earlier extract is cheap.

## Reference-Derived Direction

Unreal is the primary architecture reference. `FScene::AddPrimitiveSceneInfo_RenderThread` queues
per-primitive scene work; `TSceneUpdateCommandQueue` coalesces the latest typed payload for a
scene object and retains a persistent scene identity. `FGPUScene::AddPrimitiveToUpdate` marks only
the corresponding persistent primitive dirty. This supports a generation-owned render scene that
receives explicit primitive deltas instead of cloning an authoring world for every viewport packet.

Bevy independently confirms the separation: its `ExtractSchedule` exposes the main world only for
the short extraction phase, keeps a separate render world, synchronizes only opted-in entities, and
copies only data required by rendering. Fyrox keeps editor-scene selection/controller ownership in
the editor scene container while the runtime engine remains the rendering owner. Neither reference
supports putting editor interaction state in the runtime scene or rebuilding a full authoring-world
projection for normal viewport redraws.

The intended Zircon direction is therefore:

```text
runtime World mutation
  -> generation-owned RenderSceneExtractSnapshot (runtime-owned, immutable Arc)
  -> renderer consumes snapshot and emits visible spatial query
  -> editor consumes snapshot metadata/handles and generation-bound spatial query
```

The snapshot must retain `stable_instance_key` as the render-instance identity and `node_id` only
as the authoring owner. This preserves the Render04 rule that multiple primitives of one entity are
never collapsed by visibility, BVH, batching, history, or editor picking.

## Measurement Before Design Selection

Before changing world ownership or packet DTO layout, produce one frozen Windows current-source
baseline with `tools/mvp/Capture-RenderExtractBaseline.ps1`. The script requires both runtime and
editor profiling input pairs from the managed build path and writes only under
`E:\ZirconBuilds\mvp-perf\<session>`; do not substitute a C: output directory or an unbound binary.
Each capture invocation freezes its executable, sibling runtime DLL, and the complete F0 engine
asset root into an invocation-local product directory. The asset root is the formal merge of
`zircon_editor/assets` and `zircon_runtime/assets`: identical duplicate files are idempotent and a
differing duplicate fails before publication. The launched product receives only the relative
`ZIRCON_ASSET_ROOT=assets` layout. Immediately before every process start, the capture tool
recomputes the frozen EXE, DLL, asset-manifest, and full asset-tree hashes; the summary records those
actual prelaunch hashes plus resource file count and bytes. A changed or missing frozen input must
fail before launch rather than falling through to source-tree resources. WPR starts only after this
preflight. Product elapsed time is captured from process launch through exit with a monotonic
stopwatch and stored explicitly as `process_elapsed_ms`; UTC timestamps remain metadata only.
Each launched product is immediately assigned to a Windows Job Object with
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Timeout and output-drain failures terminate and wait for the
whole job even when the root process already exited, so descendants cannot survive into the next
sample. The process and read pipes are acquired while the primary thread remains suspended, before
that already-assigned thread resumes. A successful root exit also terminates and waits for the job
to become empty before root metrics are published or WPR stops.
The runtime product supplies cold first-frame plus pipelined and synchronous steady measurements.
The current editor host supplies only a cold first-presented-frame measurement. Do not label the
editor sample as steady state or infer editor steady-state cost from the runtime product.

Run the same scene and viewport size for each matrix point below. Use three repetitions at minimum
and report median, p95, p99, Windows product-process peak working set and total processor time,
allocations or copied bytes where instrumented, and WPR CPU attribution when its ETL is actually
parsed. Product-process metrics exclude child processes and GPU memory. Record GPU frame time and GPU utilization only when a supported GPU trace/capture is
available; record power only from an OS or device source that explicitly identifies its sampling
method. A missing GPU or power tool is reported as unavailable, never inferred from CPU time.

Create each scale point with `tools/mvp/New-RenderExtractScaleProject.ps1` under
`E:\ZirconBuilds\mvp-perf-projects\<session>`, then supply that created project to the capture
tool. The generated project keeps `res://scenes/main.scene.toml` and `assets/...` resource
references; its manifest records the current source fingerprint and primitive count, but no
absolute resource path becomes part of the project contract. The 100k point is an extract and
payload-scaling workload: clone, projection rebuild, mesh visit, sort, and interaction copying all
occur before renderer visibility culling. It is not evidence that every primitive fits in the
captured viewport. Use the representative 1k and 10k runs for decoded product-image inspection and
report their submitted/cull/visible counts separately.

| Dimension | Values | Required observations |
|---|---|---|
| Scene scale | 1, 1k, 10k, 100k render primitives | world clone time, projection rebuild time, mesh visits, sort comparisons, packet bytes, interaction-copy bytes |
| Change mode | stable, selection-only, camera-only, one transform, topology change | extract/cache hit and miss counts, copied DTO bytes, visible-query generation acceptance |
| Viewport path | submission, pointer move before first submit, pointer move after visible-query publish | full packet builds, pointer fallback builds, query candidate/visited/hit counts |
| Product and submission mode | runtime pipelined first frame, runtime pipelined steady 120 frames, runtime synchronous steady 120 frames, editor first presented frame | frame p50/p95/p99, extract scopes, scheduler and render-framework waits; editor data is cold-first-frame only |
| Graphics evidence | one representative 1k and 10k run | submitted primitives, cull/visible primitives, CPU/GPU overlap, GPU frame time when capture exists |

Instrumentation must add named scopes/counters at the actual ownership boundaries: `world_clone`,
`world_projection_rebuild`, `render_frame_extract`, `render_mesh_visit`, `render_mesh_sort`,
`viewport_packet_mesh_count`, `viewport_packet_mesh_payload_bytes`, `render_frame_mesh_count`,
`render_frame_mesh_payload_bytes`, `interaction_mesh_copy_payload_bytes`,
`interaction_extract_cache_hit`, `interaction_extract_cache_miss`, and
`pointer_fallback_packet_build`. Counters must be generation-scoped, saturating, and reset or
published by the same owner as the frame diagnostics. They must not be kept in a shared artifact
that becomes cross-World state after `World::clone`.

`viewport_packet_mesh_payload_bytes`, `render_frame_mesh_payload_bytes`, and
`interaction_mesh_copy_payload_bytes` are explicit DTO payload proxies: fixed
`RenderMeshSnapshot` storage plus cloned morph-weight elements. They exclude allocator metadata,
vector slack, light/overlay packet fields, and process RSS; report them as payload proxies only.

The scale-project writer itself was measured separately so preparation overhead is not confused
with engine data. A 100k scene is 50,521,244 UTF-8 bytes on the current schema. The original full
StringBuilder plus UTF-8 byte-array path took 20.87 seconds in the local PowerShell process; the
block-streaming writer took 8.04 seconds for byte-identical output (61.5% lower elapsed time) with
about 19.66 MiB working-set growth during the measured write. These are tooling measurements, not
runtime or editor frame-performance claims.

## Decision Gate And Implementation Order

1. If `world_clone + projection rebuild` is material in steady frames, introduce a runtime-owned,
   immutable generation snapshot and make both render submission and editor interaction borrow it.
   Do not expose mutable ECS storage to the editor.
2. If interaction mesh copying is material while world clone is not, change the packet geometry to a
   shared immutable owner and make interaction extract retain an `Arc`, with a test proving an
   unchanged generation adds zero mesh DTO copies. This is not sufficient if it leaves full world
   cloning in the steady submission path.
3. If pointer fallback packet builds are material, route only after a renderer-visible query exists
   or use a runtime-owned spatial extract; preserve the current pre-first-render fallback behavior
   explicitly and test it.
4. If the data shows these paths are not material, do not undertake a snapshot architecture change.
   Attribute the measured hotspot to its real owner instead.

Implementation must be test-first and preserve these contracts:

- two primitives on one entity retain distinct `stable_instance_key` values through scene extract,
  visibility, batching, BVH/history, GPU scene, and picking;
- world, viewport, and rendered-frame generations reject stale snapshots;
- selection, camera, transform, topology, resize, hidden-object, and active-camera changes use
  precise invalidation without stale interaction geometry;
- pre-first-render pointer behavior remains deterministic and post-render pointer routing uses the
  renderer-visible query without a second full packet build;
- worker callback and command-application panic paths restore taken systems, and diagnostics remain
  isolated per `World` clone;
- telemetry describes implemented metrics only. Worker batches, conflicts, and utilization cannot
  be labelled as an overlap metric until such a metric actually exists.

After one selected implementation, repeat the identical matrix, compare absolute and percentage
delta against the frozen baseline, inspect the code a second time, and require product PNG plus
interaction evidence before a performance result is accepted. The comparison must include the
remaining bottleneck, algorithmic scaling, CPU data, and explicitly available GPU/power data.
