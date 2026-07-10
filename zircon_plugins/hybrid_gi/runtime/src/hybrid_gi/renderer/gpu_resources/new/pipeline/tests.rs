fn completion_shader_source() -> String {
    format!(
        "{}\n{}",
        include_str!("../../../shaders/update_completion.wgsl"),
        include_str!("../../../shaders/update_completion_scene_radiance.wgsl"),
    )
}

#[test]
fn completion_shader_does_not_relight_authoritative_scene_radiance() {
    let shader = completion_shader_source();

    assert!(
        shader.contains("fn scene_prepare_descriptor_has_authoritative_radiance"),
        "completion shader must distinguish captured Surface Cache/Voxel radiance from synthetic fallback colors"
    );
    assert!(
        shader.contains(
            "if (scene_prepare_descriptor_has_authoritative_radiance(descriptor)) {\n        return base_rgb;\n    }"
        ),
        "authoritative card and voxel radiance must bypass the scene-wide fallback light seed"
    );
}

#[test]
fn completion_shader_preserves_explicit_trace_region_lighting() {
    let shader = completion_shader_source();

    assert!(
        shader.contains(
            "if (region.rt_lighting_rgb != 0u) {\n        return unpack_rgb8(region.rt_lighting_rgb);\n    }"
        ),
        "explicit trace-region lighting already carries resolved radiance and must bypass the scene-wide fallback light seed"
    );
}
