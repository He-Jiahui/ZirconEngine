use super::*;

const VOLUMETRIC_APPLY_INCLUDE: &str =
    include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl");
const VOLUMETRIC_APPLY_TEST_SHADER: &str = concat!(
    r#"
struct SceneUniform {
    inverse_view_proj: mat4x4<f32>,
    camera_world_position: vec4<f32>,
    camera_view_direction: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
"#,
    include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl"),
    r#"
@group(0) @binding(1) var scene_color: texture_2d<f32>;
@group(0) @binding(2) var output_color: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let dimensions = textureDimensions(scene_color);
    if (any(invocation.xy >= dimensions)) {
        return;
    }
    let source = textureLoad(scene_color, vec2<i32>(invocation.xy), 0);
    let fragment_position = vec2<f32>(invocation.xy) + vec2<f32>(0.5);
    textureStore(output_color, invocation.xy, vec4<f32>(zr_volumetric_apply(source.rgb, fragment_position, 1.0), source.a));
}
"#,
);

#[test]
fn render_volumetric_integrate_upload_bytes_match_uniform_abi() {
    assert_eq!(FroxelIntegratePipeline::UPLOADED_BYTES_PER_DISPATCH, 128);
}

#[test]
fn render_volumetric_integrate_shader_writes_3d_radiance_transmittance_for_shading_apply() {
    assert!(INTEGRATE_SHADER.contains("zr_froxel_step_length"));
    assert!(INTEGRATE_SHADER.contains("exp(-extinction * step_length)"));
    assert!(INTEGRATE_SHADER.contains("(1.0 - step_transmittance) / extinction"));
    assert!(INTEGRATE_SHADER.contains("transmittance * max(sample.rgb"));
    assert!(INTEGRATE_SHADER.contains("texture_storage_3d<rgba16float, write>"));
    assert!(INTEGRATE_SHADER.contains("vec4<f32>(radiance, transmittance)"));
    assert!(!INTEGRATE_SHADER.contains("scene_color"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("@group(1) @binding(25)"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("@group(1) @binding(26)"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("@group(1) @binding(27)"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("fn zr_volumetric_apply("));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("color * zr_volumetric_transmittance("));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("+ zr_volumetric_scattering("));

    let module = naga::front::wgsl::parse_str(INTEGRATE_SHADER)
        .expect("volumetric integrate shader must parse");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("volumetric integrate shader must validate");

    let apply_module = naga::front::wgsl::parse_str(VOLUMETRIC_APPLY_TEST_SHADER)
        .expect("volumetric apply include must compose into a shader");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&apply_module)
    .expect("volumetric apply include must validate in a shading consumer");
}

mod fixture;
mod product;
mod support;
mod temporal_product;
