use super::*;

#[test]
fn volumetric_apply_layout_reserves_plan18_bindings() {
    let entries = volumetric_apply_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT);
    assert_eq!(
        entries.map(|entry| entry.binding),
        [
            VOLUMETRIC_APPLY_PARAMS_BINDING,
            VOLUMETRIC_INTEGRATED_BINDING,
            VOLUMETRIC_SAMPLER_BINDING,
        ]
    );
}
