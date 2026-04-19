pub(in crate::asset::pipeline::manager) fn builtin_pbr_wgsl() -> &'static str {
    r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.position = scene.view_proj * world_position;
    out.world_normal = normalize((model.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(color_texture, color_sampler, input.uv) * model.tint;
    let ndotl = max(dot(normalize(input.world_normal), normalize(-scene.light_dir.xyz)), 0.0);
    let lighting = 0.15 + ndotl;
    return vec4<f32>(albedo.rgb * scene.light_color.rgb * lighting, albedo.a);
}
"#
}
