from __future__ import annotations


MIRROR_DOCS_GUARD = (
    "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts"
)
STAGE_VARIANT_ANCHORS = (
    "Self::First",
    "Self::PreUpdate",
    "Self::FixedFirst",
    "Self::FixedUpdate",
    "Self::FixedPostUpdate",
    "Self::Update",
    "Self::PostUpdate",
    "Self::Last",
    "Self::RenderExtract",
)
SYSTEM_STAGE_ANCHORS = (
    "pub const ORDER: [Self; Self::COUNT]",
    "pub const FIXED_LOOP: [Self; 3]",
    "pub const fn rank(self) -> usize",
    "pub const fn is_fixed_loop(self) -> bool",
)
SESSION_TICK_ANCHORS = (
    "fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>",
    ".tick_time(self.profile.max_fixed_steps_per_frame())",
    ".tick(&self.runtime.handle(), advance)",
    "self.resolve_input_manager()",
    "DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME: u32 = 8",
)
TIME_HANDOFF_ANCHORS = (
    "pub fn tick(&self, core: &CoreHandle, advance: RuntimeTimeAdvance)",
    "driver.tick_level(core, self, advance)",
    "pub fn tick_level(",
    "advance: RuntimeTimeAdvance",
    "let fixed_step_plan = advance.fixed_step_plan();",
    "for _ in 0..fixed_step_plan.step_count",
    "for fixed_stage in SystemStage::FIXED_LOOP",
    "if stage.is_fixed_loop()",
    "SceneScheduleRunner::run_stage(",
)
FIXED_PLAN_ANCHORS = (
    "pub fn overstep_fraction(&self) -> f32",
    "remaining_overstep.as_secs_f64() / self.timestep.as_secs_f64()",
    ".clamp(0.0, 1.0)",
)
UI_EXTRACT_ANCHORS = (
    "fn current_ui_extract(&self)",
    "let ui = self.current_ui_extract();",
    "runtime_session_menu_extract(world, viewport_size)",
    ".or_else(|| runtime_session_hud_extract(world, viewport_size))",
    "submit_extract_with_ui",
    "present_extract_with_ui",
    "submit_frame_extract_with_ui",
    "present_frame_extract_with_ui",
)
SCHEDULE_ORDER_ANCHORS = (
    "pub enum SystemOrderingConstraint",
    "Before(SystemRef)",
    "After(SystemRef)",
    "pub fn before(self, reference: SystemRef) -> Self",
    "pub fn after(self, reference: SystemRef) -> Self",
    "pub fn with_order(mut self, order: i32) -> Self",
    "topological_stage_order(",
    "SystemOrderingConstraint::Before",
    "SystemOrderingConstraint::After",
    "compare_plan_nodes(",
    "builtin_scene_systems()",
)
SCHEDULE_RUNNER_ANCHORS = (
    "runtime_frame_schedule_stage.{stage:?}",
    "ScheduledSceneStepRef::Internal",
    "ScheduledSceneStepRef::ApplyDeferred",
    "ScheduledSceneStepRef::Hook",
    "world.apply_deferred()",
)
PARALLEL_EXECUTOR_ANCHORS = (
    'pub const SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC: &str = "schedule.parallel_batches";',
    'pub const SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC: &str = "schedule.serial_fallbacks";',
    "pub struct ScheduleParallelExecutionReport",
    "pub fn with_parallel_enabled(mut self, enabled: bool) -> Self",
    "pub fn run_batches_with_report",
    "pub fn record_diagnostics(&self, core: &CoreHandle, frame_index: u64)",
    ".schedule_after(",
)
RUNTIME_03_TEST_ANCHORS = (
    "schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration",
    "session_ui_extract_remains_documented_dynamic_session_side_path",
    "world_driver_consumes_runtime_time_advance_without_advancing_clocks_again",
    "level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps",
    "level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained",
    "level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance",
    "fixed_step_plan_reports_overstep_fraction_in_unit_range",
    "schedule_parallel_executor_can_run_parallel_batches_serially_with_report",
    "schedule_parallel_execution_report_records_diagnostic_counts",
    "schedule_parallel_report_keeps_run_batches_compatible",
    "schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts",
    "representative_schedule_produces_multi_system_parallel_batches",
    "parallel_and_serial_execution_reach_identical_world_state",
    "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
)
RUNTIME_03_BEHAVIOR_TEST_ANCHORS = (
    "schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration",
    "session_ui_extract_remains_documented_dynamic_session_side_path",
    "world_driver_consumes_runtime_time_advance_without_advancing_clocks_again",
    "level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps",
    "level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained",
    "level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance",
    "fixed_step_plan_reports_overstep_fraction_in_unit_range",
    "schedule_parallel_executor_can_run_parallel_batches_serially_with_report",
    "schedule_parallel_execution_report_records_diagnostic_counts",
    "schedule_parallel_report_keeps_run_batches_compatible",
    "schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts",
    "representative_schedule_produces_multi_system_parallel_batches",
    "parallel_and_serial_execution_reach_identical_world_state",
)
RUNTIME_03_DOC_ANCHORS = (
    "Runtime Frame Schedule",
    "UI extraction is a legal dynamic-session side path",
    "RuntimeTimeAdvance",
    "schedule.parallel_batches",
    "schedule_frame_loop_boundary",
    "behavior_test_anchor_count = 13",
    "missing_behavior_test_anchors = []",
    "runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation",
    "ecs_schedule/time/session/schedule_parallel",
    "ScheduleParallelExecutionReport",
)
FRAME_SCHEDULE_DOC_ANCHORS = (
    "guard/test files 11/11",
    "Runtime 03 guard anchors 14/14",
    "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
)
CARGO_GATE_ANCHORS = (
    "cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture",
    "cargo test -p zircon_runtime --lib session --locked",
    "cargo test -p zircon_app --locked",
    "cargo test -p zircon_runtime --lib fixed_update --locked -- --nocapture",
    "cargo test -p zircon_runtime --lib tests::time:: --locked",
    "cargo test -p zircon_runtime --lib schedule_parallel --locked -- --nocapture",
)
