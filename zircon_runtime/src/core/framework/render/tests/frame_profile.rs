use super::{
    RenderBudgetKey, RenderFrameBudget, RenderFrameProfile, RenderGpuTimingStatus,
    RenderMeshSubmissionProfile, RenderPassNativeResourceCreateMetrics, RenderPassProfileEntry,
};

#[test]
fn reference_1080p_mid_budget_covers_each_profile_category() {
    let budget = RenderFrameBudget::reference_1080p_mid();

    assert_eq!(budget.total_budget_us(), 14_000);
    assert_eq!(budget.entries().len(), RenderBudgetKey::ALL.len());
    assert_eq!(
        budget
            .entries()
            .iter()
            .map(|(_, budget_us)| *budget_us)
            .sum::<u64>(),
        budget.total_budget_us()
    );
    assert_eq!(budget.budget_us(RenderBudgetKey::Shadow), 2_200);
    assert_eq!(budget.budget_us(RenderBudgetKey::BasePass), 3_200);
    assert_eq!(budget.budget_us(RenderBudgetKey::Other), 0);
}

#[test]
fn empty_frame_profile_reports_no_gpu_timing_until_the_timer_resolves() {
    let profile = RenderFrameProfile::default();

    assert_eq!(profile.gpu_frame_time_us, None);
    assert_eq!(profile.gpu_timing_status, RenderGpuTimingStatus::Disabled);
    assert!(profile.passes.is_empty());
    assert!(profile.subsystems.is_empty());
}

#[test]
fn legacy_frame_profile_json_defaults_missing_mesh_submission_metrics() {
    let profile = RenderFrameProfile {
        mesh_submission: RenderMeshSubmissionProfile {
            command_count: 9,
            cached_command_hit_count: 5,
            ..RenderMeshSubmissionProfile::default()
        },
        ..RenderFrameProfile::default()
    };
    let mut legacy = serde_json::to_value(profile).expect("frame profile serializes");
    legacy
        .as_object_mut()
        .expect("frame profile is a JSON object")
        .remove("mesh_submission");

    let decoded: RenderFrameProfile =
        serde_json::from_value(legacy).expect("legacy frame profile remains readable");

    assert_eq!(
        decoded.mesh_submission,
        RenderMeshSubmissionProfile::default()
    );
}

#[test]
fn mesh_submission_profile_json_defaults_missing_opaque_phase_counts() {
    let profile = RenderFrameProfile {
        mesh_submission: RenderMeshSubmissionProfile {
            opaque_command_count: 2,
            advanced_pbr_opaque_command_count: 1,
            ..RenderMeshSubmissionProfile::default()
        },
        ..RenderFrameProfile::default()
    };
    let mut serialized = serde_json::to_value(profile).expect("frame profile serializes");
    let mesh_submission = serialized
        .as_object_mut()
        .and_then(|profile| profile.get_mut("mesh_submission"))
        .and_then(serde_json::Value::as_object_mut)
        .expect("serialized frame profile contains mesh submission metrics");
    mesh_submission.remove("opaque_command_count");
    mesh_submission.remove("advanced_pbr_opaque_command_count");

    let decoded: RenderFrameProfile =
        serde_json::from_value(serialized).expect("legacy frame profile remains readable");

    assert_eq!(decoded.mesh_submission.opaque_command_count, 0);
    assert_eq!(decoded.mesh_submission.advanced_pbr_opaque_command_count, 0);
}

#[test]
fn legacy_pass_profile_json_defaults_missing_cpu_time() {
    let entry = RenderPassProfileEntry {
        pass_name: "opaque".to_owned(),
        executor_id: "mesh.opaque".to_owned(),
        budget_key: RenderBudgetKey::BasePass,
        cpu_elapsed_micros: 41,
        gpu_time_us: None,
        pipeline_statistics: None,
        draw_count: 1,
        instance_count: 1,
        state_change_count: 0,
        upload_bytes: 0,
        dispatch_count: 0,
        native_resource_creates: RenderPassNativeResourceCreateMetrics::new(1, 2, 3, 4, 5, 6, 7),
    };
    let mut legacy = serde_json::to_value(entry).expect("pass profile serializes");
    let legacy_object = legacy
        .as_object_mut()
        .expect("pass profile is a JSON object");
    legacy_object.remove("cpu_elapsed_micros");
    legacy_object.remove("native_resource_creates");

    let decoded: RenderPassProfileEntry =
        serde_json::from_value(legacy).expect("legacy pass profile remains readable");

    assert_eq!(decoded.cpu_elapsed_micros, 0);
    assert_eq!(
        decoded.native_resource_creates,
        RenderPassNativeResourceCreateMetrics::default()
    );
}

#[test]
fn native_resource_create_metrics_keep_categories_and_saturating_total() {
    let metrics = RenderPassNativeResourceCreateMetrics::new(1, 2, 3, 4, 5, 6, u32::MAX);

    assert_eq!(metrics.buffer_count, 1);
    assert_eq!(metrics.bind_group_count, 2);
    assert_eq!(metrics.bind_group_layout_count, 3);
    assert_eq!(metrics.shader_module_count, 4);
    assert_eq!(metrics.pipeline_layout_count, 5);
    assert_eq!(metrics.compute_pipeline_count, 6);
    assert_eq!(metrics.render_pipeline_count, u32::MAX);
    assert_eq!(metrics.total_count(), u32::MAX);
}

#[test]
fn legacy_frame_profile_json_defaults_missing_parallel_recording_counts() {
    let profile = RenderFrameProfile {
        parallel_recording_eligible_stage_count: 1,
        parallel_recording_eligible_bucket_count: 3,
        parallel_recording_executed_stage_count: 1,
        parallel_recording_executed_bucket_count: 2,
        ..RenderFrameProfile::default()
    };
    let mut legacy = serde_json::to_value(profile).expect("frame profile serializes");
    let object = legacy
        .as_object_mut()
        .expect("frame profile is a JSON object");
    for field in [
        "gpu_timing_status",
        "parallel_recording_eligible_stage_count",
        "parallel_recording_eligible_bucket_count",
        "parallel_recording_executed_stage_count",
        "parallel_recording_executed_bucket_count",
    ] {
        object.remove(field);
    }

    let decoded: RenderFrameProfile =
        serde_json::from_value(legacy).expect("legacy frame profile remains readable");

    assert_eq!(decoded.gpu_timing_status, RenderGpuTimingStatus::Disabled);
    assert_eq!(decoded.parallel_recording_eligible_stage_count, 0);
    assert_eq!(decoded.parallel_recording_eligible_bucket_count, 0);
    assert_eq!(decoded.parallel_recording_executed_stage_count, 0);
    assert_eq!(decoded.parallel_recording_executed_bucket_count, 0);
}
