const ZR_ENVIRONMENT_EPSILON: f32 = 0.000001;
const ZR_ENVIRONMENT_ROUGHEST_PMREM_MIP: f32 = 1.0;
const ZR_ENVIRONMENT_PMREM_ROUGHNESS_MIP_SCALE: f32 = 1.2;
const ZR_ENVIRONMENT_SOURCE_CUBEMAP_KIND: f32 = 3.0;
const ZR_ENVIRONMENT_REALTIME_IBL_KIND: f32 = 4.0;

struct ZrEnvironmentSh9 {
    coefficients: array<vec4<f32>, 9>,
};

struct ZrEnvironmentPbrComponents {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
};

@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;
@group(0) @binding(2) var zr_environment_sampler: sampler;
@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;
@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;
@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;
@group(0) @binding(6) var<uniform> zr_environment_sh9: ZrEnvironmentSh9;

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
    return scene.environment_sample_params.x >= ZR_ENVIRONMENT_SOURCE_CUBEMAP_KIND - 0.5
        && scene.environment_sample_params.x < ZR_ENVIRONMENT_REALTIME_IBL_KIND - 0.5;
}

fn zr_environment_is_realtime_ibl() -> bool {
    return scene.environment_sample_params.x >= ZR_ENVIRONMENT_REALTIME_IBL_KIND - 0.5;
}

fn zr_environment_has_irradiance_cube() -> bool {
    return scene.environment_params.x > 0.5;
}

fn zr_environment_rotated_direction(direction: vec3<f32>) -> vec3<f32> {
    if (scene.environment_rotation_sin_cos.z < 0.5) {
        return direction;
    }
    let s = scene.environment_rotation_sin_cos.x;
    let c = scene.environment_rotation_sin_cos.y;
    return vec3<f32>(
        direction.x * c - direction.z * s,
        direction.y,
        direction.x * s + direction.z * c,
    );
}

fn zr_environment_fix_cube_lookup_for_face_size(
    direction: vec3<f32>,
    _lod: f32,
    _face_size: f32,
) -> vec3<f32> {
    // WGPU cube sampling filters across face edges natively. Do not apply the
    // legacy cmft/OpenGL edge warp to a direction before lookup.
    return direction;
}

fn zr_environment_fix_pmrem_cube_lookup(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    return zr_environment_fix_cube_lookup_for_face_size(
        direction,
        lod,
        scene.environment_sample_params.z,
    );
}

fn zr_environment_specular_pmrem_color_at_clamped_lod_normalized(
    direction: vec3<f32>,
    clamped_lod: f32,
) -> vec3<f32> {
    let rotated = zr_environment_rotated_direction(direction);
    return textureSampleLevel(
        zr_environment_specular_pmrem_cube,
        zr_environment_sampler,
        zr_environment_fix_pmrem_cube_lookup(rotated, clamped_lod),
        clamped_lod,
    ).rgb * max(scene.environment_params.y, 0.0);
}

fn zr_environment_mip_from_roughness(roughness: f32, max_mip: f32) -> f32 {
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    if (clamped_roughness <= ZR_ENVIRONMENT_EPSILON || max_mip <= 0.0) {
        return 0.0;
    }
    return clamp(
        max_mip - ZR_ENVIRONMENT_ROUGHEST_PMREM_MIP
            + ZR_ENVIRONMENT_PMREM_ROUGHNESS_MIP_SCALE * log2(clamped_roughness),
        0.0,
        max_mip,
    );
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

fn zr_environment_sh9_eval_normalized(n: vec3<f32>) -> vec3<f32> {
    let x = n.x;
    let y = n.y;
    let z = n.z;
    var irradiance = zr_environment_sh9.coefficients[0].rgb * 0.2820948;
    irradiance += zr_environment_sh9.coefficients[1].rgb * (0.48860252 * z);
    irradiance += zr_environment_sh9.coefficients[2].rgb * (0.48860252 * y);
    irradiance += zr_environment_sh9.coefficients[3].rgb * (0.48860252 * x);
    irradiance += zr_environment_sh9.coefficients[4].rgb * (1.0925485 * x * z);
    irradiance += zr_environment_sh9.coefficients[5].rgb * (1.0925485 * z * y);
    irradiance += zr_environment_sh9.coefficients[6].rgb * (0.31539157 * (3.0 * y * y - 1.0));
    irradiance += zr_environment_sh9.coefficients[7].rgb * (1.0925485 * x * y);
    irradiance += zr_environment_sh9.coefficients[8].rgb * (0.54627424 * (x * x - z * z));
    return max(irradiance, vec3<f32>(0.0, 0.0, 0.0));
}

fn zr_environment_sh9_color_normalized(normal: vec3<f32>) -> vec3<f32> {
    let rotated = zr_environment_rotated_direction(normal);
    return zr_environment_sh9_eval_normalized(rotated)
        * max(scene.environment_params.y, 0.0);
}

fn zr_environment_irradiance_cube_color_normalized(normal: vec3<f32>) -> vec3<f32> {
    let rotated = zr_environment_rotated_direction(normal);
    return textureSample(
        zr_environment_irradiance_cube,
        zr_environment_sampler,
        zr_environment_fix_cube_lookup_for_face_size(rotated, 0.0, 32.0),
    ).rgb
        * max(scene.environment_params.y, 0.0);
}

fn zr_environment_procedural_sky_color_normalized(
    normalized_direction: vec3<f32>,
) -> vec3<f32> {
    let sky_t = clamp(normalized_direction.y * 0.5 + 0.5, 0.0, 1.0);
    let ground_t = clamp(normalized_direction.y + 1.0, 0.0, 1.0);
    let sky = mix(scene.sky_horizon_color.rgb, scene.sky_zenith_color.rgb, sky_t);
    let ground = mix(scene.sky_ground_color.rgb, scene.sky_horizon_color.rgb, ground_t);
    var color = select(ground, sky, normalized_direction.y >= 0.0);
    if (
        scene.sky_sun_direction.w >= 0.5
        && scene.sky_sun_params.x > 0.0
    ) {
        let sun_mask = smoothstep(
            scene.sky_sun_params.y,
            scene.sky_sun_params.z,
            dot(normalized_direction, scene.sky_sun_direction.xyz),
        );
        color += scene.sky_sun_color_radius.rgb * scene.sky_sun_params.x * sun_mask;
    }
    return color * max(scene.environment_params.y, 0.0);
}

fn zr_environment_sky_reflection_color(
    reflected: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
    if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl()) {
        let max_mip = max(scene.environment_sample_params.w - 1.0, 0.0);
        let lod = zr_environment_mip_from_roughness(roughness, max_mip);
        return zr_environment_specular_pmrem_color_at_clamped_lod_normalized(reflected, lod);
    }
    // A sky without a PMREM has no roughness convolution. Keep the reflected
    // direction instead of fabricating a rough lobe from the surface normal.
    return zr_environment_procedural_sky_color_normalized(reflected);
}

fn zr_environment_diffuse_color_normalized(normal: vec3<f32>) -> vec3<f32> {
    if (zr_environment_is_source_cubemap()) {
        if (zr_environment_has_irradiance_cube()) {
            return zr_environment_irradiance_cube_color_normalized(normal);
        }
        return zr_environment_sh9_color_normalized(normal);
    }
    if (zr_environment_is_realtime_ibl()) {
        return zr_environment_sh9_color_normalized(normal);
    }
    return zr_environment_procedural_sky_color_normalized(normal);
}

fn zr_environment_specular_occlusion(
    no_v: f32,
    roughness: f32,
    occlusion: f32,
) -> f32 {
    let clamped_no_v = clamp(no_v, 0.0, 1.0);
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    let clamped_occlusion = clamp(occlusion, 0.0, 1.0);
    if (clamped_occlusion <= 0.0
        || clamped_occlusion >= 1.0
        || clamped_roughness <= ZR_ENVIRONMENT_EPSILON)
    {
        return clamped_occlusion;
    }
    let roughness_sq = clamped_roughness * clamped_roughness;
    let specular_occlusion =
        pow(clamped_no_v + clamped_occlusion, roughness_sq) - 1.0 + clamped_occlusion;
    return clamp(specular_occlusion, 0.0, 1.0);
}

fn zr_environment_pbr_components_from_reflection(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    clamped_roughness: f32,
    clamped_metallic: f32,
    clamped_occlusion: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    has_global_environment: bool,
    reflection: vec3<f32>,
) -> ZrEnvironmentPbrComponents {
    let diffuse_energy_scale = 1.0 - clamped_metallic;
    var diffuse_environment = vec3<f32>(0.0);
    if (diffuse_energy_scale > 0.0
        && has_global_environment
        && any(diffuse_color != vec3<f32>(0.0)))
    {
        diffuse_environment = zr_environment_diffuse_color_normalized(normal)
            * diffuse_color
            * diffuse_energy_scale;
    }
    var specular_environment = vec3<f32>(0.0);
    if (any(reflection != vec3<f32>(0.0))) {
        let f0 = mix(
            vec3<f32>(0.04, 0.04, 0.04),
            max(base_color, vec3<f32>(0.0, 0.0, 0.0)),
            clamped_metallic,
        );
        let no_v = clamp(dot(normal, view_dir), 0.0, 1.0);
        let specular_occlusion = zr_environment_specular_occlusion(
            no_v,
            clamped_roughness,
            clamped_occlusion,
        );
        specular_environment = reflection
            * zr_environment_env_brdf_lut(f0, clamped_roughness, no_v)
            * specular_occlusion;
    }
    return ZrEnvironmentPbrComponents(
        diffuse_environment * clamped_occlusion,
        specular_environment,
    );
}
