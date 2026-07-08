const ZR_ENVIRONMENT_EPSILON: f32 = 0.000001;
const ZR_ENVIRONMENT_SOURCE_CUBEMAP_KIND: f32 = 3.0;
override ZR_ENV_DIFFUSE_IEM: bool = false;

@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;
@group(0) @binding(2) var zr_environment_sampler: sampler;
@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;
@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;
@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;

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

fn zr_environment_is_source_cubemap() -> bool {
    return scene.environment_sample_params.x >= ZR_ENVIRONMENT_SOURCE_CUBEMAP_KIND - 0.5;
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

fn zr_environment_fix_cube_lookup(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    var adjusted = direction;
    let face_size = max(scene.environment_sample_params.y, 1.0);
    let scale = clamp(1.0 - exp2(max(lod, 0.0)) / face_size, 0.0, 1.0);
    let axis = abs(adjusted);
    if (axis.x > axis.y && axis.x > axis.z) {
        adjusted = vec3<f32>(adjusted.x, adjusted.y * scale, adjusted.z * scale);
    } else if (axis.y > axis.z) {
        adjusted = vec3<f32>(adjusted.x * scale, adjusted.y, adjusted.z * scale);
    } else {
        adjusted = vec3<f32>(adjusted.x * scale, adjusted.y * scale, adjusted.z);
    }
    return adjusted;
}

fn zr_environment_source_cube_color_at_lod(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    let mip_count = max(scene.environment_sample_params.w, 1.0);
    let max_mip = mip_count - 1.0;
    let clamped_lod = clamp(lod, 0.0, max_mip);
    let rotated = zr_environment_rotated_direction(zr_environment_normalize_or_zero(direction));
    return textureSampleLevel(
        zr_environment_source_cube,
        zr_environment_sampler,
        zr_environment_fix_cube_lookup(rotated, clamped_lod),
        clamped_lod,
    ).rgb * max(scene.environment_params.y, 0.0);
}

fn zr_environment_specular_pmrem_color_at_lod(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    let mip_count = max(scene.environment_sample_params.w, 1.0);
    let max_mip = mip_count - 1.0;
    let clamped_lod = clamp(lod, 0.0, max_mip);
    let rotated = zr_environment_rotated_direction(zr_environment_normalize_or_zero(direction));
    return textureSampleLevel(
        zr_environment_specular_pmrem_cube,
        zr_environment_sampler,
        zr_environment_fix_cube_lookup(rotated, clamped_lod),
        clamped_lod,
    ).rgb * max(scene.environment_params.y, 0.0);
}

fn zr_environment_mip_from_roughness(roughness: f32, max_mip: f32) -> f32 {
    return clamp(roughness, 0.0, 1.0) * max(max_mip, 0.0);
}

fn zr_environment_env_brdf_approx(f0: vec3<f32>, roughness: f32, no_v: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * no_v)) * r.x + r.y;
    let ab = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return clamp(f0 * ab.x + vec3<f32>(ab.y), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn zr_environment_env_brdf_lut(f0: vec3<f32>, roughness: f32, no_v: f32) -> vec3<f32> {
    let uv = vec2<f32>(clamp(no_v, 0.0, 1.0), clamp(roughness, 0.0, 1.0));
    let ab = textureSampleLevel(
        zr_environment_brdf_lut,
        zr_environment_sampler,
        uv,
        0.0,
    ).rg;
    let f90 = clamp(50.0 * f0.g, 0.0, 1.0);
    return clamp(f0 * ab.x + vec3<f32>(f90) * ab.y, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn zr_environment_sh9_eval(normal_ws: vec3<f32>) -> vec3<f32> {
    let n = zr_environment_normalize_or_zero(normal_ws);
    let x = n.x;
    let y = n.y;
    let z = n.z;
    var irradiance = scene.environment_sh9[0].rgb * 0.2820948;
    irradiance += scene.environment_sh9[1].rgb * (0.48860252 * z);
    irradiance += scene.environment_sh9[2].rgb * (0.48860252 * y);
    irradiance += scene.environment_sh9[3].rgb * (0.48860252 * x);
    irradiance += scene.environment_sh9[4].rgb * (1.0925485 * x * z);
    irradiance += scene.environment_sh9[5].rgb * (1.0925485 * z * y);
    irradiance += scene.environment_sh9[6].rgb * (0.31539157 * (3.0 * y * y - 1.0));
    irradiance += scene.environment_sh9[7].rgb * (1.0925485 * x * y);
    irradiance += scene.environment_sh9[8].rgb * (0.54627424 * (x * x - z * z));
    return max(irradiance, vec3<f32>(0.0, 0.0, 0.0));
}

fn zr_environment_irradiance_cube_color(normal_ws: vec3<f32>) -> vec3<f32> {
    let rotated = zr_environment_rotated_direction(zr_environment_normalize_or_zero(normal_ws));
    return textureSample(
        zr_environment_irradiance_cube,
        zr_environment_sampler,
        zr_environment_fix_cube_lookup(rotated, 0.0),
    ).rgb
        * max(scene.environment_params.y, 0.0);
}

fn zr_environment_sky_color(direction: vec3<f32>) -> vec3<f32> {
    if (zr_environment_is_source_cubemap()) {
        return zr_environment_source_cube_color_at_lod(direction, 0.0);
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
    if (zr_environment_is_source_cubemap()) {
        let max_mip = max(scene.environment_sample_params.w - 1.0, 0.0);
        let lod = zr_environment_mip_from_roughness(clamp(roughness, 0.0, 1.0), max_mip);
        return zr_environment_specular_pmrem_color_at_lod(reflected, lod);
    }
    let sharp_reflection = zr_environment_sky_color(reflected);
    let rough_reflection = zr_environment_sky_color(normal);
    return mix(sharp_reflection, rough_reflection, clamp(roughness, 0.0, 1.0));
}

fn zr_environment_diffuse_color(normal_ws: vec3<f32>) -> vec3<f32> {
    if (zr_environment_is_source_cubemap()) {
        if (ZR_ENV_DIFFUSE_IEM) {
            return zr_environment_irradiance_cube_color(normal_ws);
        }
        return zr_environment_sh9_eval(normal_ws) * max(scene.environment_params.y, 0.0);
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
        reflection * zr_environment_env_brdf_lut(f0, clamped_roughness, no_v);
    return (diffuse_environment + specular_environment) * clamped_occlusion;
}
