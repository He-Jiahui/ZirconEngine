struct VelocityCameraParams {
    viewport_and_flags: vec4<u32>,
    current_clip_from_world: mat4x4<f32>,
    current_world_from_clip: mat4x4<f32>,
    previous_clip_from_world: mat4x4<f32>,
};

@group(0) @binding(0) var scene_depth_tex: texture_depth_2d;
@group(0) @binding(1) var<uniform> params: VelocityCameraParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

const EPSILON: f32 = 0.000001;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn load_velocity_scene_depth(coord: vec2<u32>) -> f32 {
    let viewport_size = max(params.viewport_and_flags.xy, vec2<u32>(1u, 1u));
    let clamped = min(coord, viewport_size - vec2<u32>(1u, 1u));
    return clamp(textureLoad(scene_depth_tex, clamped, 0), 0.0, 1.0);
}

fn coord_to_screen_uv(coord: vec2<u32>, viewport_size: vec2<u32>) -> vec2<f32> {
    return (vec2<f32>(coord) + vec2<f32>(0.5, 0.5)) / vec2<f32>(viewport_size);
}

fn screen_uv_to_clip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

fn clip_to_uv(clip_position: vec4<f32>) -> vec2<f32> {
    if (abs(clip_position.w) <= EPSILON) {
        return vec2<f32>(0.5, 0.5);
    }
    let ndc = clip_position.xy / clip_position.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

fn velocity_camera_velocity(coord: vec2<u32>, depth: f32) -> vec2<f32> {
    if (params.viewport_and_flags.z == 0u) {
        return vec2<f32>(0.0, 0.0);
    }

    let viewport_size = max(params.viewport_and_flags.xy, vec2<u32>(1u, 1u));
    let current_uv = coord_to_screen_uv(coord, viewport_size);
    let current_clip = screen_uv_to_clip(current_uv, depth);
    let current_world = params.current_world_from_clip * current_clip;
    if (abs(current_world.w) <= EPSILON) {
        return vec2<f32>(0.0, 0.0);
    }

    let world_position = vec4<f32>(current_world.xyz / current_world.w, 1.0);
    let previous_uv = clip_to_uv(params.previous_clip_from_world * world_position);
    return clamp(current_uv - previous_uv, vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = max(params.viewport_and_flags.xy, vec2<u32>(1u, 1u));
    let coord = min(vec2<u32>(position.xy), viewport_size - vec2<u32>(1u, 1u));
    let depth = load_velocity_scene_depth(coord);
    let velocity = velocity_camera_velocity(coord, depth);
    return vec4<f32>(velocity, 0.0, 1.0);
}
