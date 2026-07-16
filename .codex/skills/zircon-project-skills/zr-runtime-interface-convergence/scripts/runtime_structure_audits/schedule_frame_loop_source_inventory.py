from __future__ import annotations


EXPECTED_STAGE_COUNT = 9
EXPECTED_FIXED_LOOP_STAGE_COUNT = 3
EXPECTED_DYNAMIC_SESSION_TICK_TIME_CALLS = 1

RUNTIME_03_SOURCE_FILES = (
    "zircon_runtime/src/dynamic_api/session/state.rs",
    "zircon_runtime/src/dynamic_api/session/profile.rs",
    "zircon_runtime/src/dynamic_api/session/extract.rs",
    "zircon_runtime/src/dynamic_api/runtime_loop.rs",
    "zircon_runtime/src/dynamic_api/session/hud.rs",
    "zircon_runtime/src/dynamic_api/session/menu.rs",
    "zircon_runtime/src/scene/level_system.rs",
    "zircon_runtime/src/scene/module/world_driver.rs",
    "zircon_runtime/src/core/framework/scene/system_stage.rs",
    "zircon_runtime/src/scene/ecs/schedule_stage_plan.rs",
    "zircon_runtime/src/scene/ecs/schedule_runner.rs",
    "zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs",
    "zircon_runtime/src/scene/ecs/scene_system_descriptor.rs",
    "zircon_runtime/src/scene/ecs/scene_system_registry.rs",
    "zircon_runtime/src/core/runtime/handle/time.rs",
    "zircon_runtime/src/core/runtime/time.rs",
    "zircon_runtime/src/core/runtime/frame_clock.rs",
    "zircon_runtime/src/core/framework/time/clock.rs",
    "zircon_runtime/src/core/framework/time/fixed_step_plan.rs",
)
RUNTIME_03_GUARD_FILES = (
    "zircon_runtime/src/scene/tests/ecs_schedule.rs",
    "zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs",
    "zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs",
    "zircon_runtime/src/scene/tests/ecs_schedule/schedule_plan.rs",
    "zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs",
    "zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs",
    "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
    "zircon_runtime/src/tests/time.rs",
    "zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs",
    "zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop/mirror_docs.rs",
    "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
)
