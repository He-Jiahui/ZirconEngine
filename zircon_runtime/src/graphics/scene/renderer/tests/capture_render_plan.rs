use super::*;

#[test]
fn plan_has_one_opaque_winding_safe_pass_per_cubemap_face() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_face_size(256)
        .unwrap();
    let plan = EnvironmentCaptureRenderPlan::from_request(&request);

    assert_eq!(plan.total_pass_count(), 6);
    for (index, pass) in plan.passes().iter().copied().enumerate() {
        assert_eq!(pass.face().index(), index);
        assert_eq!(pass.color_array_layer(), index as u32);
        assert_eq!(pass.uniform_slot(), index as u32);
        assert!(pass.reverse_raster_winding());
        assert!(pass.opaque_only());
    }
    assert_eq!(plan.pass(CubemapFace::NegativeZ).uniform_slot(), 5);
    assert_eq!(plan.target().face_size(), 256);
}

#[test]
fn plan_keeps_target_mip_budget_and_capture_pass_count_together() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_face_size(1024)
        .unwrap();
    let plan = EnvironmentCaptureRenderPlan::from_request(&request);

    assert_eq!(plan.target().source_mip_count(), 11);
    // BUG: [CR-SCENE-ENV-0003] 旧预算只计源与深度，遗漏固定 PMREM 的 1,048,560 字节；
    // 同一 TargetPlan 的当前总纹理预算为 72,351,712，见 gpu_target 的最大请求测试。
    assert_eq!(plan.target().total_texture_bytes(), 71_303_152);
    assert_eq!(
        plan.total_pass_count() as u32,
        RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT
    );
}
