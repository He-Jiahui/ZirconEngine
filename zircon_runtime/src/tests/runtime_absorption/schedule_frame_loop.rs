const EXPECTED_RUNTIME_03_SOURCE_FILES: &[&str] = &[
    "src/dynamic_api/session.rs",
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

#[test]
fn runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

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
    assert_eq!(
        session.matches(".tick_time(").count(),
        1,
        "dynamic session should keep RuntimeTimeAdvance as the single tick_time handoff"
    );
    assert!(
        session.contains("DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME: u32 = 8"),
        "dynamic session should keep the documented fixed-step cap"
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

    let mirror_docs = [
        (
            "Runtime 03 plan",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"),
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
            "source files 18/18",
            "guard/test files 8/8",
            "`SystemStage` count and variants 9/9",
            "fixed-loop stages 3/3",
            "dynamic-session `.tick_time(...)` calls 1/1",
            "Runtime 03 guard anchors 14/14",
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
