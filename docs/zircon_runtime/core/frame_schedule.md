---
related_code:
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
implementation_files:
  - docs/zircon_runtime/core/frame_schedule.md
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md
plan_sources:
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/time/fixed_step_plan.rs zircon_runtime/src/tests/time.rs zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/scene/level_system.rs zircon_runtime/src/scene/module/world_driver.rs zircon_runtime/src/scene/tests/ecs_schedule.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_scene_hooks.rs zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - source scans for retired raw-delta level tick and world-driver second advance paths
  - schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration
  - session_ui_extract_remains_documented_dynamic_session_side_path
  - schedule_parallel_executor_can_run_parallel_batches_serially_with_report
  - schedule_parallel_execution_report_records_diagnostic_counts
  - representative_schedule_produces_multi_system_parallel_batches
  - parallel_and_serial_execution_reach_identical_world_state
  - schedule_parallel_report_keeps_run_batches_compatible
  - schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts
  - cargo test -p zircon_runtime --lib ecs_schedule --locked --target-dir E:/cargo-targets/zircon-runtime-03-0612 -- --nocapture --test-threads=1 failed before executing runtime 03 tests on unrelated unresolved import `crate::asset::ui_v2_asset_references` in zircon_runtime/src/ui/tests/asset_dependency_index.rs
doc_type: module-detail
---

# Runtime Frame Schedule

This document is the runtime-owned frame-loop record for plan 03. It records the current frame path after the M2.1 single-time-advance handoff and the M2.2 fixed overstep interpolation accessor.

## Current Conclusion

The runtime has a single authoritative stage enum, fixed-loop stages, and a single time-advance handoff for the dynamic session frame path. `RuntimeDynamicSession::tick_frame` advances time once through `tick_time(...)`, passes the resulting `RuntimeTimeAdvance` through `LevelSystem`, and `WorldDriver` consumes that plan without calling `advance_time_by(...)` again.

The remaining higher-level design choice is whether a future UI/render plan wants to move UI extraction into a scheduled `RenderExtract` producer. For the runtime 03 plan, the current contract is explicit: UI extraction is a legal dynamic-session side path.

## Current Frame Chain

1. The dynamic ABI entry `zircon_runtime/src/dynamic_api/session.rs:301` exposes `tick_frame(handle)`.
2. `RuntimeDynamicSession::tick_frame` at `zircon_runtime/src/dynamic_api/session.rs:548` calls `self.runtime.tick_time(self.profile.max_fixed_steps_per_frame())`.
3. The profile cap comes from `DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME = 8` at `zircon_runtime/src/dynamic_api/session.rs:93`, returned through `max_fixed_steps_per_frame()` at `zircon_runtime/src/dynamic_api/session.rs:359`.
4. `CoreHandle::tick_time(...)` at `zircon_runtime/src/core/runtime/handle/time.rs:43` samples `FrameClock::tick()` and delegates to `advance_time_by(...)`.
5. `CoreHandle::advance_time_by(...)` at `zircon_runtime/src/core/runtime/handle/time.rs:29` advances `RuntimeTimeClocks`, then records time diagnostics at `record_time_diagnostics(...)` around `zircon_runtime/src/core/runtime/handle/time.rs:77`.
6. `RuntimeTimeClocks::advance_by(...)` at `zircon_runtime/src/core/runtime/time.rs:45` advances real time, virtual time, fixed overstep, and drains a `FixedStepPlan`.
7. `RuntimeDynamicSession::tick_frame` passes the full `RuntimeTimeAdvance` into `LevelSystem::tick(...)` at `zircon_runtime/src/dynamic_api/session.rs:553`.
8. `LevelSystem::tick(...)` at `zircon_runtime/src/scene/level_system.rs:103` resolves `WorldDriver` and calls `driver.tick_level(core, self, advance)`.
9. `WorldDriver::tick_level(...)` at `zircon_runtime/src/scene/module/world_driver.rs:11` converts `advance.real_delta()` and the fixed timestep to `Real`, then consumes `advance.fixed_step_plan()`.
10. `WorldDriver` consumes that single `FixedStepPlan`: when the schedule reaches `SystemStage::FixedFirst`, it runs every stage in `SystemStage::FIXED_LOOP` once per drained step, then skips fixed-loop stages in the outer stage iteration.
11. `run_stage(...)` at `zircon_runtime/src/scene/module/world_driver.rs:64` delegates to `SceneScheduleRunner::run_stage(...)`.
12. `SceneScheduleRunner::run_stage(...)` at `zircon_runtime/src/scene/ecs/schedule_runner.rs:13` executes `Internal`, `Native`, `ApplyDeferred`, and `Hook` steps. Internal non-`ApplyDeferred` systems and hooks flush deferred world work after each step.

The old gap was step 9: `WorldDriver` used to advance time again after the dynamic session had already called `tick_time(...)`. Current source has removed that second advance.

## Stage Table

`SystemStage` is the single runtime stage authority. Current source has 9 stages, not the older 7-stage shape:

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

- `SystemStage::COUNT = 9` at `zircon_runtime/src/scene/ecs/system_stage.rs:17`.
- `SystemStage::ORDER` at `zircon_runtime/src/scene/ecs/system_stage.rs:18`.
- `SystemStage::FIXED_LOOP = [FixedFirst, FixedUpdate, FixedPostUpdate]` at `zircon_runtime/src/scene/ecs/system_stage.rs:29`.
- `SystemStage::rank()` and `is_fixed_loop()` at `zircon_runtime/src/scene/ecs/system_stage.rs:31` and `:45`.

## Extract Path

Scene extraction is currently pull-based from the dynamic session, not proven to be produced by a scheduled `RenderExtract` system:

- `capture_frame(...)` builds a `RenderFrameExtract` and optional UI extract at `zircon_runtime/src/dynamic_api/session.rs:638` and `:639`, then submits through `submit_extract_with_ui(...)` at `:642`.
- `present_viewport(...)` builds the same two extracts at `zircon_runtime/src/dynamic_api/session.rs:667` and `:668`, then presents through `present_extract_with_ui(...)` at `:670`.
- `current_extract(...)` at `zircon_runtime/src/dynamic_api/session.rs:695` reads the world and calls `world.to_render_frame_extract().with_viewport_size(...)`.
- `RuntimeRenderBridge::submit_extract_with_ui(...)` and `present_extract_with_ui(...)` in `zircon_runtime/src/dynamic_api/runtime_loop.rs` apply viewport size and forward the extract to the resolved render framework.

Current verdict: the UI extract path is a documented legal side path, not part of the scheduled `RenderExtract` stage. `session_ui_extract_remains_documented_dynamic_session_side_path` guards the current contract by checking both capture/present consumers and the menu-then-HUD producer order.

The side-path inventory is:

| Producer or consumer | Current role | M0 verdict |
|---|---|---|
| `RuntimeDynamicSession::current_ui_extract` at `session.rs:703` | Chooses menu extract first, HUD extract second | Legal side path, owner is dynamic session |
| `runtime_session_menu_extract` at `session/menu.rs:47` | Builds menu UI commands from runtime menu state | Legal side path, not a schedule stage producer |
| `runtime_session_hud_extract` at `session/hud.rs:19` | Builds text HUD UI commands from world text state | Legal side path, not a schedule stage producer |
| `RuntimeRenderBridge::*_with_ui` in `runtime_loop.rs` | Submits optional UI extract beside scene extract | Legal consumer; render internals untouched by this plan |

No new UI extract producer should be added without updating this table.

## Time Authority

The current time model contains the right pieces:

- `RuntimeTimeClocks` owns real, virtual, and fixed clocks.
- `RuntimeTimeAdvance` carries `real_delta` and `FixedStepPlan`.
- `FixedStepPlan` carries `step_count`, `timestep`, `consumed`, and `remaining_overstep`.
- `FixedStepPlan::overstep_fraction()` reports remaining overstep divided by timestep, clamped to `[0.0, 1.0]`.
- `Time<Fixed>::drain_steps(max_steps)` at `zircon_runtime/src/core/framework/time/clock.rs:133` drains fixed overstep with a cap.

The owner wiring is now:

- The dynamic session advances time once through `tick_time(max_fixed_steps = 8)`.
- Tests and deterministic callers can explicitly create the same type through `CoreRuntime::advance_time_by(...)`.
- `LevelSystem::tick(...)` accepts `RuntimeTimeAdvance`, not raw seconds.
- `WorldDriver::tick_level(...)` consumes that plan and no longer owns a local fixed-step cap.
- Time diagnostics are recorded by the time owner, not by world scheduling.

## Stage Ordering Inventory

The current ECS schedule is not purely registration-order based:

- `SceneSystemDescriptor` supports `order`, `sets`, `before`, and `after` constraints in `zircon_runtime/src/scene/ecs/scene_system_descriptor.rs`.
- `SceneScheduleStagePlan::from_registry(...)` at `zircon_runtime/src/scene/ecs/schedule_stage_plan.rs:13` builds per-stage groups and calls `topological_stage_order(...)`.
- `topological_stage_order(...)` at `zircon_runtime/src/scene/ecs/schedule_stage_plan.rs:200` resolves same-stage constraints and falls back to `order` plus id through `compare_plan_nodes(...)` at `:327`.
- Runtime-owned built-in scene systems are explicitly ordered in `zircon_runtime/src/scene/ecs/scene_system_registry.rs:318`: hierarchy validity, active hierarchy, world transform, node cache, and render extract prepare all set negative order values.
- External system registration exposes explicit order/constraint data through the plugin registration builder and native host adapter. Those files are plugin-public-surface owners, so this frame-schedule slice records them but does not edit them.

M0 inventory verdict:

| Area | Evidence | Verdict |
|---|---|---|
| Built-in scene systems | `builtin_scene_systems()` uses explicit `with_order(...)` values | Accepted |
| Same-stage ordering core | `schedule_stage_plan.rs` uses topological order with order/id fallback | Accepted |
| Dynamic/native plugin systems | Adapter maps order and `before`/`after` into descriptors; `plugin_system_constraints_order_registered_native_systems` covers reversed plugin registration order | Accepted; plugin-owner code untouched |
| UI extract side path | Produced outside scheduled `RenderExtract`; source guard documents capture/present consumers and menu-then-HUD producer order | Accepted as a documented side path |
| Single time authority | Session passes `RuntimeTimeAdvance`; `WorldDriver` does not call `advance_time_by(...)` | Code converged; Cargo pending |

## Parallel Schedule Observability

Runtime 03 M3.1 adds executor-level observability without changing the frame owner:

- `ScheduleParallelExecutor::run_batches(...)` still keeps the old result-only compatibility path.
- `run_batches_with_report(...)` returns `ScheduleParallelExecutionReport`.
- `with_parallel_enabled(false)` disables parallel batch execution and runs every batch serially through the same task registry.
- `ScheduleParallelExecutionReport::record_diagnostics(...)` writes `schedule.parallel_batches` and `schedule.serial_fallbacks` through core diagnostics.
- The representative M3.2 fixture currently produces 3 two-system batches. Default execution reports 3 parallel batches; disabled execution reports 3 serial fallbacks; both paths reach the same representative world state.

The diagnostic write remains report-owned. A future frame owner can call it at the point it considers authoritative for a frame without making the executor depend on dynamic session or scene-level state.

Detailed owner notes live in `docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md`.

## Follow-Up Work

1. Re-run Cargo validation after the unrelated UI asset dependency-index compile error is resolved.
2. Revisit UI extraction only if the UI/render architecture plan explicitly decides to move the side path into a scheduled `RenderExtract` producer.
