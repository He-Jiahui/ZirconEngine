use super::shader_source::DEFERRED_LIGHTING_SHADER;

#[test]
fn deferred_lighting_shader_matches_scene_uniform_layout() {
    let scene_uniform = DEFERRED_LIGHTING_SHADER
        .split("struct SceneUniform {")
        .nth(1)
        .and_then(|tail| tail.split("};").next())
        .expect("deferred lighting shader should declare SceneUniform");

    let view_proj = scene_uniform.find("view_proj").unwrap();
    let inverse_view_proj = scene_uniform.find("inverse_view_proj").unwrap();
    let light_dir = scene_uniform.find("light_dir").unwrap();

    assert!(
        view_proj < inverse_view_proj && inverse_view_proj < light_dir,
        "deferred lighting shader must match the Rust SceneUniform matrix layout before light fields"
    );
}

#[test]
fn deferred_lighting_shader_receives_shadow_map_resources() {
    for expected in [
        "@group(1) @binding(4) var scene_depth_tex: texture_depth_2d;",
        "@group(1) @binding(5) var shadow_map_tex: texture_depth_2d;",
        "@group(1) @binding(6) var<uniform> shadow_receiver: ShadowReceiverUniform;",
        "@group(1) @binding(7) var shadow_compare_sampler: sampler_comparison;",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "missing deferred shadow receiver binding `{expected}`"
        );
    }

    for expected in [
        "reconstruct_world_position(coord, depth)",
        "textureSampleCompare(shadow_map_tex, shadow_compare_sampler",
        "sample_shadow_visibility",
        "shadow_receiver.params.y",
        "shadow_receiver.params.z",
        "direct_visibility",
    ] {
        assert!(
            DEFERRED_LIGHTING_SHADER.contains(expected),
            "deferred lighting shader should use `{expected}` for shadow receiving"
        );
    }
    assert!(
        !DEFERRED_LIGHTING_SHADER.contains("textureLoad(shadow_map_tex"),
        "deferred shadow receiving should use the hardware comparison sampler instead of raw depth loads"
    );
}
