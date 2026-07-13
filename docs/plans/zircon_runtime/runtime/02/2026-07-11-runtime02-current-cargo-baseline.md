---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_app/src/entry
  - tests/acceptance/runtime-02-core-spine-root-surface.md
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - cargo test -p zircon_runtime --lib generated --locked
  - cargo test -p zircon_runtime --lib core:: --locked
  - cargo test -p zircon_app --locked
status: runtime_02_managed_core_647_passed_10_external_render_ui_failures_absorption_priority_guard_drift_open
---

# Runtime 02 current Cargo baseline

Date: 2026-07-11

Status: `runtime_02_managed_core_647_passed_10_external_render_ui_failures_absorption_priority_guard_drift_open`

## Scope

This record starts Runtime 02's current-source milestone validation after Runtime 01 completion. It uses the fresh default-feature Runtime lib-test binary produced during Runtime 01 acceptance.

## Results

- `generated`: 27 passed / 0 failed / 7456 filtered in 102.23s. The generated-code marker, adapter-only, delegation, folder-backed scope, mirror-doc, export archive/platform, shader/RHI, scene/script, and compiled-template tests are green.
- `core::`: 643 passed / 12 failed / 6828 filtered in 49.43s.
- Runtime 02's core spine/root/generated structure owners did not fail.

## Coordinator-managed current-source rerun

- Managed lane: `D:\targets\zircon-engine\lanes\test-204487b704354632928fdd363cd804eb`.
- Fresh default-feature lib-test build: completed in 14m22s with 537 existing warnings.
- `core::`: 647 passed / 10 failed / 6835 filtered in 31.71s.
- The two earlier volume-registry failures are no longer present. The remaining
  failures are five active Render cases and five active Runtime UI
  text/layout/extract cases; no Runtime 02 spine/root/generated owner failed.
- `runtime_absorption`: 1555 passed / 76 failed / 5861 filtered in 332.85s.
  Most failures are current source/archive-routing assertions in the priority
  review and structure guard families while another Runtime 15 owner is
  actively converging those routes. This result is recorded as an open gate,
  not as a Runtime 02 production-behavior regression.
- Priority structure convention managed snapshot: 1303 passed / 0 failed /
  6189 filtered in 239.49s.
- After the priority review-route repair and concurrent UI/Text edits, the same
  structure filter reports 1299 passed / 4 failed. The four failures are
  active UI/Text owners (`render.rs` 818 lines, `render/tests.rs` 823 lines,
  and two layout-engine owner/current-anchor assertions); no modified review
  guard file failed a budget.
- Current-source standalone `code_review_findings`: 80 passed / 0 failed in
  7.21s after hard-cutting concrete evidence reads to Runtime 15 numbered
  output records.
- Runtime plan-status suite: 48 passed / 0 failed after the maintenance-authorized
  Runtime 02 `last_refined` metadata sync to 2026-07-11.

## Failure ownership

The current 10 broad `core::` failures are outside the Runtime 02 owner and already lie in active sessions:

- Render/post-process/IBL/pipeline: 5 failures (`effect_stack_settings`, runtime feature flags, IBL required-source binding, pipeline validation x2).
- Runtime UI text/layout/extract: 5 failures (two intrinsic measurement expectations and three text extraction expectations).

No external-owner source was modified by this Runtime 02 slice. The gate remains open until those active owners land and a fresh binary rerun is green.

## Existing closed evidence reused

- Default-feature runtime library check: passed in the existing Runtime 02 archive.
- `export_build_plan`: current Runtime 01 acceptance passed 67/67 after provider-contract reconciliation.
- Runtime 02 static structure audit remains risks-empty, and the core root contains only `framework`, `manager`, `math`, `resource`, `runtime`, and `mod.rs`.

## Downstream app gate

- `cargo test -p zircon_app --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-runtime02-app-0711`: exit 0 after a fresh 25m17s build.
- Exact app lib-test executable rerun: 135 passed / 0 failed / 1 ignored in 40.24s. The ignored dynamic-runtime capture requires the documented ZR VM/runtime-library environment.

## Remaining gates

`core::`, `runtime_absorption`, current structure convention, a newly compiled package-level review filter, full Runtime lib tests, `zircon_editor`, and plugin-workspace all-targets validation remain before Runtime 02 can complete. The `zircon_app` gate and current-source review-guard slice are closed.

## 2026-07-12 continuation evidence

- Fresh current-worktree Python audits for `runtime_root_surface`,
  `core_spine_root_generated_boundary`, and `generated_code_boundary` all report
  `risks = []`; both root-surface and generated M1 gates remain
  `classified-and-clear`.
- A fresh standalone compile of the current `code_review_findings` sources passes
  80/80. This replaces the older package test binary as the authoritative result
  for the priority review guard while full Cargo compilation is shared with active
  owners.
- A fresh standalone compile of the current structure-convention sources initially
  passes 1295/1304. Two Runtime 15 guard drifts were then repaired without changing
  production behavior: the skinning helper anchor follows the hard cut from
  `oversized_uniform_skeleton` to `oversized_storage_skeleton`, and the deferred
  Unlit dispatch guard now preserves the emissive call inside the authoritative
  volumetric wrapper. Both exact current-source guards pass 1/1 after recompilation.
- The remaining structure failures are real size-boundary gates, not Runtime 02
  core/root/generated regressions: five production files exceed the global 800-line
  budget (`scan_and_import.rs` 803, `ibl_bake_wgpu_command_plan.rs` 802,
  render-pass `gpu.rs` 810, screen-space UI `render.rs` 802, rich layout `rich.rs`
  831), `compute_workload.rs` exceeds its 680-line child budget at 690, and the
  froxel integrate test owner is 861 lines. Active Render/IBL/UI owners are not
  overwritten; unowned files require a behavior-preserving folder-backed split.
- Runtime 02 remains `in_progress`. A fresh Cargo package build and the broader
  `core::`/`runtime_absorption`/editor/plugin gates remain pending until the active
  shared lanes converge.
- Fresh `cargo test -p zircon_runtime --lib --locked --no-run --jobs 1` on the
  coordinator-owned `D:\cargo-targets\zircon-runtime02-current-20260712` lane
  completed after 22m22s with exit 101. All seven compile errors are confined to
  the untracked Shader 06 owner
  `environment/realtime_ibl_graph_plan/tests.rs`: two missing scheduler imports,
  two missing config imports, one missing completion import, and two owned-`String`
  arguments passed where `resource_lifetime_by_name` requires `&str`. Runtime 02
  core/root/generated code emitted no error. The active Shader 06 continuation
  owns this lower-layer repair; Runtime 02 does not claim its test-binary gate.
- Fresh `cargo check -p zircon_runtime --lib --locked --jobs 1` on the same
  coordinator-managed target succeeds with exit 0 in 13m26s. The current dirty
  production library therefore compiles after the Runtime 02 hard cut; the output
  contains 469 existing warnings and no error. This closes the default-feature
  Runtime library check independently from the Shader 06 lib-test blocker.
- After the Shader 06 continuation resumed, the previously failing realtime-IBL
  test owner now statically contains all three imports and both required borrows.
  A concurrent Render 18 `zircon_runtime --lib --no-run` job is rebuilding the
  current lib-test surface; Runtime 02 keeps the lib-test gate pending until that
  compile has an authoritative exit code.
- The seven remaining structure-convention exact gates were rerun after current
  owner activity and all seven still fail with the same budgets. No expectation
  was weakened: the five global production oversizes, the 690/680 compute-workload
  child, and the 861-line froxel test owner require real folder-backed splits.

## 2026-07-12 priority structure convergence records

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | M1-S1 exact owner claim and current-source reread | `completed` | 2026-07-12 | Coordinator Session `runtime02-priority-structure-v2-20260712` obtained exact leases for the Physics and compute-workload guards; current owners were reread before mutation. |
| M1 | M1-S2 Physics collider-shape guard anchor convergence | `completed` | 2026-07-12 | Fresh RED: `runtime_15_scene_world_property_access_physics_entries_are_child_owner` failed only on `"{path_prefix}.kind"`; guard now requires the real `format!("{prefix}.{}", $suffix)` and `push_shape_entry!("kind"` anchors without changing production code or budgets. |
| M1 | M1-S3 compute-workload child routing verification | `completed` | 2026-07-12 | Newly compiled current harness passed `runtime_15_render_graph_execution_record_is_folder_backed` 1/1; the guard reads production anchors from `compute_workload.rs` and moved test anchors from `compute_workload/tests.rs`, so no edit was made. |
| M1 | M1-S4 scoped formatting and whitespace check | `completed` | 2026-07-12 | `rustfmt --edition 2021` on the Physics guard and exact-path `git diff --check` both exited 0; no production owner was formatted. |
| M1 | M1-T1 current structure harness compile | `completed` | 2026-07-12 | MSVC `rustc --test` rebuilt `D:\cargo-targets\runtime02-structure-standalone-20260712\runtime02_structure_current.exe` from current sources with exit 0; the temporary wrapper was deleted immediately. |
| M1 | M1-T2 seven historical exact structure gates | `completed` | 2026-07-12 | Newly compiled harness reported `exact_passed=7 exact_failed=0`; both global budgets, IBL/render/GPU/UI owner checks, Physics child projection, and compute-workload child routing passed. |
| M1 | M1-T3 full current structure-convention suite | `completed` | 2026-07-12 | The newly compiled current-source harness passed 1304/1304 tests (`0 failed`) in 246.78s, closing the priority structure gate without weakening any production-file or child-owner budget. |
| M2 | M2-S1 current code-review harness compile | `completed` | 2026-07-12 | MSVC `rustc --test` rebuilt `D:\cargo-targets\runtime02-structure-standalone-20260712\runtime02_review_current.exe` from the current `code_review_findings` source with exit 0; the temporary wrapper was deleted immediately after compilation. |
| M2 | M2-S2 current code-review findings convergence | `completed` | 2026-07-12 | Fresh current-source RED was 79/80: D10 alone still required obsolete single-hit `Option` signatures after Physics 03 hard-cut ray/shape casts to hit vectors. The leased D10 guard was aligned to the runtime-owned `Vec` contract; focused D10 passed 1/1 and the rebuilt full review suite passed 80/80 in 10.12s. |
| M2 | M2-S3 priority structure/review evidence consolidation | `completed` | 2026-07-12 | Current authoritative results are structure convention 1304/1304 and code-review findings 80/80. All six Runtime 15 owners named by the approved split design are below their existing budgets, while Physics collider-shape and render-graph compute test anchors are validated from their authoritative child owners. Runtime 02 remains `in_progress`; broader Cargo, editor, and plugin gates are not claimed by this slice. |
| M2 | M2-T1 output-record and scoped-diff audit | `completed` | 2026-07-12 | The repository output-record audit reported five pre-existing violations, all under `docs/plans/zircon_editor/{editor,editor_ui}` and none in Runtime 02 or this slice. Scoped `git diff --check` exited 0 (line-ending warnings only), and `temporary_runtime02_wrappers=0`. The unrelated Editor records were preserved for their owners. |
| M2 | M2-T2 priority-slice acceptance review | `completed` | 2026-07-12 | Exact-path status confirms the three leased guards plus approved design, implementation plan, and Runtime 02 archive only; none of those paths is staged. The archive explicitly keeps Runtime 02 `in_progress`, no temporary wrapper remains, and no broader Cargo/editor/plugin completion is asserted. |
| M2 | M2-T3 fresh completion verification | `completed` | 2026-07-12 | After the D10 guard convergence, both standalone harnesses were rebuilt from the final current sources. Fresh verification passed code-review findings 80/80 and the full structure-convention suite 1304/1304 in 289.54s; both commands exited 0 and temporary wrappers were removed before execution. |
| M3 | M3-S1 Runtime 02 static architecture audits | `completed` | 2026-07-12 | Fresh direct audits of `runtime_root_surface`, `core_spine_root_generated_boundary`, and `generated_code_boundary` all exited 0 with `risks = []`. Root surface and generated-code M1 gates are `classified-and-clear`; core root exactly matches `framework/manager/math/resource/runtime/mod.rs`, and generated migration debt remains 0. The aggregate JSON renderer separately exceeded its output timeout, so acceptance uses the same three audit implementations with scoped output. |
| M3 | M3-S2 Runtime 02 core/root/generated guard harness | `completed` | 2026-07-12 | A fresh MSVC standalone harness compiled the current `root_entries`, `root_surface`, `generated_code_guard`, and `core_spine_root_generated` owners, then passed 31/31 tests in 16.34s. The temporary wrapper was removed before test execution. |
| M3 | M3-T1 compiled `core::` package-filter gate | `failed` | 2026-07-12 | The freshly generated current `zircon_runtime` lib-test binary ran 675 tests: 664 passed and 11 failed. No core-spine/root/generated guard failed. The string filter also selects subsystem paths containing `::core::`; failures are owned by Render/Shader (4), Dynamic Scene payload-version migration (2), and Runtime UI/Text (5). Runtime 02 keeps this upward gate open and does not weaken the filter. |
| M3 | M3-T2 compiled `runtime_absorption` gate | `failed` | 2026-07-12 | The same current lib-test binary ran 1633 absorption tests in 474.80s: 1629 passed and 4 failed. One local failure was Runtime 02 `last_refined` status drift; the remaining failures are Editor workbench document routing, Render fallback-fixture naming, and Runtime 07 submit-context anchor drift. The broad gate remains open pending owned repairs and a fresh compiled rerun. |
| M3 | M3-S3 runtime plan refinement-date maintenance | `completed` | 2026-07-12 | Explicit coordinator maintenance authorization updated Runtime 02 `last_refined` to 2026-07-12. A full mismatch inventory also found Runtime 09 behind its latest archive date and synchronized it to 2026-07-12; Runtime 15 converged concurrently. A freshly compiled plan-status harness then passed `runtime_plan_last_refined_covers_latest_recorded_date` 1/1. |
| M3 | M3-T2 compiled `runtime_absorption` gate | `failed` | 2026-07-12 | The current lib-test binary ran 1633 absorption tests: 1629 passed and 4 failed. Runtime 02's own stale `last_refined` date was repaired under explicit plan maintenance; the same guard then exposed Runtime 15's stale date, which was likewise synchronized, and the focused current-source plan-status harness passed 1/1. Three external-owner failures remain: Editor workbench document path, Render fallback-fixture names, and Runtime 07 submit-context sharing anchor. |
| M3 | M3-S3 current absorption guard routing convergence | `completed` | 2026-07-12 | The three M3-T2 external-owner failures were re-read against their current split owners and repaired only in absorption guards. The Editor naming guard no longer reads the concurrently retired `docs/zircon_editor/ui/host/commands.md`; the render fallback guard combines production `compute_workload.rs` with moved fixtures in `compute_workload/tests.rs`; and the Runtime 07 submit guard follows `camera_loop_submissions_from_cameras(&extract.view.cameras)` into `resolve_camera_sequence_borrowed(cameras)`. A newly compiled standalone current-source harness passed all three focused guards 3/3, and its temporary wrapper was deleted before execution. The full compiled absorption gate remains pending a fresh package binary. |
| M3 | M3-S4 Runtime 05 DynamicScene archive fixture handoff | `implemented-pending-package-rerun` | 2026-07-12 | The two DynamicScene failures from M3-T1 were imported through `05/failure-2026-07-12-dynamic-scene-version-validation.md`. Architecture reread confirmed the typed `$zircon` envelope is the version authority and canonical writing intentionally normalizes the temporary inner `format_version`; no production validation change remains. Runtime 05 updated the two stale archive fixtures plus their structure owner anchors to `normalizes_noncanonical_inner_*`. A newly compiled exact current-source structure guard passed 1/1 after its plan evidence reads were hard-cut to Runtime 15 numbered output records; package behavior rerun is still required. |
| M3 | M3-T3 current `runtime_absorption` package rerun | `failed-follow-up-implemented` | 2026-07-12 | A freshly rebuilt current lib-test binary ran 1633 tests: 1631 passed and 2 failed in 383.23s. All three M3-S3 guard-routing fixes and the Runtime 05 archive fixture/structure tests passed. Remaining failures were a native live-host typed-error guard requiring one exact import spelling despite equivalent split imports, and Runtime 05 `last_refined` lag. The guard now checks semantic import/source-preservation anchors, its newly compiled exact current-source harness passes 1/1, and Runtime 05 metadata is synchronized to 2026-07-12 with the compiled plan-status test passing 1/1. One final package rerun is required before this gate can be marked completed. |
| M3 | M3-T4 current `core::` package-filter rerun | `failed-external-owners` | 2026-07-12 | The final current package binary ran 675 tests: 666 passed and 9 failed in 94.52s. The two Runtime 05 DynamicScene failures are gone, and no Runtime 02 core/root/generated guard failed. The remaining filter overmatches are four graphics/render fixture-validation failures and five UI/Text measurement/extract failures; these stay with their lower-layer Render/Shader/UI/Text owners and keep the Runtime 02 upward gate open. |
| M3 | M3-S5 final absorption failure-group current-source verification | `completed` | 2026-07-12 | After concurrent owner convergence, a fresh standalone MSVC harness rebuilt the four previously failing absorption checks from current sources. Editor workbench authority, render fallback-fixture naming, Runtime 07 submit-context sharing, and runtime-plan refinement-date coverage each executed exactly once and passed (4/4 total). This confirms the older failing package snapshots are stale for these four checks; the full 1633-test absorption gate remains open until a fresh package binary rerun succeeds. |
| M3 | M3-S6 current plan-status and Runtime 07 hotspot guard suite | `completed` | 2026-07-12 | The same freshly rebuilt current-source harness then ran its complete mounted surface: all 81 plan-status, Runtime 15 naming-boundary, and Runtime 07 performance-hotspot guards passed with 0 failures in 1.26s. This verifies the 01–15 status map, non-empty evidence records, pending-gate visibility, latest-record dates, and approved folder-backed split anchors while package compilation remains in progress. |
| M3 | M3-T5 fresh current package no-run | `failed-external-render-owner` | 2026-07-12 | A coordinator-managed Windows ephemeral target rebuilt the default-feature `zircon_runtime` lib-test surface from current sources for 985.2s and exited 101 before test execution. The sole compile error is Render 18 product test `graphics/tests/render_product_planar_reflection.rs:398`: `RenderGraphComputeWorkload::viewport` requires `[u32; 3]`, while the concurrently edited imported planar-filter workgroup constant was observed as a two-element array in this snapshot. The Render 18 failure lifecycle is already open at `render/18/failure-2026-07-12-planar-filter-test-surface-export.md`; Runtime 02 preserves that owner boundary and keeps all package filters open. |
| M3 | M3-S7 cross-plan current architecture regression sweep | `completed` | 2026-07-12 | A final current-source Python regression sweep passed 14/14 in 8.712s across Runtime 03 schedule/status, Runtime 04 asset/schema, domain-dependency scanning, Runtime 08 ECS, Runtime 09 UI text surface, Runtime 10 archive ownership, Runtime 11 JobSystem, Runtime 12 input, Runtime 13 script binding, and Runtime 06 plugin descriptor projection. Runtime 14 direct module-family audit independently remains `risks = []`. These static gates do not replace package filters; active Frameworks/Render owners are rebuilding the shared runtime binaries after the Render 18 snapshot drift. |
| M3 | M3-S8 aggregate architecture hard-cutover risk convergence | `completed` | 2026-07-12 | The aggregate runtime interface audit initially reported only hard-cutover migration-smell risk: seven Runtime asset locations plus six unclassified Runtime-interface locations around the retired `{ uuid, url }` reference format. Runtime 04 performed a direct owner/API/doc/test cutover to explicit `retired_asset_ref_migration` naming and deleted the old folder/API. Fresh evidence is hard-cutover audit 2/2 `classified-and-clear`, Runtime interface 233/233, structure convention 1304/1304, and code-review findings 80/80. Runtime 02 package filters remain pending a full test binary compiled after this cutover. |
| M3 | M3-T6 final current runtime package build and absorption gate | `package-build-and-absorption-completed-core-filter-external-failures` | 2026-07-12 | Coordinator-managed current default-feature `zircon_runtime --lib --no-run` rebuilt successfully in 28m47s and produced the authoritative package binary. The first absorption run was 1632/1633 with only Runtime 04 `last_refined` lag; explicit maintenance synchronized Runtime 04 plus the other 2026-07-12 acceptance plans, the exact date guard passed 1/1, and the same current binary then passed `runtime_absorption` 1633/1633 in 379.73s. The broad string `core::` filter now reports 669/675, with no Runtime 02 core/root/generated failure; remaining overmatches are four Render/Shader fixture-validation tests and two Runtime 09 UI render-extract tests, owned by their lower-layer plans. |
| M3 | M3-T7 final current `core::` package-filter convergence | `completed` | 2026-07-12 | The six M3-T6 filter overmatches were repaired at their lowest owners: pipeline fixtures now author explicit `Core2d` camera intent, Button/renderer extract fixtures follow dedicated painter commands and authored `label` precedence, and Shader 06 returned the async-compute/required-external fixture handoff. A fresh default-feature Runtime lib no-run completed with exit 0, the four local exact regressions passed 4/4, and the final current binary passed the unchanged `core::` string filter 675/675 in 124.77s. No Runtime 02 core/root/generated test failed. |

## 2026-07-14 priority gates and current package continuation

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | Priority review-findings current-source gate | `completed-80-of-80` | 2026-07-14 | Fresh standalone harness rebuilt from the current `code_review_findings.rs` owner and passed 80/80 in 9.14s. This is the current evidence for the user-prioritized `engine-code-review-findings-2026-06.md` gate. |
| M3 | Runtime plan-status date convergence | `completed-48-of-48` | 2026-07-14 | Fresh plan-status harness first reported only Runtime13 `last_refined=2026-07-12` behind its existing 2026-07-13 output record. Exact plan-maintenance updated that metadata to 2026-07-13; the unchanged fresh harness then passed 48/48 in 5.69s. No lifecycle status was promoted. |
| M3 | Priority structure-convention current-source convergence | `completed-1304-of-1304` | 2026-07-14 | Fresh structure harness first passed 1303/1304; the sole RED required the pre-generational `HostRegistry` `HashMap/lock_handles` spelling after the active ZrVM owner hard-cut the production registry to generational `HostRegistryState { slots, free_slots }`. Runtime15 changed only the guard to require the real poison-recovering `lock_state` read/write boundary; exact passed 1/1 in 0.21s and the rebuilt full structure-convention suite passed 1304/1304 in 256.26s. Production ZrVM code and budgets were not changed. Detailed record: `../15/2026-07-14-script-host-registry-guard-sync.md`. |
| M3/M4 | Current default-feature Runtime package build before Text05 cut | `completed-current-no-run-with-routed-filter-failures` | 2026-07-14 | Coordinator job `e041d3dc7f804f20977ac7bbb241f424` rebuilt the fresh default-feature Runtime lib-test in 28m47s with exit 0 and 7931-test inventory. Before the Text05 hard cut, `generated` was 28/29 and `runtime_absorption` was 1626/1637; three failures were the new unclassified `font_sdf_build_tool` root seat/split anchor, while the other eight belonged to Runtime06/07/11/13/15. `core::` passed 705/705. |
| M3/M4 | Text05 namespace hard cut and current package rerun | `root-generated-core-completed-absorption-external-owner-failures` | 2026-07-14 | After moving the build tool under `graphics::text` and removing the root declaration, fresh default-feature Runtime lib-test no-run returned 0. `generated` passed 29/29 and unchanged `core::` passed 705/705. `runtime_absorption` improved to 1629/1637: all three Text05 root-surface failures disappeared; eight remaining failures are Runtime06 date mirror, Runtime07 profiling anchor, Runtime11 job/Rayon owners, Runtime13/15 naming and lifecycle owners. Runtime02 stays `in_progress` until all required upward package/app/editor/plugin/full-lib gates are current and green. |
