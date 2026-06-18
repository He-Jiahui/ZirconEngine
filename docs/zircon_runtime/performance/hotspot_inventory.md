---
related_code:
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system/param_set.rs
  - zircon_runtime/src/scene/ecs/change_detection/stats.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/animation/scene_hook/diagnostics.rs
  - zircon_runtime/src/animation/scene_hook/events.rs
  - zircon_runtime/src/animation/scene_hook/node_pose.rs
  - zircon_runtime/src/animation/scene_hook/pending.rs
  - zircon_runtime/src/animation/scene_hook/scan.rs
  - zircon_runtime/src/animation/scene_hook/tick.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/physics.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - docs/zircon_runtime/scene/world/project_io.md
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
implementation_files:
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system/param_set.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/animation/scene_hook/diagnostics.rs
  - zircon_runtime/src/animation/scene_hook/events.rs
  - zircon_runtime/src/animation/scene_hook/node_pose.rs
  - zircon_runtime/src/animation/scene_hook/pending.rs
  - zircon_runtime/src/animation/scene_hook/scan.rs
  - zircon_runtime/src/animation/scene_hook/tick.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/index.md
  - .codex/sessions/20260611-0416-rendering-10fps-analysis.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/schedule_runner.rs zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs zircon_runtime/src/tests/runtime_absorption/mod.rs
  - source/doc anchor scan for Runtime 07 M0.3 stage span, M1.3 evidence gate, and render diversion: passed 2026-06-13
  - tracked scoped git diff --check plus untracked no-index diff-check for Runtime 07 performance files: passed 2026-06-13 with LF-to-CRLF warnings only on tracked files
  - runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit added 2026-06-14; Cargo pending active compile lanes
  - runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner added 2026-06-14; standalone rustc performance_hotspots.rs 4/4 passed; Cargo pending active compile lanes
doc_type: module-detail
---

# Runtime 07 Hotspot Inventory

## Evidence Gate

No Runtime 07 M2 optimization slice may start from an unmeasured suspicion. A candidate is eligible only when it has a named diagnostic path, a named test or capture source, and an owner verdict that says whether the work belongs to Runtime 07 or to a render/plugin/editor plan.

The authoritative top list is still blocked by runtime sampling. The real vampire performance run now has a local ZR VM library path (`E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug`) and runtime DLL path (`E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug`) identified, and the lib-test support compile blockers found during the 2026-06-17 M0.1 attempt have been repaired. The follow-up command timed out after 904 seconds without test output or a `vampire_runtime_perf` sample, so the current list is a guarded scaffold rather than the final sorted M1.3 result.

## Authoritative Top List

Pending authoritative runtime sample. The `vampire_project_session_reports_runtime_fps_and_render_work` path is still the authority for FPS and frame-work evidence, but no current run has produced the required two comparable `vampire_runtime_perf` samples. The latest M0.1 attempt reached the Runtime lib-test compile after setting `ZR_VM_RUST_BINDING_LIB_DIR`, then repaired the asset UI schema-version import path and `RenderBloomSettings` test initializer drift; the follow-up run timed out after 904 seconds with no test result, and its residual cargo/rustc processes were stopped.

Until that sample exists, Runtime 07 M2 may only prepare work against the counted baselines below. It must not claim a top-three ordering, FPS improvement, or final M1.3 completion from static source evidence alone.

## Owner-Budgeted Optimization Gate

Runtime 07 M2 also has an owner-budgeted optimization gate. `performance_hotpath_boundary` now consumes `large_file_ownership_gate` so a measured hotspot cannot be promoted into a large production file without an owner verdict. The current static gate is `migration-debt-present`: threshold 1000 lines, 36 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots.

This means extract, ECS, asset, UI, render, and editor candidates must stay in their owning module families. When large production files remain above the owner budget, a Runtime 07 optimization must first split the affected owner surface or defer to the active owner session instead of adding more behavior to the hotspot file.

## Candidate Evidence Matrix

| Candidate | Current Evidence | Runtime 07 Verdict | Next Gate |
|---|---|---|---|
| Extract full rebuild on unchanged headless captures | `RuntimeFrameExtractCache` now keys the dynamic-session extract by `change_tick`, `query_cache_revision`, active camera, and viewport size. `frame_extract_rebuild_skips_unchanged_entities` records unchanged headless captures as `extract.rebuild_clones = [1, 0]`, while `frame_extract_rebuilds_after_scene_change` records `[1, 1]` after a camera transform mutation; `extract.output_bytes` remains non-zero and stable on the reuse path. | Runtime 07 M2 extract rebuild cache is statically implemented as `extract_rebuild_cache_static_passed_cargo_deferred`; broader runtime ranking still waits for authoritative FPS/profiling samples. | Re-run `extract` filters and vampire FPS/profiling gates in a clean Cargo window; then compare cached/rebuilt counts against real scene captures. |
| ECS QueryState cache reuse | `query_state_reuses_archetype_matches_across_unchanged_frames` uses 128 entities and 8 repeated runs; the unchanged path records 8 cache hits, 1 miss, and no additional rebuild. | Not currently an optimization target. The local evidence says the cache is reusing unchanged archetype matches. | Only promote if the vampire runtime sample shows low hit rate, excessive candidate counts, or repeated structural invalidation. |
| Change detection mark scanning | `change_detection_scan_skips_unmarked_archetypes` scans three stale marks twice and records 6 scanned marks with 0 added and 0 changed matches. | Not currently an optimization target. The local evidence is a baseline that proves the diagnostic path, not a measured runtime hotspot. | Promote only with scene-level scan counts showing frame cost or excessive mark volume. |
| Asset worker per-frame cost | `AssetWorkerPoolDiagnostics` exposes in-flight/completed/failed/queue peak plus `asset.worker.budgeted_threads`; `AssetWorkerPoolFrameSampler` now converts cumulative completions/failures into per-frame `asset.worker.frame_completed` and `asset.worker.frame_failed` deltas. | Not eligible for Runtime 07 M2 yet. Runtime 04/11 own the worker architecture; the new frame sampler is an evidence entry point, not a measured runtime hotspot. | Re-run worker-pool/runtime profiling gates and compare frame deltas against authoritative scene captures before creating any Runtime 07 optimization slice. |
| Animation scene hook per-frame cost | `AnimationSceneFrameDiagnostics` records `animation.scene.scanned_entities`, sequence/clip/graph/state-machine sample counts, `animation.scene.output_poses`, `animation.scene.applied_transforms`, `animation.scene.published_events`, and `animation.scene.state_transitions` from the scene hook tick path. | Not eligible for Runtime 07 M2 yet. This is an evidence entry point for the previous UI/animation suspicion, not a measured runtime hotspot or optimization. | Capture authoritative vampire/runtime frames and compare animation scene counters against frame-time spans before creating an animation optimization slice. |
| Runtime spans | `runtime_frame_time_update`, `runtime_frame_update`, `runtime_frame_extract`, `runtime_frame_submit`, and the per-`SystemStage` `runtime_frame_schedule_stage.<stage>` stage-level span are present in dynamic-session/runtime-loop/`SceneScheduleRunner` source. | Eligible as measurement infrastructure, not as an optimization target. The schedule-runner span lets traces split the broad update phase into concrete ECS stage work without changing scheduler semantics. | Profiling trace must show update/extract/submit and stage-level percentages before ranking hotspots. |

## Profiling Build Entry Points

Runtime 07 M0.2 now has reproducible profiling build commands instead of an
ad hoc manual Cargo invocation. `tools/zircon_build.py` supports
`--mode profiling`, which maps to Cargo `--profile profiling`, and
`--runtime-features target-client,profiling,profiling-tracy` records the client
runtime feature combination used for timeline captures:

```powershell
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy
```

The fast-check equivalent is:

```powershell
./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride "target-client profiling profiling-tracy"
```

These commands are the accepted M0.2 entry points. The current state is
`profiling_build_tooling_static_passed_cargo_deferred_active_lanes`: the tool
paths, root `[profile.profiling]`, and `zircon_runtime` profiling features are
locked by `runtime_07_hotspot_inventory_requires_counted_evidence_before_m2`,
but the actual profiling build duration and bottleneck segment still need a
clean Cargo/rustc window.

## ECS Frame Diagnostic Aggregation

`EcsFramePerformanceDiagnostics` is the ECS-side frame aggregation owner for Runtime 07 M1.1. It collects multiple `QueryStateCacheStats` and `ChangeDetectionScanStats` samples, merges their counters with saturating arithmetic, keeps the newest cached query revision, and writes the aggregate query/change-detection values to one `DiagnosticStore` frame index.

The production QueryState collection path now flows through `SystemState::run(...)`: each `SystemParam` gets a post-run `record_performance_diagnostics(...)` hook, tuple params and `ParamSet` forward that hook to child params, and `QueryState` reports only the delta since its last reported cache stats through `take_unreported_cache_stats()`. The world stores the current frame aggregate behind `World::ecs_frame_performance_diagnostics()`, and `WorldDriver::tick_level(...)` resets the frame aggregate before running the tick's stages.

Change-detection frame collection now uses the same production path. Cached QueryState reads call `QueryFilter::matches_component_locations_with_stats(...)` for `Added<T>` and `Changed<T>`, `QueryIter` / `QueryManyCachedIter` merge local `ChangeDetectionScanStats` back into the state, and cached count/contains/get helpers record their scan deltas before returning. The cached iterator stats sink is stored as `NonNull<QueryState<D, F>>`: cached slices still carry the real borrow, while read-only, non-cached iterator output does not inherit an extra QueryState borrow. `take_unreported_change_detection_stats()` reports only the new scan activity from the last system run, and `World::record_ecs_change_detection_stats(...)` merges it into the same `EcsFramePerformanceDiagnostics` frame.

This aggregation intentionally stays in `scene/ecs` and `scene/world`. `core::diagnostics` remains a generic store, and extract diagnostics remain owned by `dynamic_api/session/extract_stats.rs` because extract construction is a dynamic-session frame path. The source anchors `ecs_frame_performance_diagnostics_record_query_and_change_counts`, `system_state_records_query_cache_stats_into_world_frame_diagnostics`, and `system_state_records_change_detection_stats_into_world_frame_diagnostics` record the current same-frame readback and automatic QueryState/change-detection collection contracts; package-level Cargo validation is still part of the pending Runtime 07 `ecs_query` gate.

## Runtime Frame Extract Cache

Runtime 07 M2 introduces `RuntimeFrameExtractCache` in `dynamic_api/session/extract_cache.rs`. The cache belongs to the dynamic session because it sits directly on the frame capture/present path and because the render bridge must still receive a `RenderFrameExtract`. The cache key includes world `change_tick`, ECS `query_cache_revision`, the active camera entity, and viewport size; the session invalidates it when resize changes the clamped viewport.

`extract.rebuild_clones` now distinguishes a fresh extract from a cache reuse. `RuntimeFrameExtractCacheStatus::Rebuilt` records `1`, and `RuntimeFrameExtractCacheStatus::Reused` records `0`. `frame_extract_rebuild_skips_unchanged_entities` locks the unchanged headless sequence to `[1, 0]`, while `frame_extract_rebuilds_after_scene_change` mutates the active camera transform and locks the changed sequence to `[1, 1]`. This is still a static/headless implementation slice until the broader `extract` Cargo filter, vampire FPS gate, and profiling trace are rerun.

## Animation Scene Frame Diagnostics

Runtime 07 M1.1 adds `AnimationSceneFrameDiagnostics` under `animation/scene_hook/diagnostics.rs`. The scene hook owns this because it is the only frame path that scans animation-bearing scene entities, samples animation assets, publishes animation events, writes pose outputs, and applies pose transforms back to scene nodes.

The diagnostic paths are count-only rows in `DiagnosticStore`: `animation.scene.scanned_entities`, `animation.scene.sequence_samples`, `animation.scene.clip_pose_samples`, `animation.scene.clip_event_samples`, `animation.scene.graph_pose_samples`, `animation.scene.state_machine_pose_samples`, `animation.scene.output_poses`, `animation.scene.applied_transforms`, `animation.scene.published_events`, and `animation.scene.state_transitions`. Empty animation-manager or disabled-playback frames explicitly record zeroes so dashboards can distinguish no animation work from missing instrumentation. The slice status is `animation_scene_frame_diagnostics_static_passed_cargo_deferred`; it supplies evidence for future ranking, not an M2 optimization.

## Render-Plan Diversions

The old 10fps RenderDoc evidence still records 230 draws, 231 pre-draw `vkCmdCopyBuffer` calls, 31 render passes, and heavy SSR pyramid work. Those are real performance signals, but they are render submission and render graph responsibilities. Runtime 07 M2 is not allowed to fix render submission, mesh draw upload, HZB/SSR graph construction, or GPU occlusion behavior directly.

The current diversion rule is:

- Buffer-copy and draw-command storms go to render plan 02 / mesh draw command pipeline.
- Visibility, HZB, occlusion, and static-index work go to render plan 04.
- Runtime 07 may consume the resulting render diagnostics, but it must not move render ownership back into dynamic session, ECS, asset, or input code.

## Guard

`runtime_07_hotspot_inventory_requires_counted_evidence_before_m2` keeps this document, Runtime 07, the runtime index, the schedule-runner span source, the local counted tests, the profiling build entry points, and the persisted 10fps evidence anchors aligned. The guard intentionally allows the authoritative top list to remain pending, but it rejects returning to an empty hotspot placeholder, losing the M0.2 profiling profile command surface, or starting M2 from undocumented suspicion.

`runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps the owner-budget gate facts synchronized across this document, Runtime 07, the runtime index, the M0 review, the interface-convergence mirror, and `large-file-ownership-m1.md`. It locks the current static evidence to threshold 1000 lines, 36 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots, and it rejects stale 33/37/38/39/40/41/42-hotspot or removed Hub `app/` path anchors. This is a mirror guard; it does not mean the remaining large files are split.

`runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner` protects the scene asset split that removed the old scene asset large-file hotspot. It requires the folder-backed `scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs` layout, keeps `SceneMobilityAsset` owned only by `scene/mod.rs`, prevents `scene/physics.rs` from reintroducing that enum, and verifies `SceneSpotLightAsset` still has its public fields and export chain through `lighting.rs`, `scene/mod.rs`, `asset/assets/mod.rs`, and `asset/mod.rs`. It also requires Runtime 07 and scene module docs to retain the split-drift repair state anchors.

`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners` protects the project_io folder split. It requires `project_io.rs` to keep only the `World` project I/O entry orchestration and the child declarations for `project_io/{camera,physics,post_process,references,script,transform}.rs`, and it rejects moving converter helper definitions back into the entry file. This keeps the Runtime 07 project_io folder split tied to the current `large_file_hotspot_count = 36` / `runtime-other = 12` owner-budget state.

`runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner` protects the Dynamic Session Event Split. It requires `session.rs` to keep the private Rust-ABI event entry and `mod events;`, requires `session/events.rs` to own the pointer, mouse, touch, keyboard, IME, file-drag, window, gamepad, accessibility, camera, and menu event helpers, and rejects moving those helpers back into the session entry file. This keeps the dynamic session event split tied to the current `large_file_hotspot_count = 36` / `runtime-other = 12` owner-budget state.

`runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed` protects the artifact cache payload owner split. It requires `cache_payload.rs` to keep only the bincode cache dispatcher and variant conversion entry while `cache_payload/{json_value,mesh,toml_value}.rs` own JSON canonical values, Mesh attributes/indices/morph targets, and TOML table/value conversion. This keeps artifact-cache wire types folder-backed and tied to the current `large_file_hotspot_count = 36` / `runtime-other = 12` owner-budget state.

`runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed` protects the render product diagnostics owner split. It requires `render_stats_store/product.rs` to keep only product-family dispatch and child declarations while `render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs` own their diagnostic paths. The short status anchor uses `render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs` to keep the most visible camera, mesh-command-cache, and GPUScene product families in status tables. This keeps render product diagnostics tied to the current `large_file_hotspot_count = 36` / `runtime-other = 12` owner-budget state.

`runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed` protects the virtual geometry debug snapshot owner split. It requires `virtual_geometry_debug_snapshot.rs` to stay a structural facade while `virtual_geometry_debug_snapshot/{bvh_visualization,cpu_reference,cull_input,execution,node_and_cluster_cull,snapshot,sources}.rs` own BVH visualization, CPU reference, cull-input packing, execution DTOs, node/cluster cull worklists, the top-level snapshot, and provenance enums. The short status anchor uses `virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs`; this keeps the split tied to `virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred`, `large_file_hotspot_count = 36`, and `runtime-framework-render = 3`.

`runtime_07_navigation_runtime_owner_split_reduces_owner_budget_hotspot_count` records the Runtime 14 navigation fallback runtime split that removed the previous navigation runtime hotspot from the Runtime 07 owner-budget count. The code remains owned by Runtime 14 navigation, while Runtime 07 mirrors the current large-file budget facts.

`runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` is the narrower closeout gate for extract/ecs_query/performance profiling/FPS gates. It keeps the Runtime 07 plan and runtime index in `in_progress` while `extract`, `ecs_query`, trace/profiling, and authoritative vampire FPS validation are still pending a clean render/runtime build lane.

The structural mirror for this boundary is `performance_hotpath_boundary`. Its current static evidence is mirrored by `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` and reports `expected_source_file_count = 45`, `expected_test_file_count = 6`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `hotspot_guard_anchor_count = 29`, `test_anchor_count = 26`, `doc_anchor_count = 33`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = migration-debt-present`, `large_file_hotspot_count = 36`, `large_file_migration_debt_count = 5`, `large_file_owner_class_count = 5`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. It also mirrors the `large_file_ownership_gate` owner-budget result, the QueryState cache owner split under `query_state/cache.rs`, the dynamic-session `extract.cache_hits` / `extract.cache_misses` counters, the asset-worker frame sampler counters `asset.worker.frame_completed` / `asset.worker.frame_failed`, the animation scene counters under `animation.scene.*`, the render product diagnostic split `render_product_diagnostics_owner_split_static_passed_cargo_deferred`, and the virtual geometry debug snapshot owner split `virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred`. This is a structure-sync guard only; it does not replace the pending extract/ecs_query/profiling/FPS validation lane.
