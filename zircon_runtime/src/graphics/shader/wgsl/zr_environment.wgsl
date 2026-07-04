const ZR_ENVIRONMENT_EPSILON: f32 = 0.000001;
const ZR_ENVIRONMENT_INV_PI: f32 = 0.3183098861837907;
const ZR_ENVIRONMENT_INV_TAU: f32 = 0.15915494309189535;
const ZR_ENVIRONMENT_SAMPLED_EQUIRECT_KIND: f32 = 2.0;
const ZR_ENVIRONMENT_ROUGHEST_MIP: f32 = 1.0;
const ZR_ENVIRONMENT_ROUGHNESS_MIP_SCALE: f32 = 1.2;

struct ZrEnvironmentSampleBuffer {
    samples: array<vec4<f32>>,
};
@group(0) @binding(1) var<storage, read> zr_environment_samples: ZrEnvironmentSampleBuffer;

fn zr_environment_normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= ZR_ENVIRONMENT_EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return value / value_length;
}

fn zr_environment_is_enabled() -> bool {
    return scene.environment_params.w > 0.5;
}

fn zr_environment_rotated_direction(direction: vec3<f32>) -> vec3<f32> {
    let rotation = scene.environment_params.z;
    let s = sin(rotation);
    let c = cos(rotation);
    return vec3<f32>(
        direction.x * c - direction.z * s,
        direction.y,
        direction.x * s + direction.z * c,
    );
}

fn zr_environment_mip_dimensions(mip_level: u32) -> vec2<u32> {
    var width = max(u32(scene.environment_sample_params.y), 1u);
    var height = max(u32(scene.environment_sample_params.z), 1u);
    for (var mip = 0u; mip < mip_level; mip = mip + 1u) {
        width = max(width / 2u, 1u);
        height = max(height / 2u, 1u);
    }
    return vec2<u32>(width, height);
}

fn zr_environment_mip_offset(mip_level: u32) -> u32 {
    var width = max(u32(scene.environment_sample_params.y), 1u);
    var height = max(u32(scene.environment_sample_params.z), 1u);
    var offset = 0u;
    for (var mip = 0u; mip < mip_level; mip = mip + 1u) {
        offset = offset + width * height;
        width = max(width / 2u, 1u);
        height = max(height / 2u, 1u);
    }
    return offset;
}

fn zr_environment_sampled_equirect_texel(mip_level: u32, x: u32, y: u32) -> vec3<f32> {
    let dims = zr_environment_mip_dimensions(mip_level);
    let wrapped_x = x % dims.x;
    let clamped_y = min(y, dims.y - 1u);
    let index = zr_environment_mip_offset(mip_level) + clamped_y * dims.x + wrapped_x;
    return zr_environment_samples.samples[index].rgb;
}

fn zr_environment_sampled_equirect_mip_color(direction: vec3<f32>, mip_level: u32) -> vec3<f32> {
    let dims = zr_environment_mip_dimensions(mip_level);
    let u = fract(atan2(direction.z, direction.x) * ZR_ENVIRONMENT_INV_TAU + 0.5);
    let v = clamp(
        acos(clamp(direction.y, -1.0, 1.0)) * ZR_ENVIRONMENT_INV_PI,
        0.0,
        1.0,
    );
    let texel_x = u * f32(dims.x) - 0.5;
    let texel_y = v * f32(dims.y) - 0.5;
    let x0 = i32(floor(texel_x));
    let y0 = i32(floor(texel_y));
    let tx = fract(texel_x);
    let ty = fract(texel_y);
    let x0u = u32((x0 % i32(dims.x) + i32(dims.x)) % i32(dims.x));
    let x1u = (x0u + 1u) % dims.x;
    let y0u = u32(clamp(f32(y0), 0.0, f32(dims.y - 1u)));
    let y1u = min(y0u + 1u, dims.y - 1u);
    let c00 = zr_environment_sampled_equirect_texel(mip_level, x0u, y0u);
    let c10 = zr_environment_sampled_equirect_texel(mip_level, x1u, y0u);
    let c01 = zr_environment_sampled_equirect_texel(mip_level, x0u, y1u);
    let c11 = zr_environment_sampled_equirect_texel(mip_level, x1u, y1u);
    return mix(mix(c00, c10, tx), mix(c01, c11, tx), ty);
}

fn zr_environment_sampled_equirect_color_at_lod(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    let mip_count = max(scene.environment_sample_params.w, 1.0);
    let max_mip = mip_count - 1.0;
    let clamped_lod = clamp(lod, 0.0, max_mip);
    let mip0 = u32(floor(clamped_lod));
    let mip1 = min(mip0 + 1u, u32(max_mip));
    let blend = fract(clamped_lod);
    let normalized_direction = zr_environment_rotated_direction(
        zr_environment_normalize_or_zero(direction),
    );
    let c0 = zr_environment_sampled_equirect_mip_color(normalized_direction, mip0);
    let c1 = zr_environment_sampled_equirect_mip_color(normalized_direction, mip1);
    return mix(c0, c1, blend) * max(scene.environment_params.y, 0.0);
}

fn zr_environment_mip_from_roughness(roughness: f32, max_mip: f32) -> f32 {
    let level_from_one_by_one =
        ZR_ENVIRONMENT_ROUGHEST_MIP
        - ZR_ENVIRONMENT_ROUGHNESS_MIP_SCALE * log2(max(roughness, 0.001));
    return clamp(max_mip - 1.0 - level_from_one_by_one, 0.0, max_mip);
}

fn zr_environment_env_brdf_approx(f0: vec3<f32>, roughness: f32, no_v: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * no_v)) * r.x + r.y;
    let ab = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return f0 * ab.x + vec3<f32>(ab.y);
}

fn zr_environment_sky_color(direction: vec3<f32>) -> vec3<f32> {
    if (scene.environment_sample_params.x >= ZR_ENVIRONMENT_SAMPLED_EQUIRECT_KIND - 0.5) {
        return zr_environment_sampled_equirect_color_at_lod(direction, 0.0);
    }
    let normalized_direction = zr_environment_normalize_or_zero(direction);
    let sky_t = clamp(normalized_direction.y * 0.5 + 0.5, 0.0, 1.0);
    let ground_t = clamp(normalized_direction.y + 1.0, 0.0, 1.0);
    let sky = mix(scene.sky_horizon_color.rgb, scene.sky_zenith_color.rgb, sky_t);
    let ground = mix(scene.sky_ground_color.rgb, scene.sky_horizon_color.rgb, ground_t);
    return select(ground, sky, normalized_direction.y >= 0.0) * max(scene.environment_params.y, 0.0);
}

fn zr_environment_reflection_color(
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
    let normal = zr_environment_normalize_or_zero(normal_ws);
    let view_dir = zr_environment_normalize_or_zero(view_dir_ws);
    let reflected = reflect(-view_dir, normal);
    if (scene.environment_sample_params.x >= ZR_ENVIRONMENT_SAMPLED_EQUIRECT_KIND - 0.5) {
        let max_mip = max(scene.environment_sample_params.w - 1.0, 0.0);
        let lod = zr_environment_mip_from_roughness(clamp(roughness, 0.0, 1.0), max_mip);
        return zr_environment_sampled_equirect_color_at_lod(reflected, lod);
    }
    let sharp_reflection = zr_environment_sky_color(reflected);
    let rough_reflection = zr_environment_sky_color(normal);
    return mix(sharp_reflection, rough_reflection, clamp(roughness, 0.0, 1.0));
}

fn zr_environment_diffuse_color(normal_ws: vec3<f32>) -> vec3<f32> {
    if (scene.environment_sample_params.x >= ZR_ENVIRONMENT_SAMPLED_EQUIRECT_KIND - 0.5) {
        let rough_diffuse_mip = max(scene.environment_sample_params.w - 2.0, 0.0);
        return zr_environment_sampled_equirect_color_at_lod(normal_ws, rough_diffuse_mip);
    }
    return zr_environment_sky_color(normal_ws);
}

fn zr_environment_pbr_indirect(
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> vec3<f32> {
    if (!zr_environment_is_enabled() || !is_standard_pbr) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let normal = zr_environment_normalize_or_zero(normal_ws);
    let view_dir = zr_environment_normalize_or_zero(view_dir_ws);
    let clamped_metallic = clamp(metallic, 0.0, 1.0);
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    let clamped_occlusion = clamp(occlusion, 0.0, 1.0);
    let f0 = mix(
        vec3<f32>(0.04, 0.04, 0.04),
        max(base_color, vec3<f32>(0.0, 0.0, 0.0)),
        clamped_metallic,
    );
    let no_v = clamp(dot(normal, view_dir), 0.0, 1.0);
    let diffuse_environment =
        zr_environment_diffuse_color(normal) * diffuse_color * (1.0 - clamped_metallic);
    let reflection = zr_environment_reflection_color(normal, view_dir, clamped_roughness);
    let specular_environment =
        reflection * zr_environment_env_brdf_approx(f0, clamped_roughness, no_v);
    return (diffuse_environment + specular_environment) * clamped_occlusion;
}
