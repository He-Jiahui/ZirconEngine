use super::*;

#[test]
fn target_plan_counts_full_rgba16f_cube_chain_and_one_depth_face() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_face_size(128)
        .unwrap();
    let plan = EnvironmentCaptureGpuTargetPlan::from_request(&request);

    assert_eq!(plan.face_size(), 128);
    assert_eq!(plan.source_mip_count(), 8);
    assert_eq!(plan.color_texture_bytes(), 1_048_560);
    assert_eq!(plan.depth_texture_bytes(), 65_536);
    assert_eq!(plan.pmrem_texture_bytes(), 1_048_560);
    assert_eq!(
        plan.sh9_buffer_bytes(),
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64
    );
    assert_eq!(plan.sh9_buffer_bytes(), 144);
    assert_eq!(plan.total_texture_bytes(), 2_162_656);
    assert_eq!(plan.total_gpu_bytes(), 2_162_800);
}

#[test]
fn maximum_request_exposes_bounded_admission_cost_before_allocation() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_face_size(1024)
        .unwrap();
    let plan = EnvironmentCaptureGpuTargetPlan::from_request(&request);

    assert_eq!(plan.source_mip_count(), 11);
    assert_eq!(plan.color_texture_bytes(), 67_108_848);
    assert_eq!(plan.depth_texture_bytes(), 4_194_304);
    assert_eq!(plan.pmrem_texture_bytes(), 1_048_560);
    assert_eq!(
        plan.sh9_buffer_bytes(),
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64
    );
    assert_eq!(plan.sh9_buffer_bytes(), 144);
    assert_eq!(plan.total_texture_bytes(), 72_351_712);
    assert_eq!(plan.total_gpu_bytes(), 72_351_856);
}

#[test]
fn resident_output_budget_excludes_source_and_depth_scratch() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_face_size(1024)
        .unwrap();
    let plan = EnvironmentCaptureGpuTargetPlan::from_request(&request);

    assert_eq!(
        plan.pmrem_texture_bytes() + plan.sh9_buffer_bytes(),
        1_048_704
    );
    assert!(plan.total_gpu_bytes() > 72_000_000);
}
