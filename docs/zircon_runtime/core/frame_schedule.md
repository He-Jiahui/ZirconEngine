---
related_code:
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/core/framework/render/ui_submission.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/system/native/mod.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/runtime_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system_metadata.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - docs/zircon_plugins/plugin-sdk.md
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/schedule_plan.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop/mirror_docs.rs
  - tools/tests/test_runtime_schedule_frame_loop_audit.py
  - tests/acceptance/runtime-schedule-frame-loop-audit-owner-sync.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_markdown.py
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
implementation_files:
  - docs/zircon_runtime/core/frame_schedule.md
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/system/native/mod.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/runtime_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system_metadata.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - docs/zircon_plugins/plugin-sdk.md
  - docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - reactive_cadence_coalesces_requests_and_suppresses_idle_frames
  - continuous_cadence_never_suppresses_frame_pumps
  - headless_cadence_uses_fixed_wait_deadlines
  - headless_early_wake_does_not_pump_or_move_fixed_deadline
  - redraw_delivery_does_not_schedule_another_reactive_frame
  - python -m unittest tools.tests.test_runtime_schedule_frame_loop_audit
  - tests/acceptance/runtime-schedule-frame-loop-audit-owner-sync.md
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/time/fixed_step_plan.rs zircon_runtime/src/tests/time.rs zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/scene/level_system.rs zircon_runtime/src/scene/module/world_driver.rs zircon_runtime/src/scene/tests/ecs_schedule.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_scene_hooks.rs zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - source scans for retired raw-delta level tick and world-driver second advance paths
  - schedule_frame_loop_boundary targeted audit: source files 22/22, guard/test files 11/11, SystemStage 9/9, fixed_loop 3/3, tick_time calls 1/1, Runtime 03 guard anchors 14/14, behavior_test_anchor_count = 14, missing_behavior_test_anchors = [], doc_anchors = 10/10, mirror-doc aggregate guard present, frame schedule module-doc anchors 3/3, risks = []
  - schedule_frame_loop_inventory_split_static_passed_cargo_deferred_tests_deferred: source/guard inventory split into schedule_frame_loop_source_inventory.py, anchor inventory split into schedule_frame_loop_anchor_inventory.py, boundary audit kept at 475 lines, standalone schedule_frame_loop.rs 1/1, standalone plan_status.rs 33/33, Cargo gates deferred
  - schedule_frame_loop_markdown_split_static_passed_cargo_deferred_tests_deferred: Markdown renderer split into schedule_frame_loop_markdown.py, boundary audit reduced to 368 lines, markdown owner 146 lines, standalone schedule_frame_loop.rs 1/1, standalone plan_status.rs 33/33, Cargo gates deferred
  - schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration
  - session_ui_extract_remains_documented_dynamic_session_side_path
  - world_driver_consumes_runtime_time_advance_without_advancing_clocks_again
  - world_driver_pauses_virtual_systems_and_runs_explicit_real_time_systems
  - core_runtime_virtual_pause_preserves_existing_fixed_overstep
  - level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps
  - level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained
  - level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance
  - fixed_step_plan_separates_interpolation_fraction_from_total_debt
  - schedule_parallel_executor_can_run_parallel_batches_serially_with_report
  - schedule_parallel_execution_report_records_diagnostic_counts
  - representative_schedule_produces_multi_system_parallel_batches
  - parallel_and_serial_execution_reach_identical_world_state
  - schedule_parallel_report_keeps_run_batches_compatible
  - schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts
  - cargo test -p zircon_runtime --lib ecs_schedule --locked --target-dir E:/cargo-targets/zircon-runtime-03-0612 -- --nocapture --test-threads=1 failed before executing runtime 03 tests on unrelated unresolved import `crate::asset::ui_v2_asset_references` in zircon_runtime/src/ui/tests/asset_dependency_index.rs
  - 2026-07-04 gate recheck: ecs_schedule 77/77, fixed_update 3/3, schedule_parallel 15/15, session_profiles 6/6, session_lifecycle destroy guard 1/1, Runtime10 owner split guard 1/1, Runtime05 path guard 1/1, and dynamic-scene selected retention/restore 4/4 passed; exact time module path `cargo test -p zircon_runtime --lib tests::time:: --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-03-gates-0704 --message-format short --color never -- --test-threads=1` passed 4/4; broad `cargo test -p zircon_runtime --lib session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-03-gates-0704 --message-format short --color never -- --test-threads=1` passed 161/0/10 ignored after Vampire real-backend default gates were completed. Bare `--lib time` is a retired invalid gate because the Rust test filter also matches unrelated `runtime` tests.
doc_type: module-detail
---

# Runtime Frame Schedule

This document is the runtime-owned frame-loop record for plan 03. It records the current frame path after the M2.1 single-time-advance handoff, the M2.2 fixed overstep interpolation accessor, the 2026-06-13 schedule/frame-loop structural audit owner, and the 2026-06-21 splits of source/guard inventory, anchor inventory, and Markdown rendering.

## Current Conclusion

The runtime has a single authoritative stage enum, fixed-loop stages, a profile-aware host cadence owner, and a single outer-frame time handoff for the dynamic session path. The `SystemStage` declaration is owned by the neutral `core/framework/scene` contract layer; the scene scheduler consumes that declaration and owns execution, with no second scene-local enum. `RuntimeDynamicSession::tick_frame` advances the shared monotonic source once through `tick_time(...)`, passes the resulting `FrameTimeSnapshot` through `LevelSystem`, and `WorldDriver` derives one immutable `WorldTimeSnapshot` from that input without calling `advance_time_by(...)` again.

The remaining higher-level design choice is whether a future UI/render plan wants to move UI extraction into a scheduled `RenderExtract` producer. For the runtime 03 plan, the current contract is explicit: UI extraction is a legal dynamic-session side path.

## Current Frame Chain

1. The winit host `RuntimeEntryApp::about_to_wait(...)` delegates to `pump_frame_loop(...)`. `RuntimeFrameCadence::take_frame_request()` admits or suppresses the host pump according to the selected product profile.
2. An admitted host pump calls the dynamic ABI entry `zircon_runtime/src/dynamic_api/session.rs::tick_frame(handle)`.
3. `RuntimeDynamicSession::tick_frame` calls `self.runtime.tick_time(self.profile.max_fixed_steps_per_frame())`.
4. The profile cap comes from `DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME = 8` in `zircon_runtime/src/dynamic_api/session/profile.rs`, returned through `max_fixed_steps_per_frame()`.
5. `CoreHandle::tick_time(...)` at `zircon_runtime/src/core/runtime/handle/time.rs:43` samples `FrameClock::tick()` and delegates to `advance_time_by(...)`.
6. `CoreHandle::advance_time_by(...)` at `zircon_runtime/src/core/runtime/handle/time.rs` captures one monotonic outer-frame delta, its source-generation stamp, discontinuity, and the fixed-step budget, then records frame diagnostics.
7. The Core timing owner remains the shared monotonic source and no longer has virtual/fixed snapshot fields. Each Level owns a `WorldTimeController` for pause, scale, epochs, fixed accumulation, debt, and fixed-step commit.
8. `RuntimeDynamicSession::tick_frame` passes the full `FrameTimeSnapshot` into `LevelSystem::tick(...)`.
9. `LevelSystem::tick(...)` resolves `WorldDriver` and calls `driver.tick_level(core, self, snapshot)`.
10. `WorldDriver::tick_level(...)` advances the Level's controller once using the outer real delta and budget. It builds immutable `SystemTickContext` values from the resulting World-local virtual/fixed stamps and from the shared real stamp.
11. `WorldDriver` consumes the World-local `FixedStepPlan`: when the schedule reaches `SystemStage::FixedFirst`, it runs every stage in `SystemStage::FIXED_LOOP` once per committed step, then skips fixed-loop stages in the outer stage iteration.
12. `run_stage(...)` in `zircon_runtime/src/scene/module/world_driver.rs` delegates to `SceneScheduleRunner::run_stage(...)`.
13. `SceneScheduleRunner::run_stage(...)` executes `Internal`, `Native`, `Runtime`, and `ApplyDeferred` steps. Internal systems except `ApplyDeferred` and `UpdateEvents`, plus Runtime steps, flush deferred world work at their explicit schedule boundaries.

The old gap was step 10: `WorldDriver` used to advance time again after the dynamic session had already called `tick_time(...)`. Current source has removed that second advance.

`SceneSystemMetadata` defaults native and runtime systems to `SceneSystemTickPolicy::virtual_time()`. When a Level's virtual time is paused, `WorldDriver` skips those systems and the built-in scene systems instead of executing the normal schedule with a zero delta; the runner's stage-success cleanup also leaves pending derived-state work untouched until that World resumes. The World controller preserves any pre-existing fixed overstep without accumulating or draining it, so fixed-loop callbacks cannot leak through a paused frame. A diagnostic or editor runtime system that must continue can opt into `SceneSystemTickPolicy::monotonic_real()` through the runtime registry or plugin SDK registration builder; its callback receives the unclamped real delta while virtual systems receive the scaled and clamped World-local delta. The registry rejects a non-fixed policy for `FixedFirst`, `FixedUpdate`, and `FixedPostUpdate`, because those stages are governed exclusively by the World-local fixed-step plan.

## Product Profile Cadence

`zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs` is the host cadence owner. It keeps cadence state separate from `RuntimeEntryAppConfig`: config selects `EventLoopPolicy`, construction consumes it into one `RuntimeFrameCadence`, and the app does not keep a second policy field.

| Product policy | Winit control flow | Pump admission |
|---|---|---|
| `Game` | `Poll` while focused and visible; otherwise `WaitUntil(next_deadline)` | Focused visible gameplay remains continuous. An unfocused visible window uses a 10 Hz low-power deadline; an occluded window uses a 1 Hz background deadline. Explicit event requests still coalesce and can admit one immediate pump. |
| `Continuous` | `Poll` | Explicit display/debug throughput override; focus and occlusion do not throttle it. |
| `Mobile` | `WaitUntil(next_deadline)` | 60 Hz while focused and visible, 1 Hz while unfocused or occluded. Explicit event requests still coalesce and can admit one immediate pump. |
| `DesktopApp` | `Wait` | One initial frame, then only after a non-redraw window event, device event, resume/surface creation, or `proxy_wake_up`. Repeated requests coalesce. |
| `Headless` | `WaitUntil(next_deadline)` | Fixed periodic pump without a window/redraw dependency. The persistent deadline advances only after a due pump, so an early OS/proxy wake neither pumps an extra frame nor shifts the interval. The 16 ms local policy constant is private to this host cadence owner. |

The profile split follows Bevy's focused/unfocused `WinitSettings` while retaining Unreal's rule that focus loss is an engine-loop cadence concern rather than a Windows-only branch. Focus and occlusion update the same `RuntimeFrameCadence` before their ABI events are dispatched.

Reactive admission occurs before gamepad polling, `tick_frame`, host-request draining, and OS redraw. An idle Desktop `about_to_wait` therefore returns before all four. The explicit window-event relevance table includes only handled runtime events; raw device admission includes only consumed pointer motion. Focus/occlusion are edge-owned by their lifecycle handlers, so repeated state notifications do not reset low-power deadlines. `WindowEvent::RedrawRequested` is excluded from new frame admission, so presenting an admitted frame cannot schedule itself forever. The existing tick → host request → redraw order is unchanged for admitted frames, and each pump publishes final control flow once.

Low-power modes consume the same `Idle` / `Immediate` / `After` runtime demand as Desktop reactive mode. `Immediate` uses the capacity-one frame request and host wake; `After` publishes the earlier of the runtime deadline and profile period; `Idle` clears only the runtime deadline. If a producer creates another request during an admitted pump, final control-flow publication observes that pending token as one `Poll`; the next `take_frame_request` consumes it before Reactive/LowPower returns to `Wait` / `WaitUntil`. This also preserves an already-pending request when runtime `Immediate` coalesces or `Idle` clears its own deadline. `RuntimeEntryApp` logs `runtime_frame_cadence_summary` on shutdown with request-attempt, accepted, coalesced, ignored, pump, idle-suppression, redraw-request, focus-transition, occlusion-transition, low-power-pump, and low-power-suppression counts. Those counters are the application-side correlate for the required WPR CPU/wakeup trace; they are not a substitute for WPR acceptance.

The reactive contract is not yet a fixed return. OS input, the `ApplicationHandler::proxy_wake_up` consumer, and the capacity-one project-generation producer are wired in current source, including empty-change reconciliation commits. Until the managed app tests, Desktop 30-second WPR trace, event-storm/duplicate-resize counters, and runtime-origin wake product regression pass, the Runtime03 idle-cadence failure remains open.

## Stage Table

`SystemStage` in `core/framework/scene/system_stage.rs` is the single runtime stage authority. Current source has 9 stages, not the older 7-stage shape; `scene/ecs` only consumes and exposes that contract for scheduling APIs and does not define another stage enum:

| Rank | Stage | Fixed loop role |
|---:|---|---|
| 0 | `First` | none |
| 1 | `PreUpdate` | none |
| 2 | `FixedFirst` | fixed-loop entry and first fixed stage |
| 3 | `FixedUpdate` | fixed-loop stage |
| 4 | `FixedPostUpdate` | fixed-loop stage |
| 5 | `Update` | none |
| 6 | `PostUpdate` | none |
| 7 | `Last` | none |
| 8 | `RenderExtract` | render extraction preparation stage |

The authority points are:

- `SystemStage::COUNT = 9` at `zircon_runtime/src/core/framework/scene/system_stage.rs:18`.
- `SystemStage::ORDER` at `zircon_runtime/src/core/framework/scene/system_stage.rs:19`.
- `SystemStage::FIXED_LOOP = [FixedFirst, FixedUpdate, FixedPostUpdate]` at `zircon_runtime/src/core/framework/scene/system_stage.rs:30`.
- `SystemStage::rank()` and `is_fixed_loop()` at `zircon_runtime/src/core/framework/scene/system_stage.rs:32` and `:46`.

## Extract Path

Scene extraction is currently pull-based from the dynamic session, not proven to be produced by a scheduled `RenderExtract` system:

- `capture_frame(...)` builds a `RenderFrameExtract` and optional `UiRenderSubmission` in `zircon_runtime/src/dynamic_api/session.rs`, then submits through `submit_extract_with_ui(...)`.
- `present_viewport(...)` builds the same scene extract and UI submission in `zircon_runtime/src/dynamic_api/session.rs`, then presents through `present_extract_with_ui(...)`.
- `current_extract(...)` in `zircon_runtime/src/dynamic_api/session/extract.rs` reads the world and calls `world.to_render_frame_extract().with_viewport_size(...)`.
- `current_ui_submission(...)` in `zircon_runtime/src/dynamic_api/session/extract.rs` first publishes the runtime project's ordered surface segments; when no project UI surface exists, it wraps the menu-first/HUD-second cached `UiRenderExtract` as one segment.
- `RuntimeRenderBridge::submit_extract_with_ui(...)` and `present_extract_with_ui(...)` in `zircon_runtime/src/dynamic_api/runtime_loop.rs` apply viewport size and forward the extract to the resolved render framework.

Current verdict: the UI submission path is a documented legal side path, not part of the scheduled `RenderExtract` stage. `session_ui_extract_remains_documented_dynamic_session_side_path` guards the current contract by checking both capture/present consumers, project-surface publication, and the fallback menu-then-HUD producer order.

The side-path inventory is:

| Producer or consumer | Current role | M0 verdict |
|---|---|---|
| `RuntimeDynamicSession::current_ui_submission` in `session/extract.rs` | Publishes ordered project-surface segments, otherwise wraps menu-first/HUD-second fallback as one segment | Legal side path, owner is dynamic session |
| `runtime_session_menu_extract` at `session/menu.rs:47` | Builds menu UI commands from runtime menu state | Legal side path, not a schedule stage producer |
| `runtime_session_hud_extract` at `session/hud.rs:19` | Builds text HUD UI commands from world text state | Legal side path, not a schedule stage producer |
| `RuntimeRenderBridge::*_with_ui` in `runtime_loop.rs` | Submits optional `UiRenderSubmission` beside the scene extract | Legal consumer; render framework owns the segmented contract |

No new UI submission producer should be added without updating this table.

## Time Authority

The current time model keeps outer-frame and simulation ownership separate:

- Core owns the real outer-frame clock and the default policy used when a new Level is created; it has no global virtual/fixed clock.
- `FrameTimeSnapshot` carries raw real delta, outer-frame index, real source stamp, optional discontinuity, and an admitted fixed-step budget.
- `WorldTimeSnapshot` carries the Level-local virtual/fixed observation and its `FixedStepPlan`.
- `FixedStepPlan` carries `step_count`, `timestep`, `consumed`, and `remaining_overstep`.
- `FixedStepPlan` exposes full debt duration, whole-step count, and unbounded timestep ratio for health and scheduling decisions; `interpolation_fraction()` exposes only the fractional remainder for adjacent-state interpolation.
- `WorldTimeController` proposes bounded fixed work, then each `WorldFixedStep` consumes one timestep only on commit.

The owner wiring is now:

- The dynamic session advances time once through `tick_time(...)` using the fixed-step budget from its accepted `ProductTimePolicy`.
- Dynamic lifecycle, occlusion, and surface-recreation signals submit a typed `ClockDiscontinuity` to `CoreRuntime` before the next frame; its receipt rebases the monotonic source with `MeasureFromRebase` and appears once in the next snapshot.
- Tests and deterministic callers can explicitly create the same type through `CoreRuntime::advance_time_by(...)`.
- `LevelSystem::tick(...)` accepts `FrameTimeSnapshot`, not raw seconds.
- `WorldDriver::tick_level(...)` derives the Level-local proposal and begins, commits, or aborts one fixed transaction per admitted step; it does not own a second cap.
- Runtime scene systems access the immutable `SystemTickContext` through `RuntimeSceneSystemContext::tick()`. The context carries stage, clock-domain stamp, outer-frame index, optional fixed simulation tick, `Duration` delta and elapsed, and world generation. Fixed-loop dispatches use `WorldFixed` and advance elapsed/tick evidence one committed step at a time.
- Core time diagnostics are real-frame-only. Future fixed telemetry must be emitted from the World committed-step receipt, never from a pre-advanced global clock.

The fixed-loop behavior has targeted owner tests in `zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs`: `level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps`, `level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained`, and `level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance`. The 2026-07-04 gate recheck passed the focused `fixed_update` gate 3/3 and the broader `ecs_schedule` gate 77/77. That is historical Runtime03 evidence only: it does not validate the Runtime22 fixed-step transaction or core-time hard cut, whose managed Cargo and profiling gates remain pending.

## Stage Ordering Inventory

The current ECS schedule is not purely registration-order based:

- `SceneSystemDescriptor` supports `order`, `sets`, `before`, and `after` constraints in `zircon_runtime/src/scene/ecs/scene_system_descriptor.rs`.
- `SceneScheduleStagePlan::from_registry(...)` at `zircon_runtime/src/scene/ecs/schedule_stage_plan.rs:13` builds per-stage groups and calls `topological_stage_order(...)`.
- `topological_stage_order(...)` at `zircon_runtime/src/scene/ecs/schedule_stage_plan.rs:200` resolves same-stage constraints and falls back to `order` plus id through `compare_plan_nodes(...)` at `:327`.
- Runtime-owned built-in scene systems are explicitly ordered in `zircon_runtime/src/scene/ecs/scene_system_registry.rs:318`: hierarchy validity, active hierarchy, world transform, node cache, and render extract prepare all set negative order values.
- External system registration exposes explicit order, constraint, and clock-domain data through the plugin registration builder and native host adapter. Runtime22 edits the runtime registry and plugin SDK public owners to propagate that clock-domain contract.

M0 inventory verdict:

| Area | Evidence | Verdict |
|---|---|---|
| Built-in scene systems | `builtin_scene_systems()` uses explicit `with_order(...)` values | Accepted |
| Same-stage ordering core | `schedule_stage_plan.rs` uses topological order with order/id fallback | Accepted |
| Dynamic/native plugin systems | Adapter maps order and `before`/`after` into descriptors; all system metadata carries a `SceneSystemTickPolicy` with clock domain and pause behavior, and plan compilation rejects invalid stage/domain combinations | Accepted; plugin-public owners updated and covered by focused registration tests |
| UI extract side path | Produced outside scheduled `RenderExtract`; source guard documents capture/present consumers and menu-then-HUD producer order | Accepted as a documented side path |
| Single time authority | Session passes `FrameTimeSnapshot`; `WorldDriver` does not call `advance_time_by(...)` | Code converged; focused time/fixed-update evidence passed; declared broad `time/session` gates still open |

## Parallel Schedule Observability

Runtime 03 M3.1 adds executor-level observability without changing the frame owner:

- `ScheduleParallelExecutor::run_batches(...)` still keeps the old result-only compatibility path.
- `run_batches_with_report(...)` returns `ScheduleParallelExecutionReport`.
- `with_parallel_enabled(false)` disables parallel batch execution and runs every batch serially through the same task registry.
- `ScheduleParallelExecutionReport::record_diagnostics(...)` writes `schedule.parallel_batches` and `schedule.serial_fallbacks` through core diagnostics.
- The representative M3.2 fixture currently produces 3 two-system batches. Default execution reports 3 parallel batches; disabled execution reports 3 serial fallbacks; both paths reach the same representative world state.

The diagnostic write remains report-owned. A future frame owner can call it at the point it considers authoritative for a frame without making the executor depend on dynamic session or scene-level state.

Detailed owner notes live in `docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md`.

### Production native-system path

The production `SceneScheduleRunner` is distinct from the generic executor above. Its compiled native-system plan uses `SystemParamAccess` conflict metadata to form worker-safe batches, temporarily takes those systems out of `World`, and executes them through `JobScheduler::join` without sharing `&mut World` between workers. Worker-local command queues merge in stable system order; non-worker-safe, Internal, Runtime, and deferred-barrier work remains on the main-thread lane. Worker callbacks and command application share one unwind boundary, so taken systems are restored before either panic resumes.

`NativeSystemScheduleDiagnostics` publishes worker-batch/conflict counts, ready delay, worker utilization, callback p95, callback/conservative-writer counters, plus `scene.ecs.native_system.temporary_control_buffer_count` and `scene.ecs.native_system.temporary_control_buffer_bytes`. Overlap currently has behavior-test evidence but no published product counter. The last two values count temporary worker-batch containers and their capacity-byte proxy; they are not allocator-call counters and do not establish an allocation-performance acceptance result. Managed Cargo and the F2 product overlap/World-lock/allocation matrix remain pending.

## Structural Audit Mirror

`schedule_frame_loop_source_inventory.py` now owns the source/guard file inventory, stage count, fixed-loop count, and dynamic-session tick-count source scans, including the split `dynamic_api/session/profile.rs` owner for the fixed-step cap and the current child test owners `schedule_plan.rs`, `world_driver.rs`, `world_time_controller.rs`, and `schedule_frame_loop/mirror_docs.rs`. `schedule_frame_loop_anchor_inventory.py` owns the SystemStage, FrameTimeSnapshot, FixedStepPlan, UI extract, stage ordering, schedule runner, parallel executor, behavior-test, mirror-doc, and Cargo gate anchors; the time gate uses the precise `tests::time::` module filter because bare `time` also matches unrelated `runtime` tests. `schedule_frame_loop_markdown.py` owns `render_schedule_frame_loop_boundary_markdown`. `schedule_frame_loop_boundary` mirrors this document without running Cargo and is now the audit reader, missing-anchor checker, and risk classifier at 368 lines; the Markdown owner is 146 lines. The following source counts and `risks = []` result are archived Runtime03 audit evidence, not Runtime22 acceptance: source files 22/22, guard/test files 11/11, `SystemStage` count and variants 9/9, fixed-loop stages 3/3, dynamic-session `.tick_time(...)` calls 1/1, Runtime 03 guard anchors 14/14, `behavior_test_anchor_count = 14`, `missing_behavior_test_anchors = []`, `doc_anchors = 10/10`, `mirror_docs_guard_present = true`, frame schedule module-doc anchors 3/3, no `WorldDriver` second `advance_time_by(...)` references, and no dynamic-session raw-delta level tick references. `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` keeps this document aligned with Runtime 03, the runtime index, the M0 review, and runtime-interface convergence.

The 2026-07-04 Cargo recheck passed `ecs_schedule` 77/77, focused `fixed_update` 3/3, `schedule_parallel` 15/15, exact `tests::time::` 4/4, and `session_profiles` 6/6 after synchronizing the source guard with the then-current `session.rs`, `session/profile.rs`, and `session/extract.rs` owner split. The follow-up owner-sync passes also cleared `session_lifecycle` destroy owner/FFI-wrapper drift 1/1, Runtime10 session test-owner drift 1/1, Runtime05 dynamic-scene path guard drift 1/1, and dynamic-scene selected retention/restore behavior 4/4. The naked `--lib time` filter is retired as invalid because it matched broad `runtime` tests and failed at 1809 passed / 279 failed; `cargo test -p zircon_runtime --lib tests::time:: ...` is the current time gate. The broad `--lib session` default command then passed with 161 passed / 0 failed / 10 ignored after all Vampire project-session behavior tests that initialize the authored ZrVM backend were marked as requiring `backend-zr-vm`. These historical Cargo results are not evidence for the later Runtime22 hard cut or fixed-step transaction; no new Cargo result is claimed here.

## Follow-Up Work

1. Keep the Runtime03 gate command shape on `tests::time::`; do not reintroduce naked `--lib time`.
2. Run full `cargo test -p zircon_app --locked` only when the app/startup integration lane is intentionally being closed.
3. Revisit UI extraction only if the UI/render architecture plan explicitly decides to move the side path into a scheduled `RenderExtract` producer.
