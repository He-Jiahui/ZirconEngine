const EXPECTED_RUNTIME_03_SOURCE_FILES: &[&str] = &[
    "src/dynamic_api/session.rs",
    "src/dynamic_api/session/profile.rs",
    "src/dynamic_api/session/extract.rs",
    "src/dynamic_api/runtime_loop.rs",
    "src/dynamic_api/session/hud.rs",
    "src/dynamic_api/session/menu.rs",
    "src/scene/level_system.rs",
    "src/scene/module/world_driver.rs",
    "src/scene/ecs/system_stage.rs",
    "src/scene/ecs/schedule_stage_plan.rs",
    "src/scene/ecs/schedule_runner.rs",
    "src/scene/ecs/schedule_parallel_executor.rs",
    "src/scene/ecs/scene_system_descriptor.rs",
    "src/scene/ecs/scene_system_registry.rs",
    "src/core/runtime/handle/time.rs",
    "src/core/runtime/time.rs",
    "src/core/runtime/frame_clock.rs",
    "src/core/framework/time/clock.rs",
    "src/core/framework/time/fixed_step_plan.rs",
];

const EXPECTED_RUNTIME_03_GUARD_FILES: &[&str] = &[
    "src/scene/tests/ecs_schedule.rs",
    "src/scene/tests/ecs_schedule/fixed_update.rs",
    "src/scene/tests/ecs_schedule/parallel_executor.rs",
    "src/scene/tests/ecs_schedule_parallel_executor_structure.rs",
    "src/dynamic_api/tests/session_profiles.rs",
    "src/tests/time.rs",
    "src/tests/runtime_absorption/schedule_frame_loop.rs",
    "src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
];

const EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS: &[&str] = &[
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
];

#[test]
fn runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS.len(), 13);

    for source_file in EXPECTED_RUNTIME_03_SOURCE_FILES {
        assert!(
            runtime_root.join(source_file).exists(),
            "Runtime 03 source owner `{source_file}` is missing; update schedule_frame_loop_boundary before changing the schedule/frame-loop surface"
        );
    }
    for guard_file in EXPECTED_RUNTIME_03_GUARD_FILES {
        assert!(
            runtime_root.join(guard_file).exists(),
            "Runtime 03 guard owner `{guard_file}` is missing; update schedule_frame_loop_boundary before changing guard coverage"
        );
    }

    let system_stage = include_str!("../../scene/ecs/system_stage.rs");
    for stage_anchor in [
        "pub const COUNT: usize = 9;",
        "pub const ORDER: [Self; Self::COUNT]",
        "pub const FIXED_LOOP: [Self; 3]",
        "pub const fn rank(self) -> usize",
        "pub const fn is_fixed_loop(self) -> bool",
    ] {
        assert!(
            system_stage.contains(stage_anchor),
            "SystemStage authority should retain `{stage_anchor}`"
        );
    }

    let session = include_str!("../../dynamic_api/session.rs");
    let session_profile = include_str!("../../dynamic_api/session/profile.rs");
    assert_eq!(
        session.matches(".tick_time(").count(),
        1,
        "dynamic session should keep RuntimeTimeAdvance as the single tick_time handoff"
    );
    assert!(
        session.contains("fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>"),
        "dynamic session should keep the typed tick_frame result signature"
    );
    assert!(
        session_profile.contains("DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME: u32 = 8"),
        "dynamic session profile should keep the documented fixed-step cap"
    );

    let world_driver = include_str!("../../scene/module/world_driver.rs");
    assert!(
        !world_driver.contains("advance_time_by("),
        "WorldDriver must not reintroduce a second time-advance path"
    );

    let schedule_runner = include_str!("../../scene/ecs/schedule_runner.rs");
    for runner_anchor in [
        "runtime_frame_schedule_stage.{stage:?}",
        "ScheduledSceneStepRef::Internal",
        "ScheduledSceneStepRef::ApplyDeferred",
        "ScheduledSceneStepRef::Hook",
        "world.apply_deferred()",
    ] {
        assert!(
            schedule_runner.contains(runner_anchor),
            "schedule runner should retain stage/deferred/hook anchor `{runner_anchor}`"
        );
    }

    let behavior_test_sources = [
        include_str!("../../scene/tests/ecs_schedule.rs"),
        include_str!("../../scene/tests/ecs_schedule/fixed_update.rs"),
        include_str!("../../scene/tests/ecs_schedule/parallel_executor.rs"),
        include_str!("../../scene/tests/ecs_schedule_parallel_executor_structure.rs"),
        include_str!("../../dynamic_api/tests/session_profiles.rs"),
        include_str!("../../tests/time.rs"),
        include_str!("schedule_frame_loop.rs"),
        include_str!("plan_status/cargo_gates/early.rs"),
    ];
    for behavior_anchor in EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS {
        assert!(
            behavior_test_sources
                .iter()
                .any(|source| source.contains(behavior_anchor)),
            "Runtime 03 behavior test anchor `{behavior_anchor}` should stay visible to schedule_frame_loop_boundary"
        );
    }

    let mirror_docs = [
        (
            "Runtime 03 plan",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"),
        ),
        (
            "frame schedule doc",
            include_str!("../../../../docs/zircon_runtime/core/frame_schedule.md"),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "schedule_frame_loop_boundary",
            "source files 19/19",
            "guard/test files 8/8",
            "`SystemStage` count and variants 9/9",
            "fixed-loop stages 3/3",
            "dynamic-session `.tick_time(...)` calls 1/1",
            "Runtime 03 guard anchors 14/14",
            "behavior_test_anchor_count = 13",
            "missing_behavior_test_anchors = []",
            "doc_anchors = 10/10",
            "no `WorldDriver` second `advance_time_by(...)` references",
            "no dynamic-session raw-delta level tick references",
            "risks = []",
            "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 03 schedule/frame-loop audit anchor `{required_anchor}`"
            );
        }
    }
}
