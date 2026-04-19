struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let position = positions[vertex_index];
    var output: VertexOutput;
    output.clip_position = vec4<f32>(position, 0.0, 1.0);
    output.uv = vec2<f32>(position.x * 0.5 + 0.5, position.y * 0.5 + 0.5);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let horizon = vec3<f32>(0.16, 0.19, 0.24);
    let zenith = vec3<f32>(0.36, 0.46, 0.63);
    let t = clamp(input.uv.y, 0.0, 1.0);
    let color = mix(horizon, zenith, t);
    return vec4<f32>(color, 1.0);
}
