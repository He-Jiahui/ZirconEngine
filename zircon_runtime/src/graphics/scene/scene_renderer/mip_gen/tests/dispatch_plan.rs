use super::*;

#[test]
fn render_mipgen_pass_four_mips_per_dispatch() {
    let plan =
        MipGenDispatchPlan::new(2_048, 2_048, 1, 12).expect("12 levels fit a 2048 square texture");

    assert_eq!(plan.dispatch_count(), 3);
    assert_eq!(
        plan.dispatches()
            .iter()
            .map(MipGenDispatch::source_mip_level)
            .collect::<Vec<_>>(),
        vec![0, 4, 8]
    );
    assert_eq!(
        plan.dispatches()
            .iter()
            .map(MipGenDispatch::first_target_mip_level)
            .collect::<Vec<_>>(),
        vec![1, 5, 9]
    );
    assert_eq!(
        plan.dispatches()
            .iter()
            .map(MipGenDispatch::generated_mip_count)
            .collect::<Vec<_>>(),
        vec![4, 4, 3]
    );
    assert_eq!(plan.dispatches()[0].target_extent(), [1_024, 1_024]);
    assert_eq!(plan.dispatches()[1].target_extent(), [64, 64]);
    assert_eq!(plan.dispatches()[2].target_extent(), [4, 4]);
}

#[test]
fn mipgen_dispatch_plan_preserves_array_layers_in_workgroup_depth() {
    let plan = MipGenDispatchPlan::new(64, 32, 6, 7)
        .expect("six-layer texture has a valid full mip chain");

    assert_eq!(plan.texture_extent(), [64, 32]);
    assert_eq!(plan.array_layer_count(), 6);
    assert_eq!(plan.mip_level_count(), 7);
    assert_eq!(plan.dispatch_count(), 2);
    assert_eq!(plan.dispatches()[0].workgroup_count(), [4, 2, 6]);
    assert_eq!(plan.dispatches()[1].workgroup_count(), [1, 1, 6]);
}

#[test]
fn mipgen_dispatch_plan_rejects_impossible_texture_descriptions() {
    assert_eq!(
        MipGenDispatchPlan::new(0, 1, 1, 1),
        Err(MipGenPlanError::ZeroExtent {
            width: 0,
            height: 1
        })
    );
    assert_eq!(
        MipGenDispatchPlan::new(1, 1, 0, 1),
        Err(MipGenPlanError::ZeroArrayLayers)
    );
    assert_eq!(
        MipGenDispatchPlan::new(1, 1, 1, 0),
        Err(MipGenPlanError::ZeroMipLevels)
    );
    assert_eq!(
        MipGenDispatchPlan::new(4, 4, 1, 4),
        Err(MipGenPlanError::MipLevelsExceedExtent {
            requested: 4,
            maximum: 3
        })
    );
}
