const ENVIRONMENT_SHADER: &str = include_str!("../src/graphics/shader/wgsl/zr_environment.wgsl");
const SKYBOX_SHADER: &str =
    include_str!("../src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing {signature}"));
    let function = &source[start..];
    let body_start = function
        .find('{')
        .unwrap_or_else(|| panic!("missing opening brace for {signature}"));
    let body = &function[body_start + 1..];
    let body_end = body
        .find('}')
        .unwrap_or_else(|| panic!("missing closing brace for {signature}"));
    &body[..body_end]
}

#[test]
fn runtime_environment_wgpu_cubemap_sampling_does_not_warp_lookup_directions() {
    for (label, source, signature) in [
        (
            "environment",
            ENVIRONMENT_SHADER,
            "fn zr_environment_fix_cube_lookup_for_face_size",
        ),
        ("skybox", SKYBOX_SHADER, "fn skybox_fix_cube_lookup"),
    ] {
        let body = function_body(source, signature);
        assert!(
            body.contains("return direction;"),
            "{label} must preserve the cube lookup direction"
        );
        assert!(
            !body.contains("adjusted"),
            "{label} must not retain legacy edge-warp state"
        );
        assert!(
            !body.contains("exp2("),
            "{label} must not retain legacy LOD edge warping"
        );
    }
}
