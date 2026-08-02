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
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
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
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_markdown.py
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
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md
  - docs/plans/zircon_runtime/runtime/07/2026-07-11-runtime07-durable-performance-evidence-and-resource-gate.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - tests/acceptance/runtime-performance-filters-current-result.md
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs
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

## 2026-07-11 Current artifact-reuse boundary

The remaining FPS and trace gates cannot reuse artifacts already present on
the machine. ZrVM has no current `build/` tree, its Rust binding link-directory
environment variable is unset, and managed Cargo target roots contain no
`zr_vm_rust_binding` import-library/runtime-DLL pair. The source tree's old
language-server extension DLL has no import library and is not a current
runtime link product. The Perfetto trace test is compiled only under both
`profiling` and `profiling-chrome`; `--list` inspection of five existing
ordinary Runtime lib-test binaries found no matching test. Current resource
evidence is C/D/E/F 12.94/16.48/12.56/2.49 GiB free with 12 external
Cargo/rustc processes active. Runtime 07 therefore remains in progress without
substituting stale or feature-incompatible artifacts for M0.1/M0.3 evidence.
The coordinator could identify seven old released lanes, but its cleanup apply
operation returned `maintenance_unauthorized`; no manual target deletion was
used, so the low-space gate remains authoritative.

The third consecutive resource check further reduced C/D/E/F free space to
12.71/5.97/13.49/0.08 GiB, with 13 Cargo/rustc processes and four
coordinator-managed active or leased lanes. ZrVM still has no current link
product. Runtime 07 cannot enter M0.1/M0.3 execution without an external-state
change or explicit maintenance authority.

Current executable reconciliation 2026-07-10: the available binary passed all 56 selected `ecs_query` behavior tests; its two naming failures pass 2/2 in current source. `extract` passed 281/311; seven Runtime 07/15-owned source/status guards now pass as performance-hotspots 5/5, structure 2/2, naming 1/1, plus a 50-file production-only snapshot scan with zero offenders. The remaining 23 failures stay assigned to active render/HGI/UI/Text owners, and both full filters remain pending until a fresh binary rerun. Evidence: `tests/acceptance/runtime-performance-filters-current-result.md`.

## Evidence Gate

No Runtime 07 M2 optimization slice may start from an unmeasured suspicion. A candidate is eligible only when it has a named diagnostic path, a named test or capture source, and an owner verdict that says whether the work belongs to Runtime 07 or to a render/plugin/editor plan.

The authoritative top list is still blocked by runtime sampling. The previously
identified local ZR VM library path
(`E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug`) and runtime DLL path
(`E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug`) were both absent in the
2026-07-11 current-state check. The lib-test support compile blockers found
during the 2026-06-17 M0.1 attempt have been repaired, but the follow-up command
timed out after 904 seconds without test output or a `vampire_runtime_perf`
sample, so the current list is a guarded scaffold rather than the final sorted
M1.3 result.

The Dynamic Session event-split guard resolves historical hotspot counts from Runtime 07's numbered output archive and index-migration evidence from its numbered runtime-index archive. Parent plans remain current routing/overview owners and are not required to duplicate concrete counts. The current standalone guard passes 1/1; this routing repair does not promote the still-pending Runtime 07 extract, query, profiling, or FPS behavior gates.

The same owner rule now covers the complete performance-hotspots guard tree.
Six live-session inputs were removed, stale parent/status-mirror assertions were
routed to Runtime 07/15 numbered archives, and
`runtime_07_performance_guards_use_durable_evidence_not_session_notes` rejects
the whole `.codex/sessions/` path family. The current-source standalone suite
passes 28/28; this is structural evidence only and does not close the pending
FPS or trace-execution gates.

## Authoritative Top List

The 2026-07-12 current ZrVM run closed the sampling gate with two exact-command
passes: `30.894424483213513 FPS / 32.368300000000005 ms` and
`33.98320549984198 FPS / 29.426299999999998 ms`, both with 116 mesh draws. The
mean-relative FPS deviation is `9.521868%`, below the required 20%. The current
ZrVM binding came from HEAD `2eb70efa143c44c9acc91e002f9f054f54e9f588`;
the import-library and runtime-DLL SHA-256 values are recorded in Runtime 07's
durable evidence record. A `39.22630044992567 FPS` run remains diagnostic only
because shared Runtime source changed during that command.

The authoritative current counter ordering is intentionally ownership-aware:

1. Render submission is the dominant observed work item at 116 mesh draws per
   accepted capture. Optimization ownership stays with Render 02/18; Runtime 07
   does not add a draw-path workaround.
2. Query reuse is healthy in the fixed 128-entity baseline: 8 repeated runs
   produce 8 hits, 1 initial miss and 1 initial rebuild, so QueryState is not a
   Runtime 07 optimization target.
3. Extract reuse is healthy in the fixed headless baseline: unchanged captures
   record rebuilds `[1, 0]`, hits `[0, 1]`, misses `[1, 0]`, and stable non-zero
   output bytes. Change detection records 6 scanned stale marks with 0 matches;
   neither path is promoted without a future scene-level cost regression.

`EcsFramePerformanceDiagnostics::publish(...)` now makes the query and
change-detection frame aggregate visible in the runtime diagnostic store after
every completed `WorldDriver` tick. This closes the previous gap where the
counts existed only in the World-local aggregate and therefore could not be
read by the Vampire diagnostic path.

`counter_hotspots.json` is now part of the profiling export evidence path. It ranks finite positive `ProfileCounterSnapshot` values through `CounterHotspotReport` / `CounterHotspotEntry`, but it is still an input to the evidence gate rather than the authoritative top list. A counter-only row can identify where to inspect adjacent spans or owner diagnostics; it cannot by itself promote an M2 optimization or replace the pending vampire FPS/profile sample.

## Owner-Budgeted Optimization Gate

Runtime 07 M2 also has an owner-budgeted optimization gate. `performance_hotpath_boundary` now consumes `large_file_ownership_gate` so a measured hotspot cannot be promoted into a large production file without an owner verdict. The current static gate is `classified-and-clear`: threshold 1000 lines, 0 hotspots, 0 owner debt groups, 0 owner classes, and 0 unclassified hotspots.

Current mirror anchors: `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, and `risks = []`. The exact summary is threshold 1000 lines, 0 hotspots, 0 owner debt groups, 0 owner classes, and 0 unclassified hotspots.

This means extract, ECS, asset, UI, render, and editor candidates must stay in their owning module families. If a large production file reappears above the owner budget, a Runtime 07 optimization must first split the affected owner surface or defer to the active owner session instead of adding more behavior to the hotspot file.

## Candidate Evidence Matrix

| Candidate | Current Evidence | Runtime 07 Verdict | Next Gate |
|---|---|---|---|
| Extract full rebuild on unchanged headless captures | `RuntimeFrameExtractCache` now keys the dynamic-session extract by `change_tick`, `query_cache_revision`, active camera, and viewport size. `frame_extract_rebuild_skips_unchanged_entities` records unchanged headless captures as `extract.rebuild_clones = [1, 0]`, while `frame_extract_rebuilds_after_scene_change` records `[1, 1]` after a camera transform mutation; `extract.output_bytes` remains non-zero and stable on the reuse path. | Accepted. The current cache removes the unchanged-frame rebuild and the exact Vampire FPS gate is closed. | Keep the fixed regression thresholds; reopen only if a scene-level capture regresses. |
| ECS QueryState cache reuse | `query_state_reuses_archetype_matches_across_unchanged_frames` uses 128 entities and 8 repeated runs; the unchanged path records 8 cache hits, 1 miss, and no additional rebuild. | Not currently an optimization target. The local evidence says the cache is reusing unchanged archetype matches. | Only promote if the vampire runtime sample shows low hit rate, excessive candidate counts, or repeated structural invalidation. |
| Change detection mark scanning | `change_detection_scan_skips_unmarked_archetypes` scans three stale marks twice and records 6 scanned marks with 0 added and 0 changed matches. | Not currently an optimization target. The local evidence is a baseline that proves the diagnostic path, not a measured runtime hotspot. | Promote only with scene-level scan counts showing frame cost or excessive mark volume. |
| Asset worker per-frame cost | `AssetWorkerPoolDiagnostics` exposes in-flight/completed/failed/queue peak plus `asset.worker.budgeted_threads`; `AssetWorkerPoolFrameSampler` now converts cumulative completions/failures into per-frame `asset.worker.frame_completed` and `asset.worker.frame_failed` deltas. | Not eligible for Runtime 07 M2 yet. Runtime 04/11 own the worker architecture; the new frame sampler is an evidence entry point, not a measured runtime hotspot. | Re-run worker-pool/runtime profiling gates and compare frame deltas against authoritative scene captures before creating any Runtime 07 optimization slice. |
| Animation evaluation per-frame cost | The current Animation Plugin pipeline owns scan/evaluation, events, pose writeback, requests, and tick orchestration, but the former `AnimationSceneFrameDiagnostics` producer and its `animation.scene.*` counters were removed with Runtime `scene_hook`. | Not eligible for Runtime 07 M2. The missing counter producer is an open Plugins04 behavior gap, not measured hotspot evidence. | Restore the established count and explicit-zero semantics in the current Plugin pipeline, then capture authoritative vampire/runtime frames before creating an optimization slice. |
| Generic profiling counters | `CounterHotspotReport` and `counter_hotspots.json` aggregate finite positive counters such as `extract.*`, `ecs.*`, `asset.worker.*`, `time.*`, `schedule.*`, and `tasks.*` into total/avg/p95/max/latest/count/frame_count rows, and `ProfileControlResponse.counter_hotspot_report` returns the same report from `export_report`. Historical `animation.scene.*` rows will remain absent until the Plugins04 diagnostics gap is implemented. | Eligible as evidence routing only. This bridges existing counter rows into profile artifacts without declaring a top list or changing counter ownership. | Use the ranked counter rows to choose which owner diagnostics and frame spans to inspect; only promote an M2 slice after authoritative FPS/profile evidence confirms the cost. |
| Runtime spans | `runtime_frame_time_update`, `runtime_frame_update`, `runtime_frame_extract`, `runtime_frame_submit`, and the per-`SystemStage` `runtime_frame_schedule_stage.<stage>` stage-level span are present in dynamic-session/runtime-loop/`SceneScheduleRunner` source. Direct runtime-frame submit also exposes render-framework `build_submission_context`, `prepare_runtime_submission`, `render_frame_with_pipeline`, and `collect_runtime_feedback` spans under `submit_runtime_frame`. | Eligible as measurement infrastructure, not as an optimization target. The schedule-runner span lets traces split the broad update phase into concrete ECS stage work without changing scheduler semantics, while the direct submit spans keep generated submit/present and `ViewportRenderFrame` submit comparable in profiling artifacts. | Profiling trace must show update/extract/submit, direct render-framework subspans, and stage-level percentages before ranking hotspots. |

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

Runtime 07 F3 now has direct runtime-frame trace artifact coverage under
`render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending`.
`direct_runtime_frame_submit_exports_perfetto_trace_artifacts` submits a
headless direct `ViewportRenderFrame`, stops capture, exports the profiling
report, and requires `timeline.zrtrace.json`, `timeline.perfetto.json`,
`hotspots.json`, and `summary.md` to be written. Both native and Perfetto traces
must retain `submit_runtime_frame`, `render_frame_with_pipeline`,
`DepthPrepass`, and `depth-prepass`. Static guards passed on 2026-06-22, while
cargo profiling validation still needs a clean build lane after the current
Runtime 06/plugin private-field compile errors are resolved. This is
trace/export source and static evidence for the direct submit path only; it
does not replace authoritative vampire FPS samples or the clean
profiling-tracy build-duration gate.

Runtime 07 F3 now also records
`render_generated_camera_loop_shared_extract_static_passed_cargo_locked_blocked`
for the ordinary generated submit/present camera loop. `submit_camera_loop(...)`
streams a single `Arc<RenderFrameExtract>` source through
`stream_camera_loop_extract_submissions(...)`; `CameraLoopExtractSourceState`
restores view, post-process, VG, and HGI source fields before each child, then
`Arc::make_mut(&mut source_extract)` selects the child camera in place. The
selected submit/present bodies share
`build_frame_submission_context_from_runtime_frame_extract(...)`, and the old
owned `build_frame_submission_context(...)` / `FrameSubmissionExtractSource`
path is removed. Static guards passed, but final Cargo validation is blocked by
current `Cargo.lock` drift, so this is a clone-removal status slice rather than
an FPS/profile closeout.

## ECS Frame Diagnostic Aggregation

`EcsFramePerformanceDiagnostics` is the ECS-side frame aggregation owner for Runtime 07 M1.1. It collects multiple `QueryStateCacheStats` and `ChangeDetectionScanStats` samples, merges their counters with saturating arithmetic, keeps the newest cached query revision, and writes the aggregate query/change-detection values to one `DiagnosticStore` frame index.

The production QueryState collection path now flows through `SystemState::run(...)`: each `SystemParam` gets a post-run `record_performance_diagnostics(...)` hook, tuple params and `ParamSet` forward that hook to child params, and `QueryState` reports only the delta since its last reported cache stats through `take_unreported_cache_stats()`. The world stores the current frame aggregate behind `World::ecs_frame_performance_diagnostics()`, and `WorldDriver::tick_level(...)` resets the frame aggregate before running the tick's stages.

Change-detection frame collection now uses the same production path. Cached QueryState reads call `QueryFilter::matches_component_locations_with_stats(...)` for `Added<T>` and `Changed<T>`, `QueryIter` / `QueryManyCachedIter` merge local `ChangeDetectionScanStats` back into the state, and cached count/contains/get helpers record their scan deltas before returning. The cached iterator stats sink is stored as `NonNull<QueryState<D, F>>`: cached slices still carry the real borrow, while read-only, non-cached iterator output does not inherit an extra QueryState borrow. `take_unreported_change_detection_stats()` reports only the new scan activity from the last system run, and `World::record_ecs_change_detection_stats(...)` merges it into the same `EcsFramePerformanceDiagnostics` frame.

This aggregation intentionally stays in `scene/ecs` and `scene/world`. `core::diagnostics` remains a generic store, and extract diagnostics remain owned by `dynamic_api/session/extract_stats.rs` because extract construction is a dynamic-session frame path. The source anchors `ecs_frame_performance_diagnostics_record_query_and_change_counts`, `system_state_records_query_cache_stats_into_world_frame_diagnostics`, and `system_state_records_change_detection_stats_into_world_frame_diagnostics` record the current same-frame readback and automatic QueryState/change-detection collection contracts; package-level Cargo validation is still part of the pending Runtime 07 `ecs_query` gate.

## Runtime Frame Extract Cache

Runtime 07 M2 introduces `RuntimeFrameExtractCache` in `dynamic_api/session/extract_cache.rs`. The cache belongs to the dynamic session because it sits directly on the frame capture/present path and because the render bridge must still receive a `RenderFrameExtract`. The cache key includes world `change_tick`, ECS `query_cache_revision`, the active camera entity, and viewport size; the session invalidates it when resize changes the clamped viewport.

`extract.rebuild_clones` now distinguishes a fresh extract from a cache reuse. `RuntimeFrameExtractCacheStatus::Rebuilt` records `1`, and `RuntimeFrameExtractCacheStatus::Reused` records `0`. `frame_extract_rebuild_skips_unchanged_entities` locks the unchanged headless sequence to `[1, 0]`, while `frame_extract_rebuilds_after_scene_change` mutates the active camera transform and locks the changed sequence to `[1, 1]`. This is still a static/headless implementation slice until the broader `extract` Cargo filter, vampire FPS gate, and profiling trace are rerun.

## Animation Scene Frame Diagnostics Gap

Runtime 07 M1.1 historically added `AnimationSceneFrameDiagnostics` under `animation/scene_hook/diagnostics.rs`. The 2026-08-01 hard cut deleted the entire Runtime scene hook while moving evaluation to `zircon_plugins/animation/runtime/src/evaluation/pipeline/`; no current production type or producer preserves those diagnostics.

The former count-only paths were `animation.scene.scanned_entities`, `animation.scene.sequence_samples`, `animation.scene.clip_pose_samples`, `animation.scene.clip_event_samples`, `animation.scene.graph_pose_samples`, `animation.scene.state_machine_pose_samples`, `animation.scene.output_poses`, `animation.scene.applied_transforms`, `animation.scene.published_events`, and `animation.scene.state_transitions`. Their explicit-zero semantics remain an acceptance requirement, but the historical token `animation_scene_frame_diagnostics_static_passed_cargo_deferred` is not current implementation evidence. `docs/plans/zircon_plugins/04/failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md` owns the open behavior-preserving migration; this inventory does not redirect the missing diagnostics to a nearby file or restore the retired hook.

Later `animation_scene_anchor_count = 19` mirror receipts in this document count static audit anchors from their dated snapshots. They do not prove that `AnimationSceneFrameDiagnostics` or the `animation.scene.*` producers exist in the current production tree.

## Render-Plan Diversions

The old 10fps RenderDoc evidence still records 230 draws, 231 pre-draw `vkCmdCopyBuffer` calls, 31 render passes, and heavy SSR pyramid work. Those are real performance signals, but they are render submission and render graph responsibilities. Runtime 07 M2 is not allowed to fix render submission, mesh draw upload, HZB/SSR graph construction, or GPU occlusion behavior directly.

The current diversion rule is:

- Buffer-copy and draw-command storms go to render plan 02 / mesh draw command pipeline.
- Visibility, HZB, occlusion, and static-index work go to render plan 04.
- Runtime 07 may consume the resulting render diagnostics, but it must not move render ownership back into dynamic session, ECS, asset, or input code.

## Guard

`runtime_07_hotspot_inventory_requires_counted_evidence_before_m2` keeps this document, Runtime 07, the runtime index, the schedule-runner span source, the local counted tests, the profiling build entry points, and the persisted 10fps evidence anchors aligned. The guard intentionally allows the authoritative top list to remain pending, but it rejects returning to an empty hotspot placeholder, losing the M0.2 profiling profile command surface, or starting M2 from undocumented suspicion.

`runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps the owner-budget gate facts synchronized across this document, Runtime 07, the runtime index, the M0 review, the interface-convergence mirror, and `large-file-ownership-m1.md`. It locks the current static evidence to threshold 1000 lines, 0 hotspots, 0 owner debt groups, 0 owner classes, and 0 unclassified hotspots, and it rejects stale 30/33/36/37/38/39/40/41/42-hotspot or removed Hub `app/` path anchors. This is a mirror guard; it does not mean the extract/ecs_query/profiling/FPS validation lane is complete.

`runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner` protects the scene asset split that removed the old scene asset large-file hotspot. It requires the folder-backed `scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs` layout, keeps `SceneMobilityAsset` owned only by `scene/mod.rs`, prevents `scene/physics.rs` from reintroducing that enum, and verifies `SceneSpotLightAsset` still has its public fields and export chain through `lighting.rs`, `scene/mod.rs`, `asset/assets/mod.rs`, and `asset/mod.rs`. It also requires Runtime 07 and scene module docs to retain the split-drift repair state anchors.

`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners` protects the project_io folder split. It requires `project_io.rs` to keep only the `World` project I/O entry orchestration and the child declarations for `project_io/{camera,physics,post_process,references,script,transform}.rs`, and it rejects moving converter helper definitions back into the entry file. This keeps the Runtime 07 project_io folder split tied to the current `large_file_hotspot_count = 0` / empty owner-class state.

`runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner` protects the Dynamic Session Event Split. It requires `session.rs` to keep the private Rust-ABI event entry and `mod events;`, requires `session/events.rs` to own the pointer, mouse, touch, keyboard, IME, file-drag, window, gamepad, accessibility, camera, and menu event helpers, and rejects moving those helpers back into the session entry file. This keeps the dynamic session event split tied to the current `large_file_hotspot_count = 0` / empty owner-class state.

`runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed` protects the artifact cache payload owner split. It requires `cache_payload.rs` to keep only the bincode cache dispatcher and variant conversion entry while `cache_payload/{json_value,mesh,toml_value}.rs` own JSON canonical values, Mesh attributes/indices/morph targets, and TOML table/value conversion. This keeps artifact-cache wire types folder-backed and tied to the current `large_file_hotspot_count = 0` / empty owner-class state.

`runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed` protects the render product diagnostics owner split. It requires `render_stats_store/product.rs` to keep only product-family dispatch and child declarations while `render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs` own their diagnostic paths. The short status anchor uses `render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs` to keep the most visible camera, mesh-command-cache, and GPUScene product families in status tables. This keeps render product diagnostics tied to the current `large_file_hotspot_count = 0` / empty owner-class state.

`runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed` protects the virtual geometry debug snapshot owner split. It requires `virtual_geometry_debug_snapshot.rs` to stay a structural facade while `virtual_geometry_debug_snapshot/{bvh_visualization,cpu_reference,cull_input,execution,node_and_cluster_cull,snapshot,sources}.rs` own BVH visualization, CPU reference, cull-input packing, execution DTOs, node/cluster cull worklists, the top-level snapshot, and provenance enums. The short status anchor uses `virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs`; this keeps the split tied to `virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred`, `large_file_hotspot_count = 0`, and the empty owner-class state.

`runtime_07_navigation_runtime_owner_split_reduces_owner_budget_hotspot_count` records the Runtime 14 navigation fallback runtime split that removed the previous navigation runtime hotspot from the Runtime 07 owner-budget count. The code remains owned by Runtime 14 navigation, while Runtime 07 mirrors the current large-file budget facts.

`runtime_07_profile_counter_hotspot_export_keeps_generic_counter_evidence_visible` protects the generic profiling counter evidence path. It requires `CounterHotspotReport`, `CounterHotspotEntry`, `PROFILE_COUNTER_HOTSPOTS_FILE`, `counter_hotspots.json`, `analyze_counter_hotspots`, and `ProfileControlResponse.counter_hotspot_report` to stay wired through the interface DTOs, runtime profiling export, Runtime 07 status rows, and this evidence gate. This guard keeps counter export visible without allowing a counter-only result to become an M2 optimization.

`runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` is the narrower closeout gate for extract/ecs_query/performance profiling/FPS gates. It keeps the Runtime 07 plan and runtime index in `in_progress` while `extract`, `ecs_query`, profiling-tracy build duration, and authoritative vampire FPS validation are still pending a clean render/runtime build lane. Direct runtime-frame native/Perfetto trace export is covered by `direct_runtime_frame_submit_exports_perfetto_trace_artifacts`, but it is not enough to close the full Runtime 07 performance gate.

Current execution refresh 2026-07-11: the exact `profiling,profiling-chrome`
direct runtime-frame trace test completed its current-source optimized build in
67m59s and passed 1/1 in 12.30 seconds. It generated and validated both timeline
formats, hotspots, and summary in a temporary tree, including the required
`submit_runtime_frame`, `render_frame_with_pipeline`, `DepthPrepass`, and
`depth-prepass` anchors, then cleaned that tree. The trace execution gate is
accepted; this does not replace the still-open two-run Vampire FPS baseline.

Current binding provenance refresh 2026-07-11: a broader recursive search found
`zr_vm_rust_binding.lib` and `zr_vm_rust_binding.dll` under ZrVM's isolated
`.codex/tmp/aot-clean-verify-20260622-121531` tree. Both were generated on
2026-06-11, and the copied CMake cache points at the absent
`E:/Git/zr_vm/build-msvc` directories; the current clean ZrVM checkout is the
2026-07-09 commit `2eb70efa143c44c9acc91e002f9f054f54e9f588`. The pair is therefore
stale, non-reproducible current-source evidence and is not used for M0.1. The
resumed check saw D below the 50 GiB Cargo threshold and other sessions' active
or leased compile lanes, so it did not start a competing full build or alter
external processes. The authoritative two-run FPS/deviation gate remains open.

The structural mirror for this boundary is `performance_hotpath_boundary`. Source/test lists, including `performance_hotspots/{submit_context,hotspot_inventory,scene_project_splits,artifact_render_diagnostics_splits}.rs`, and expected counts are split into `performance_hotpath_source_inventory.py`, domain and status anchors are split into `performance_hotpath_anchor_inventory.py`, Markdown rendering is split into `performance_hotpath_markdown.py`, and `performance_hotpath_boundary.py` remains the audit reader, missing-anchor calculator, large-file gate consumer, and risk aggregator. Current line ownership is source inventory 70 lines, anchor inventory 244 lines, boundary 353 lines, and Markdown renderer 139 lines. Its current static evidence is mirrored by `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` and reports `expected_source_file_count = 46`, `expected_test_file_count = 14`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. It also mirrors the `large_file_ownership_gate` owner-budget result, the QueryState cache owner split under `query_state/cache.rs`, the dynamic-session `extract.cache_hits` / `extract.cache_misses` counters, the asset-worker frame sampler counters `asset.worker.frame_completed` / `asset.worker.frame_failed`, the animation scene counters under `animation.scene.*`, the profiling counter hotspot export `counter_hotspots.json`, the render product diagnostic split `render_product_diagnostics_owner_split_static_passed_cargo_deferred`, and the virtual geometry debug snapshot owner split `virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred`. This is a structure-sync guard only; it does not replace the pending extract/ecs_query/profiling/FPS validation lane.

Current mirror refresh 2026-07-01: `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` now lives in `performance_hotspots/owner_budget/mirror_docs.rs`, and the Runtime 07 audit input includes `performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 14`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-05: `Runtime 15 M3 Runtime 07 submit-context guard child-owner split` / `runtime_15_runtime_07_submit_context_guard_child_owner_split_static_passed_cargo_deferred` keeps the submit-context guard as a route owner and adds `performance_hotspots/submit_context/{sources,source_extract_payloads,camera_loop_sharing,feedback_sidebands,status_docs,split_layout}.rs` to Runtime 07 audit input. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 20`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-05: `Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split` / `runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split_static_passed_cargo_deferred` keeps the hotspot-inventory guard as a route owner and adds `performance_hotspots/hotspot_inventory/{sources,evidence_gate_docs,ecs_extract_counters,profiling_trace_render,split_layout}.rs` to Runtime 07 audit input. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 25`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split` / `runtime_15_runtime_07_owner_budget_guard_folder_backed_static_passed_cargo_deferred` keeps the owner-budget guard as a route/test-entry owner and adds `performance_hotspots/owner_budget/{sources,parent_routes,child_routes,source_inventory,line_budgets,status_docs,split_layout}.rs` to Runtime 07 audit input alongside `performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs`; the explicit parent-route child is `performance_hotspots/owner_budget/parent_routes.rs`. The split guard is `runtime_15_runtime_07_owner_budget_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 32`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split` / `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred` keeps the artifact/render diagnostics guard as a route owner and adds `performance_hotspots/artifact_render_diagnostics_splits/{artifact_cache_payload,render_product_diagnostics,split_layout}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/artifact_render_diagnostics_splits/artifact_cache_payload.rs`. The split guard is `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 35`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 scene/project guard child-owner split` / `runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred` keeps the scene/project guard as a route owner and adds `performance_hotspots/scene_project_splits/{scene_asset,project_io,dynamic_session_event,split_layout}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/scene_project_splits/scene_asset.rs`. The split guard is `runtime_15_runtime_07_scene_project_guard_child_owner_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 39`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters child-owner split` / `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split_static_passed_cargo_deferred` keeps `performance_hotspots/hotspot_inventory/ecs_extract_counters.rs` as a route owner and adds `performance_hotspots/hotspot_inventory/ecs_extract_counters/{query_change,extract_cache,asset_animation,frame_diagnostics,split_layout}.rs` to Runtime 07 audit input; the explicit child anchor is `hotspot_inventory/ecs_extract_counters/query_change.rs`. The split guard is `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 44`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split` / `runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/mirror_docs.rs` as a route owner and adds `performance_hotspots/owner_budget/mirror_docs/{sources,performance_guard,source_inventory,audit_wiring,doc_mirrors,split_layout}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/owner_budget/mirror_docs/sources.rs`. The split guard is `runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 50`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split` / `runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/hotspot_inventory/split_layout.rs` as a route owner and adds `performance_hotspots/hotspot_inventory/split_layout/{sources,route,source_inventory,status_docs}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/hotspot_inventory/split_layout/sources.rs`. The split guard is `runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 54`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split` / `runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/split_layout.rs` as a route owner and adds `performance_hotspots/owner_budget/split_layout/{route,source_inventory,status_docs}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/owner_budget/split_layout/route.rs`. The split guard is `runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 57`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 submit-context split-layout guard folder-backed split` / `runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/submit_context/split_layout.rs` as a route owner and adds `performance_hotspots/submit_context/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/submit_context/split_layout/source_inventory.rs`. The split guard is `runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_submit_context_guard_child_owner_split` remains as the historical wrapper. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split` / `runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/scene_project_splits/split_layout.rs` as a route owner and adds `performance_hotspots/scene_project_splits/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/scene_project_splits/split_layout/source_inventory.rs`. The split guard is `runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_scene_project_guard_child_owner_split` remains as the historical wrapper. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 65`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split` / `runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/artifact_render_diagnostics_splits/split_layout.rs` as a route owner and adds `performance_hotspots/artifact_render_diagnostics_splits/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/artifact_render_diagnostics_splits/split_layout/source_inventory.rs`. The split guard is `runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split` remains as the historical wrapper. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 69`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters split-layout guard folder-backed split` / `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout.rs` as a route owner and adds `performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input; the explicit child anchor is `performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/source_inventory.rs`. The split guard is `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split` remains as the historical wrapper. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 73`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current mirror refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split` / `runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/mirror_docs/sources.rs` as a route/type owner and adds `performance_hotspots/owner_budget/mirror_docs/sources/{assertions,load,views}.rs` to Runtime 07 audit input. The split guard is `runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 76`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current owner-budget source refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split` / `runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/sources.rs` as a route/type owner and adds `performance_hotspots/owner_budget/sources/load.rs` to Runtime 07 audit input. The split guard is `runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 77`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current owner-budget child-routes refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split` / `runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/child_routes.rs` as a route owner and adds `performance_hotspots/owner_budget/child_routes/{submit_context,hotspot_inventory,scene_project,artifact_render_diagnostics,owner_budget}.rs` to Runtime 07 audit input. The split guard is `runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 82`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Explicit child path anchor: `tests/runtime_absorption/performance_hotspots/owner_budget/child_routes/submit_context.rs`.

Current owner-budget line-budgets refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split` / `runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/line_budgets.rs` as a route owner and adds `performance_hotspots/owner_budget/line_budgets/{root,artifact_render_diagnostics,hotspot_inventory,owner_budget,scene_project,submit_context}.rs` to Runtime 07 audit input. The explicit test child path anchor is `tests/runtime_absorption/performance_hotspots/owner_budget/line_budgets/{root,artifact_render_diagnostics,hotspot_inventory,owner_budget,scene_project,submit_context}.rs`. The split guard is `runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 88`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.

Current owner-budget split-layout route refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split` / `runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred` keeps `performance_hotspots/owner_budget/split_layout/route.rs` as a route owner and adds `performance_hotspots/owner_budget/split_layout/route/{parent_route,split_route,support_routes}.rs` to Runtime 07 audit input. The explicit test child path anchor is `tests/runtime_absorption/performance_hotspots/owner_budget/split_layout/route/{parent_route,split_route,support_routes}.rs`. The split guard is `runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split`. `performance_hotpath_boundary` currently reports `expected_source_file_count = 46`, `expected_test_file_count = 91`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. This only updates the static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo validation remains pending.
