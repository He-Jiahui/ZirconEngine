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
        .contains("henyey_greenstein(params.phase_and_ambient.x, dot(incoming, -view_direction))"));
    assert!(!LIGHT_SCATTER_SHADER
        .contains("henyey_greenstein(params.phase_and_ambient.x, dot(incoming, view_direction))"));
}

#[test]
fn render_volumetric_light_scatter_keeps_authored_ambient_in_shadowed_froxels() {
    assert!(LIGHT_SCATTER_SHADER.contains("phase_and_ambient: vec4<f32>"));
    assert!(LIGHT_SCATTER_SHADER
        .contains("var lighting = max(params.phase_and_ambient.yzw, vec3<f32>(0.0));"));
    assert!(LIGHT_SCATTER_SHADER.contains("let current = vec4<f32>(media.rgb * lighting, media.a)"));
}

#[test]
fn render_volumetric_light_scatter_filters_lights_without_participation_marker() {
    assert!(LIGHT_SCATTER_SHADER.contains("if (light.cookie_misc.z == 0u)"));
    assert!(LIGHT_SCATTER_SHADER.contains("return vec3<f32>(0.0);"));
}

#[test]
fn render_volumetric_light_scatter_packs_phase_and_authored_ambient_without_abi_growth() {
    assert_eq!(
        pack_phase_and_ambient(1.25, Vec3::new(0.2, 0.3, 0.4)),
        [0.9, 0.2, 0.3, 0.4]
    );
    assert_eq!(
        pack_phase_and_ambient(0.25, Vec3::new(f32::NAN, 0.3, 0.4)),
        [0.25, 0.0, 0.0, 0.0]
    );
    assert_eq!(FroxelLightScatterPipeline::UPLOADED_BYTES_PER_DISPATCH, 288);
}

#[test]
fn render_volumetric_light_scatter_sums_non_negative_authored_ambient_only_when_enabled() {
    let lights = [
        RenderAmbientLightSnapshot {
            color: Vec3::new(0.1, 0.2, 0.3),
            intensity: 0.5,
            renderer_degraded: false,
            degradation_reason: None,
        },
        RenderAmbientLightSnapshot {
            color: Vec3::new(0.4, -0.2, 0.1),
            intensity: 0.25,
            renderer_degraded: false,
            degradation_reason: None,
        },
    ];

    let ambient = volumetric_ambient_radiance(&lights, true);
    assert!((ambient - Vec3::new(0.15, 0.1, 0.175)).abs().max_element() < 0.000001);
    assert_eq!(volumetric_ambient_radiance(&lights, false), Vec3::ZERO);

    let overflowing = [
        RenderAmbientLightSnapshot {
            color: Vec3::splat(f32::MAX),
            intensity: 1.0,
            renderer_degraded: false,
            degradation_reason: None,
        },
        RenderAmbientLightSnapshot {
            color: Vec3::splat(f32::MAX),
            intensity: 1.0,
            renderer_degraded: false,
            degradation_reason: None,
        },
    ];
    assert_eq!(
        volumetric_ambient_radiance(&overflowing, true),
        Vec3::splat(f32::MAX)
    );
}
