const ZR_VELOCITY_EPSILON: f32 = 0.000001;

struct ZrVelocityVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) joints: vec4<u32>,
    @location(4) weights: vec4<f32>,
    @location(5) tangent: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) uv1: vec2<f32>,
    @location(8) previous_position: vec3<f32>,
    @builtin(vertex_index) vertex_index: u32,
};

struct ZrVelocityVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position_ws: vec3<f32>,
    @location(1) normal_ws: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) tangent_ws: vec3<f32>,
    @location(5) tangent_handedness: f32,
    @location(6) color: vec4<f32>,
    @location(7) tint: vec4<f32>,
    @location(8) shadow_params: vec4<f32>,
    @location(9) motion_params: vec4<f32>,
    @location(10) current_clip_position: vec4<f32>,
    @location(11) previous_clip_position: vec4<f32>,
};

fn zr_velocity_vertex_input(v: ZrVelocityVertexInput, position: vec3<f32>) -> ZrVertexInput {
    var input: ZrVertexInput;
    input.position = position;
    input.normal = v.normal;
    input.uv0 = v.uv0;
    input.joints = v.joints;
    input.weights = v.weights;
    input.tangent = v.tangent;
    input.color = v.color;
    input.uv1 = v.uv1;
    input.vertex_index = v.vertex_index;
    return input;
}

fn zr_velocity_output(
    material_output: ZrVertexOutput,
    motion_params: vec4<f32>,
    current_clip: vec4<f32>,
    previous_clip: vec4<f32>,
) -> ZrVelocityVertexOutput {
    var output: ZrVelocityVertexOutput;
    output.clip_position = current_clip;
    output.position_ws = material_output.position_ws;
    output.normal_ws = material_output.normal_ws;
    output.uv0 = material_output.uv0;
    output.uv1 = material_output.uv1;
    output.tangent_ws = material_output.tangent_ws;
    output.tangent_handedness = material_output.tangent_handedness;
    output.color = material_output.color;
    output.tint = material_output.tint;
    output.shadow_params = material_output.shadow_params;
    output.motion_params = motion_params;
    output.current_clip_position = current_clip;
    output.previous_clip_position = previous_clip;
    return output;
}

fn zr_vs_main_impl(v: ZrVelocityVertexInput, instance_index: u32) -> ZrVelocityVertexOutput {
    let current_input = zr_velocity_vertex_input(v, v.position);
    let previous_input = zr_velocity_vertex_input(v, v.previous_position);
    let current_position = fetch_position(current_input, instance_index);
    let previous_position = fetch_prev_position(previous_input, instance_index);
    let current_world = zr_world_from_local(instance_index) * vec4<f32>(current_position, 1.0);
    let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_position, 1.0);
    let current_clip = scene.view_proj_unjittered * current_world;
    let previous_clip = scene.previous_view_proj_unjittered * previous_world;
    let material_output = zr_build_vertex_output(
        instance_index,
        current_position,
        fetch_normal(current_input, instance_index),
        fetch_tangent(current_input, instance_index),
        fetch_uv0(current_input),
        fetch_uv1(current_input),
        fetch_color(current_input, instance_index),
    );
    return zr_velocity_output(material_output, zr_gpu_scene_motion_params(instance_index), current_clip, previous_clip);
}

@vertex
fn zr_vs_main(v: ZrVelocityVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVelocityVertexOutput {
    return zr_vs_main_impl(v, instance_index);
}

@vertex
fn vs_main(v: ZrVelocityVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVelocityVertexOutput {
    return zr_vs_main_impl(v, instance_index);
}

fn zr_velocity_clip_to_uv(clip_position: vec4<f32>) -> vec2<f32> {
    if (abs(clip_position.w) <= ZR_VELOCITY_EPSILON) {
        return vec2<f32>(0.5, 0.5);
    }
    let ndc = clip_position.xy / clip_position.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

fn zr_velocity_apply_alpha_clip(input: ZrVelocityVertexOutput) {
    _ = input;
}

fn zr_fs_main_impl(input: ZrVelocityVertexOutput) -> vec4<f32> {
    if (scene.motion_params.x <= 0.5 || input.motion_params.x <= 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    zr_velocity_apply_alpha_clip(input);

    let current_uv = zr_velocity_clip_to_uv(input.current_clip_position);
    let previous_uv = zr_velocity_clip_to_uv(input.previous_clip_position);
    let velocity = clamp(current_uv - previous_uv, vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0));
    return vec4<f32>(velocity, 0.0, 1.0);
}

@fragment
fn zr_fs_main(input: ZrVelocityVertexOutput) -> @location(0) vec4<f32> {
    return zr_fs_main_impl(input);
}

@fragment
fn fs_main(input: ZrVelocityVertexOutput) -> @location(0) vec4<f32> {
    return zr_fs_main_impl(input);
}
