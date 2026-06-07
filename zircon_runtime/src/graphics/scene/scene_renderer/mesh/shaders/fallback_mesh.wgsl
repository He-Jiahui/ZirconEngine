struct SceneUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj: mat4x4<f32>,
    motion_params: vec4<f32>,
};
struct ShadowReceiverUniform {
    light_view_proj: mat4x4<f32>,
    params: vec4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
    shadow_params: vec4<f32>,
    previous_model: mat4x4<f32>,
    motion_params: vec4<f32>,
};
struct MaterialPropertyUniform {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(3) @binding(0) var<uniform> material_properties: MaterialPropertyUniform;
@group(4) @binding(0) var shadow_map_tex: texture_depth_2d;
@group(4) @binding(1) var<uniform> shadow_receiver: ShadowReceiverUniform;
@group(4) @binding(2) var shadow_compare_sampler: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
};

struct MotionVectorVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) current_clip_position: vec4<f32>,
    @location(2) previous_clip_position: vec4<f32>,
};

const EPSILON: f32 = 0.000001;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.world_normal = normalize((model_data.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.world_position = world.xyz;
    return output;
}

fn world_to_shadow_coord(world_position: vec3<f32>) -> vec4<f32> {
    let light_clip = shadow_receiver.light_view_proj * vec4<f32>(world_position, 1.0);
    if (abs(light_clip.w) <= EPSILON) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let light_ndc = light_clip.xyz / light_clip.w;
    if (any(light_ndc.xy < vec2<f32>(-1.0, -1.0)) || light_ndc.z < 0.0 || any(light_ndc > vec3<f32>(1.0, 1.0, 1.0))) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let shadow_uv = light_ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return vec4<f32>(shadow_uv, light_ndc.z, 1.0);
}

fn sample_shadow_visibility(shadow_uv: vec2<f32>, receiver_depth: f32, offset: vec2<i32>) -> f32 {
    let shadow_size = max(textureDimensions(shadow_map_tex), vec2<u32>(1u, 1u));
    let shadow_texel = vec2<f32>(1.0, 1.0) / vec2<f32>(shadow_size);
    let sample_uv = clamp(shadow_uv + vec2<f32>(offset) * shadow_texel, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    return textureSampleCompare(shadow_map_tex, shadow_compare_sampler, sample_uv, receiver_depth);
}

fn shadow_visibility(world_position: vec3<f32>) -> f32 {
    if (shadow_receiver.params.x <= 0.5) {
        return 1.0;
    }
    if (model_data.shadow_params.z <= 0.5) {
        return 1.0;
    }

    let shadow_coord = world_to_shadow_coord(world_position);
    if (shadow_coord.w <= 0.0) {
        return 1.0;
    }

    let receiver_depth = clamp(shadow_coord.z - shadow_receiver.params.y, 0.0, 1.0);
    let offsets = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1),
        vec2<i32>(0, -1),
        vec2<i32>(1, -1),
        vec2<i32>(-1, 0),
        vec2<i32>(0, 0),
        vec2<i32>(1, 0),
        vec2<i32>(-1, 1),
        vec2<i32>(0, 1),
        vec2<i32>(1, 1),
    );
    var lit = 0.0;
    for (var i = 0u; i < 9u; i = i + 1u) {
        lit = lit + sample_shadow_visibility(shadow_coord.xy, receiver_depth, offsets[i]);
    }

    return mix(clamp(shadow_receiver.params.z, 0.0, 1.0), 1.0, lit / 9.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(-scene.light_dir.xyz);
    let lambert = max(dot(light_dir, normalize(input.world_normal)), 0.0);
    let direct_visibility = shadow_visibility(input.world_position);
    let lighting = scene.ambient_color.rgb + scene.light_color.rgb * lambert * direct_visibility;
    let albedo = textureSample(albedo_tex, albedo_sampler, input.uv).rgba * model_data.tint;
    return vec4<f32>(albedo.rgb * lighting, albedo.a);
}

@vertex
fn vs_motion_vector(input: VertexInput) -> MotionVectorVertexOutput {
    var output: MotionVectorVertexOutput;
    let current_world = model_data.model * vec4<f32>(input.position, 1.0);
    let previous_world = model_data.previous_model * vec4<f32>(input.position, 1.0);
    let current_clip = scene.view_proj * current_world;
    let previous_clip = scene.previous_view_proj * previous_world;
    output.clip_position = current_clip;
    output.uv = input.uv;
    output.current_clip_position = current_clip;
    output.previous_clip_position = previous_clip;
    return output;
}

fn clip_to_motion_uv(clip_position: vec4<f32>) -> vec2<f32> {
    if (abs(clip_position.w) <= EPSILON) {
        return vec2<f32>(0.5, 0.5);
    }
    let ndc = clip_position.xy / clip_position.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@fragment
fn fs_motion_vector(input: MotionVectorVertexOutput) -> @location(0) vec4<f32> {
    if (scene.motion_params.x <= 0.5 || model_data.motion_params.x <= 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    if (model_data.shadow_params.x > 0.5) {
        let albedo = textureSample(albedo_tex, albedo_sampler, input.uv).rgba * model_data.tint;
        if (albedo.a < model_data.shadow_params.y) {
            discard;
        }
    }

    let current_uv = clip_to_motion_uv(input.current_clip_position);
    let previous_uv = clip_to_motion_uv(input.previous_clip_position);
    let velocity = clamp(current_uv - previous_uv, vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0));
    return vec4<f32>(velocity, 0.0, 1.0);
}
