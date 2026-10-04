// Shader 仅覆盖调用方设置的相机 ViewRect；外部区域和未请求清除的 attachment 保留原值。
pub(super) const SCENE_REGION_CLEAR_SHADER: &str = r#"
struct SceneClearColor {
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> scene_clear: SceneClearColor;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 1.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return scene_clear.color;
}
"#;
