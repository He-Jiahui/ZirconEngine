---
related_code:
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/change_detection/stats.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
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
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/schedule_runner.rs zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs zircon_runtime/src/tests/runtime_absorption/mod.rs
  - source/doc anchor scan for Runtime 07 M0.3 stage span, M1.3 evidence gate, and render diversion: passed 2026-06-13
  - tracked scoped git diff --check plus untracked no-index diff-check for Runtime 07 performance files: passed 2026-06-13 with LF-to-CRLF warnings only on tracked files
  - runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit added 2026-06-14; Cargo pending active compile lanes
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

Runtime 07 M2 also has an owner-budgeted optimization gate. `performance_hotpath_boundary` now consumes `large_file_ownership_gate` so a measured hotspot cannot be promoted into a large production file without an owner verdict. The current static gate is `migration-debt-present`: threshold 1000 lines, 41 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots.

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

`runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps the owner-budget gate facts synchronized across this document, Runtime 07, the runtime index, the M0 review, the interface-convergence mirror, and `large-file-ownership-m1.md`. It locks the current static evidence to threshold 1000, 41 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots, and it rejects stale 33-hotspot or removed Hub `app/` path anchors. This is a mirror guard; it does not mean the large files are split.

`runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` is the narrower closeout gate for extract/ecs_query/performance profiling/FPS gates. It keeps the Runtime 07 plan and runtime index in `in_progress` while `extract`, `ecs_query`, trace/profiling, and authoritative vampire FPS validation are still pending a clean render/runtime build lane.

The structural mirror for this boundary is `performance_hotpath_boundary`. Its current static evidence reports source 10/10, guard/test 5/5, frame span anchors 9/9, QueryState telemetry anchors 13/13, change-detection telemetry anchors 9/9, extract telemetry anchors 10/10, asset-worker candidate telemetry anchors 5/5, hotspot guard anchors 16/16, Runtime 07 counter assertion anchors 12/12, doc anchors 16/16, pending Cargo/profiling/FPS gate anchors 5/5, stale top3 placeholder false, and `risks = []`. It also mirrors the `large_file_ownership_gate` owner-budget result described above. This is a structure-sync guard only; it does not replace the pending extract/ecs_query/profiling/FPS validation lane.
