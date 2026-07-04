struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
    sky_horizon_color: vec4<f32>,
    sky_zenith_color: vec4<f32>,
    sky_ground_color: vec4<f32>,
    environment_params: vec4<f32>,
    environment_sample_params: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;

struct EnvironmentSampleBuffer {
    samples: array<vec4<f32>>,
};
@group(0) @binding(1) var<storage, read> environment_samples: EnvironmentSampleBuffer;

const SKYBOX_INV_PI: f32 = 0.3183098861837907;
const SKYBOX_INV_TAU: f32 = 0.15915494309189535;
const SKYBOX_SAMPLED_EQUIRECT_KIND: f32 = 2.0;

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

fn sampled_equirect_texel(mip_level: u32, x: u32, y: u32) -> vec3<f32> {
    var width = max(u32(scene.environment_sample_params.y), 1u);
    var height = max(u32(scene.environment_sample_params.z), 1u);
    var offset = 0u;
    for (var mip = 0u; mip < mip_level; mip = mip + 1u) {
        offset = offset + width * height;
        width = max(width / 2u, 1u);
        height = max(height / 2u, 1u);
    }
    let wrapped_x = x % width;
    let clamped_y = min(y, height - 1u);
    return environment_samples.samples[offset + clamped_y * width + wrapped_x].rgb;
}

fn sampled_equirect_mip_color(direction: vec3<f32>, mip_level: u32) -> vec3<f32> {
    var width = max(u32(scene.environment_sample_params.y), 1u);
    var height = max(u32(scene.environment_sample_params.z), 1u);
    for (var mip = 0u; mip < mip_level; mip = mip + 1u) {
        width = max(width / 2u, 1u);
        height = max(height / 2u, 1u);
    }
    let u = fract(atan2(direction.z, direction.x) * SKYBOX_INV_TAU + 0.5);
    let v = clamp(acos(clamp(direction.y, -1.0, 1.0)) * SKYBOX_INV_PI, 0.0, 1.0);
    let texel_x = u * f32(width) - 0.5;
    let texel_y = v * f32(height) - 0.5;
    let x0 = i32(floor(texel_x));
    let y0 = i32(floor(texel_y));
    let tx = fract(texel_x);
    let ty = fract(texel_y);
    let x0u = u32((x0 % i32(width) + i32(width)) % i32(width));
    let x1u = (x0u + 1u) % width;
    let y0u = u32(clamp(f32(y0), 0.0, f32(height - 1u)));
    let y1u = min(y0u + 1u, height - 1u);
    let c00 = sampled_equirect_texel(mip_level, x0u, y0u);
    let c10 = sampled_equirect_texel(mip_level, x1u, y0u);
    let c01 = sampled_equirect_texel(mip_level, x0u, y1u);
    let c11 = sampled_equirect_texel(mip_level, x1u, y1u);
    return mix(mix(c00, c10, tx), mix(c01, c11, tx), ty);
}

fn sampled_equirect_color(direction: vec3<f32>) -> vec3<f32> {
    let rotation = scene.environment_params.z;
    let s = sin(rotation);
    let c = cos(rotation);
    let rotated = normalize(vec3<f32>(
        direction.x * c - direction.z * s,
        direction.y,
        direction.x * s + direction.z * c,
    ));
    return sampled_equirect_mip_color(rotated, 0u) * max(scene.environment_params.y, 0.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (scene.environment_sample_params.x >= SKYBOX_SAMPLED_EQUIRECT_KIND - 0.5) {
        let ndc = input.uv * 2.0 - vec2<f32>(1.0, 1.0);
        let direction = normalize(vec3<f32>(ndc.x, ndc.y, 1.0));
        return vec4<f32>(sampled_equirect_color(direction), 1.0);
    }
    let t = clamp(input.uv.y, 0.0, 1.0);
    let intensity = max(scene.environment_params.y, 0.0);
    let color = mix(scene.sky_horizon_color.rgb, scene.sky_zenith_color.rgb, t) * intensity;
    return vec4<f32>(color, 1.0);
}
