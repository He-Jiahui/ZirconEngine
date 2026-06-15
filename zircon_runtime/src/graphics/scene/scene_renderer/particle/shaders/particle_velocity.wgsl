struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;

struct VertexInput {
    @location(0) current_position: vec3<f32>,
    @location(1) previous_position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) velocity: vec2<f32>,
};

const EPSILON: f32 = 0.000001;

fn clip_to_uv(clip_position: vec4<f32>) -> vec2<f32> {
    if (abs(clip_position.w) <= EPSILON) {
        return vec2<f32>(0.5, 0.5);
    }
    let ndc = clip_position.xy / clip_position.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let current_clip = scene.view_proj_unjittered * vec4<f32>(input.current_position, 1.0);
    let previous_clip = scene.previous_view_proj_unjittered * vec4<f32>(input.previous_position, 1.0);

    var output: VertexOutput;
    output.clip_position = current_clip;
    output.velocity = clamp(
        clip_to_uv(current_clip) - clip_to_uv(previous_clip),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec2<f32> {
    return input.velocity;
}
