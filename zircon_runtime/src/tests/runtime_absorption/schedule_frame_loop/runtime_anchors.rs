use super::inventory::{
    EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS, EXPECTED_RUNTIME_03_GUARD_FILES,
    EXPECTED_RUNTIME_03_SOURCE_FILES,
};

pub(super) fn assert_runtime_03_sources_and_anchors(runtime_root: &std::path::Path) {
    assert_eq!(EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS.len(), 13);
    assert_expected_files_exist(runtime_root);
    assert_system_stage_contract();
    assert_dynamic_session_time_handoff();
    assert_world_driver_has_no_second_time_advance();
    assert_schedule_runner_contract();
    assert_behavior_test_anchors();
}

fn assert_expected_files_exist(runtime_root: &std::path::Path) {
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
}

fn assert_system_stage_contract() {
    let system_stage = include_str!("../../../core/framework/scene/system_stage.rs");
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
}

fn assert_dynamic_session_time_handoff() {
    let session = include_str!("../../../dynamic_api/session/state.rs");
    let session_profile = include_str!("../../../dynamic_api/session/profile.rs");
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
}

fn assert_world_driver_has_no_second_time_advance() {
    let world_driver = include_str!("../../../scene/module/world_driver.rs");
    assert!(
        !world_driver.contains("advance_time_by("),
        "WorldDriver must not reintroduce a second time-advance path"
    );
}

fn assert_schedule_runner_contract() {
    let schedule_runner = include_str!("../../../scene/ecs/schedule_runner.rs");
    for runner_anchor in [
        "profile_scope!",
        "schedule_stage_profile_name(stage)",
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
    for stage_name in [
        "First",
        "PreUpdate",
        "FixedFirst",
        "FixedUpdate",
        "FixedPostUpdate",
        "Update",
        "PostUpdate",
        "Last",
        "RenderExtract",
    ] {
        let profile_name = format!("runtime_frame_schedule_stage.{stage_name}");
        assert!(
            schedule_runner.contains(&profile_name),
            "schedule runner should retain static stage profile label `{profile_name}`"
        );
    }
    assert!(
        !schedule_runner.contains("format!(\"runtime_frame_schedule_stage"),
        "schedule runner should not format static stage labels per frame"
    );
}

fn assert_behavior_test_anchors() {
    let behavior_test_sources = [
        include_str!("../../../scene/tests/ecs_schedule.rs"),
        include_str!("../../../scene/tests/ecs_schedule/fixed_update.rs"),
        include_str!("../../../scene/tests/ecs_schedule/parallel_executor.rs"),
        include_str!("../../../scene/tests/ecs_schedule_parallel_executor_structure.rs"),
        include_str!("../../../dynamic_api/tests/session_profiles.rs"),
        include_str!("../../time.rs"),
        include_str!("inventory.rs"),
    ];
    for behavior_anchor in EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS {
        assert!(
            behavior_test_sources
                .iter()
                .any(|source| source.contains(behavior_anchor)),
            "Runtime 03 behavior test anchor `{behavior_anchor}` should stay visible to schedule_frame_loop_boundary"
        );
    }
}
