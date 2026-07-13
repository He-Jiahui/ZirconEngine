use super::*;

#[test]
fn render_volumetric_light_scatter_upload_bytes_match_uniform_abi() {
    assert_eq!(FroxelLightScatterPipeline::UPLOADED_BYTES_PER_DISPATCH, 288);
}

#[test]
fn render_volumetric_light_scatter_shader_reuses_light_grid_and_shadow_atlas_contracts() {
    assert!(LIGHT_SCATTER_SHADER.contains("// include: zr_light_grid.wgsl"));
    assert!(LIGHT_SCATTER_SHADER.contains("// include: zr_shadow.wgsl"));
    assert!(LIGHT_SCATTER_SHADER.contains("zr_light_mask_word"));
    assert!(LIGHT_SCATTER_SHADER.contains("zr_gpu_light_shadow_visibility"));
    assert!(LIGHT_SCATTER_SHADER.contains("henyey_greenstein"));
    assert!(LIGHT_SCATTER_SHADER.contains("zr_froxel_world_position"));
    assert!(LIGHT_SCATTER_SHADER.contains("world_from_clip"));
    assert!(LIGHT_SCATTER_SHADER.contains("texture_storage_3d<rgba16float, write>"));

    let module = naga::front::wgsl::parse_str(LIGHT_SCATTER_SHADER)
        .expect("volumetric light scatter shader must parse");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("volumetric light scatter shader must validate");
}

#[test]
fn render_volumetric_light_scatter_phase_uses_light_travel_and_camera_outgoing_angle() {
    assert!(LIGHT_SCATTER_SHADER
        .contains("henyey_greenstein(params.phase_g.x, dot(incoming, -view_direction))"));
    assert!(!LIGHT_SCATTER_SHADER
        .contains("henyey_greenstein(params.phase_g.x, dot(incoming, view_direction))"));
}
