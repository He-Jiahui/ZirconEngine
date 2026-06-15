---
related_code:
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/change_detection/stats.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
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
  - docs/zircon_runtime/scene/world/project_io.md
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
implementation_files:
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
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

The authoritative top list is still blocked by runtime sampling. The real vampire performance run is waiting on the ZrVM/runtime validation path and on a clean render compile window, so the current list is a guarded scaffold rather than the final sorted M1.3 result.

## Authoritative Top List

Pending authoritative runtime sample. The `vampire_project_session_reports_runtime_fps_and_render_work` path is still the authority for FPS and frame-work evidence, but no current run has produced the required two comparable `vampire_runtime_perf` samples.

Until that sample exists, Runtime 07 M2 may only prepare work against the counted baselines below. It must not claim a top-three ordering, FPS improvement, or final M1.3 completion from static source evidence alone.

## Owner-Budgeted Optimization Gate

Runtime 07 M2 also has an owner-budgeted optimization gate. `performance_hotpath_boundary` now consumes `large_file_ownership_gate` so a measured hotspot cannot be promoted into a large production file without an owner verdict. The current static gate is `migration-debt-present`: threshold 1000 lines, 39 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots.

This means extract, ECS, asset, UI, render, and editor candidates must stay in their owning module families. When large production files remain above the owner budget, a Runtime 07 optimization must first split the affected owner surface or defer to the active owner session instead of adding more behavior to the hotspot file.

## Candidate Evidence Matrix

| Candidate | Current Evidence | Runtime 07 Verdict | Next Gate |
|---|---|---|---|
| Extract full rebuild on unchanged headless captures | `frame_extract_rebuild_skips_unchanged_entities` records two unchanged captures, both with `extract.rebuild_clones = 1`, and stable non-zero `extract.output_bytes`. | Eligible for Runtime 07 M2 extract-incremental design after authoritative runtime sampling confirms the cost matters outside headless scaffolding. | Re-run `extract` filters when the lib-test target is buildable; then compare before/after rebuild and output-byte samples. |
| ECS QueryState cache reuse | `query_state_reuses_archetype_matches_across_unchanged_frames` uses 128 entities and 8 repeated runs; the unchanged path records 8 cache hits, 1 miss, and no additional rebuild. | Not currently an optimization target. The local evidence says the cache is reusing unchanged archetype matches. | Only promote if the vampire runtime sample shows low hit rate, excessive candidate counts, or repeated structural invalidation. |
| Change detection mark scanning | `change_detection_scan_skips_unmarked_archetypes` scans three stale marks twice and records 6 scanned marks with 0 added and 0 changed matches. | Not currently an optimization target. The local evidence is a baseline that proves the diagnostic path, not a measured runtime hotspot. | Promote only with scene-level scan counts showing frame cost or excessive mark volume. |
| Asset worker per-frame cost | `AssetWorkerPoolDiagnostics` exposes in-flight/completed/failed/queue peak plus `asset.worker.budgeted_threads`; `asset_worker_pool_matches_runtime_04_and_11_decisions` prevents the worker pool from returning to the old unbounded/no-dedupe baseline. | Not eligible for Runtime 07 M2 yet. Runtime 04/11 own the worker architecture and current evidence is structural/diagnostic, not per-frame polling cost. | Add per-frame worker polling evidence before creating any Runtime 07 optimization slice. |
| Runtime spans | `runtime_frame_time_update`, `runtime_frame_update`, `runtime_frame_extract`, `runtime_frame_submit`, and the per-`SystemStage` `runtime_frame_schedule_stage.<stage>` stage-level span are present in dynamic-session/runtime-loop/`SceneScheduleRunner` source. | Eligible as measurement infrastructure, not as an optimization target. The schedule-runner span lets traces split the broad update phase into concrete ECS stage work without changing scheduler semantics. | Profiling trace must show update/extract/submit and stage-level percentages before ranking hotspots. |

## Render-Plan Diversions

The old 10fps RenderDoc evidence still records 230 draws, 231 pre-draw `vkCmdCopyBuffer` calls, 31 render passes, and heavy SSR pyramid work. Those are real performance signals, but they are render submission and render graph responsibilities. Runtime 07 M2 is not allowed to fix render submission, mesh draw upload, HZB/SSR graph construction, or GPU occlusion behavior directly.

The current diversion rule is:

- Buffer-copy and draw-command storms go to render plan 02 / mesh draw command pipeline.
- Visibility, HZB, occlusion, and static-index work go to render plan 04.
- Runtime 07 may consume the resulting render diagnostics, but it must not move render ownership back into dynamic session, ECS, asset, or input code.

## Guard

`runtime_07_hotspot_inventory_requires_counted_evidence_before_m2` keeps this document, Runtime 07, the runtime index, the schedule-runner span source, the local counted tests, and the persisted 10fps evidence anchors aligned. The guard intentionally allows the authoritative top list to remain pending, but it rejects returning to an empty hotspot placeholder or starting M2 from undocumented suspicion.

`runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps the owner-budget gate facts synchronized across this document, Runtime 07, the runtime index, the M0 review, the interface-convergence mirror, and `large-file-ownership-m1.md`. It locks the current static evidence to threshold 1000, 39 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots, and it rejects stale 33/37/38/41/42-hotspot or removed Hub `app/` path anchors. This is a mirror guard; it does not mean the remaining large files are split.

`runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner` protects the scene asset split that removed the old scene asset large-file hotspot. It requires the folder-backed `scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs` layout, keeps `SceneMobilityAsset` owned only by `scene/mod.rs`, prevents `scene/physics.rs` from reintroducing that enum, and verifies `SceneSpotLightAsset` still has its public fields and export chain through `lighting.rs`, `scene/mod.rs`, `asset/assets/mod.rs`, and `asset/mod.rs`. It also requires Runtime 07 and scene module docs to retain the split-drift repair state anchors.

`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners` protects the project_io folder split. It requires `project_io.rs` to keep only the `World` project I/O entry orchestration and the child declarations for `project_io/{camera,physics,post_process,references,script,transform}.rs`, and it rejects moving converter helper definitions back into the entry file. This keeps the Runtime 07 project_io folder split tied to the current `large_file_hotspot_count = 39` / `runtime-other = 12` owner-budget state.

`runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner` protects the Dynamic Session Event Split. It requires `session.rs` to keep the private Rust-ABI event entry and `mod events;`, requires `session/events.rs` to own the pointer, mouse, touch, keyboard, IME, file-drag, window, gamepad, accessibility, camera, and menu event helpers, and rejects moving those helpers back into the session entry file. This keeps the dynamic session event split tied to the current `large_file_hotspot_count = 39` / `runtime-other = 12` owner-budget state.

`runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` is the narrower closeout gate for extract/ecs_query/performance profiling/FPS gates. It keeps the Runtime 07 plan and runtime index in `in_progress` while `extract`, `ecs_query`, trace/profiling, and authoritative vampire FPS validation are still pending a clean render/runtime build lane.

The structural mirror for this boundary is `performance_hotpath_boundary`. Its current static evidence is mirrored by `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` and reports `expected_source_file_count = 10`, `expected_test_file_count = 5`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 13`, `change_counter_anchor_count = 9`, `extract_counter_anchor_count = 10`, `asset_worker_anchor_count = 5`, `hotspot_guard_anchor_count = 20`, `test_anchor_count = 12`, `doc_anchor_count = 17`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = migration-debt-present`, `large_file_hotspot_count = 39`, `large_file_migration_debt_count = 5`, `large_file_owner_class_count = 5`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. It also mirrors the `large_file_ownership_gate` owner-budget result described above. This is a structure-sync guard only; it does not replace the pending extract/ecs_query/profiling/FPS validation lane.
