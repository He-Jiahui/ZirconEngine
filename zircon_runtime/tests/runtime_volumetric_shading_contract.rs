const VOLUMETRIC: &str = include_str!("../src/graphics/shader/wgsl/zr_volumetric.wgsl");
const FORWARD: &str = concat!(
    include_str!("../src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_environment.wgsl"),
);
const DEFERRED: &str = concat!(
    include_str!("../src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_shade_deferred_standard_pbr.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_shade_deferred_blinn_phong.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_shade_deferred_unlit.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_environment.wgsl"),
);
const SKY: &str = concat!(
    include_str!("../src/graphics/shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl"),
);
const LIGHT_SCATTER: &str = concat!(
    include_str!(
        "../src/graphics/scene/scene_renderer/advanced_lighting/froxel/shaders/zr_froxel_reconstruct.wgsl"
    ),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/types.wgsl"
    ),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/main.wgsl"
    ),
);

#[test]
fn runtime_volumetric_shading_contract_uses_fixed_group_one_bindings() {
    for expected in [
        "@group(1) @binding(25) var<uniform> zr_volumetric_apply_params",
        "@group(1) @binding(26) var zr_volumetric_integrated: texture_3d<f32>;",
        "@group(1) @binding(27) var zr_volumetric_sampler: sampler;",
        "color * clamp(integrated.a, 0.0, 1.0) + max(integrated.rgb",
    ] {
        assert!(
            VOLUMETRIC.contains(expected),
            "volumetric include should contain `{expected}`"
        );
    }
}

#[test]
fn runtime_volumetric_shading_contract_validates_forward_deferred_and_sky_wgsl() {
    for (label, source) in [
        ("forward", FORWARD),
        ("deferred", DEFERRED),
        ("sky", SKY),
        ("light-scatter-temporal", LIGHT_SCATTER),
    ] {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{label}: {}", error.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
    }
}

#[test]
fn runtime_volumetric_light_scatter_reprojects_and_rejects_history() {
    for expected in [
        "previous_clip_from_world",
        "previous_froxel_scattering: texture_3d<f32>",
        "zr_froxel_world_position_jittered",
        "previous_view_depth",
        "extinction_threshold",
        "mix(current.rgb, history.rgb, history_weight)",
    ] {
        assert!(
            LIGHT_SCATTER.contains(expected),
            "temporal light scatter should contain `{expected}`"
        );
    }
}

#[test]
fn runtime_volumetric_shading_contract_applies_at_all_scene_output_endpoints() {
    assert!(FORWARD
        .contains("zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z)"));
    assert!(DEFERRED.contains("zr_volumetric_apply(shaded.rgb, position.xy, depth)"));
    assert!(SKY.contains("zr_volumetric_apply(color, input.clip_position.xy, 1.0)"));

    let forward_template = include_str!("../src/graphics/shader/wgsl/zr_template_forward.wgsl");
    let module_registry = include_str!("../src/graphics/shader/template/module_registry.rs");
    assert!(forward_template
        .contains("zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z)"));
    assert!(module_registry.contains("zr_volumetric.wgsl"));
}
