struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var gbuffer_albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var normal_tex: texture_2d<f32>;
@group(1) @binding(2) var background_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = vec2<i32>(position.xy);
    let albedo = textureLoad(gbuffer_albedo_tex, coord, 0);
    let background = textureLoad(background_tex, coord, 0);
    if (albedo.a <= 0.001) {
        return background;
    }

    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));
    let light_dir = normalize(-scene.light_dir.xyz);
    let lambert = max(dot(light_dir, normal), 0.0);
    let lighting = scene.ambient_color.rgb + scene.light_color.rgb * lambert;
    let color = albedo.rgb * lighting;
    return vec4<f32>(color, albedo.a);
}
