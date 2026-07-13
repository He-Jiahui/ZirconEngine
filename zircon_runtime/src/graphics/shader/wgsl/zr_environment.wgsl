const ZR_ENVIRONMENT_EPSILON: f32 = 0.000001;
const ZR_PLANAR_NEAR_CLIP_EPSILON: f32 = 0.001;
const ZR_ENVIRONMENT_SOURCE_CUBEMAP_KIND: f32 = 3.0;
const ZR_ENVIRONMENT_REALTIME_IBL_KIND: f32 = 4.0;
const ZR_ENVIRONMENT_PROBE_FACE_SIZE: f32 = 128.0;
const ZR_ENVIRONMENT_INVALID_PROBE: u32 = 0xffffffffu;
override ZR_ENV_DIFFUSE_IEM: bool = false;

struct ZrGpuReflectionProbe {
    position_blend: vec4<f32>,
    box_min: vec4<f32>,
    box_max: vec4<f32>,
    proj_params: vec4<f32>,
    rotation: vec4<f32>,
    misc: vec4<f32>,
};

struct ZrReflectionProbeHeader {
    probe_count: u32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
};

struct ZrReflectionProbeSelection {
    primary_index: u32,
    secondary_index: u32,
    primary_weight: f32,
    secondary_weight: f32,
};

struct ZrPlanarReflection {
    clip_from_world: mat4x4<f32>,
    local_from_world: mat4x4<f32>,
    bounds_min: vec4<f32>,
    bounds_max: vec4<f32>,
    sample_params: vec4<f32>,
};

struct ZrEnvironmentSh9 {
    coefficients: array<vec4<f32>, 9>,
};

@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;
@group(0) @binding(2) var zr_environment_sampler: sampler;
@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;
@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;
@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;
@group(0) @binding(6) var<uniform> zr_environment_sh9: ZrEnvironmentSh9;
@group(1) @binding(16) var<storage, read> zr_env_probes: array<ZrGpuReflectionProbe>;
@group(1) @binding(17) var<uniform> zr_env_probe_header: ZrReflectionProbeHeader;
@group(1) @binding(18) var zr_env_probe_cubemaps: texture_cube_array<f32>;
@group(1) @binding(29) var zr_env_planar_reflection: texture_2d<f32>;
@group(1) @binding(30) var<uniform> zr_env_planar: ZrPlanarReflection;

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

fn zr_environment_fix_cube_lookup_for_face_size(
    direction: vec3<f32>,
    lod: f32,
    face_size: f32,
) -> vec3<f32> {
    var adjusted = direction;
    let scale = clamp(1.0 - exp2(max(lod, 0.0)) / max(face_size, 1.0), 0.0, 1.0);
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

fn zr_environment_fix_source_cube_lookup(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    return zr_environment_fix_cube_lookup_for_face_size(
        direction,
        lod,
        scene.environment_sample_params.y,
    );
}

fn zr_environment_fix_pmrem_cube_lookup(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    return zr_environment_fix_cube_lookup_for_face_size(
        direction,
        lod,
        scene.environment_sample_params.z,
    );
}

fn zr_environment_source_cube_color_at_lod(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    let mip_count = floor(log2(max(scene.environment_sample_params.y, 1.0))) + 1.0;
    let max_mip = mip_count - 1.0;
    let clamped_lod = clamp(lod, 0.0, max_mip);
    let rotated = zr_environment_rotated_direction(zr_environment_normalize_or_zero(direction));
    return textureSampleLevel(
        zr_environment_source_cube,
        zr_environment_sampler,
        zr_environment_fix_source_cube_lookup(rotated, clamped_lod),
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
        zr_environment_fix_pmrem_cube_lookup(rotated, clamped_lod),
        clamped_lod,
    ).rgb * max(scene.environment_params.y, 0.0);
}

fn zr_environment_mip_from_roughness(roughness: f32, max_mip: f32) -> f32 {
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    if (clamped_roughness <= 0.000001 || max_mip <= 0.0) {
        return 0.0;
    }
    return clamp(max_mip - 2.0 + 1.2 * log2(clamped_roughness), 0.0, max_mip);
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

fn zr_environment_irradiance_cube_color(normal_ws: vec3<f32>) -> vec3<f32> {
    let rotated = zr_environment_rotated_direction(zr_environment_normalize_or_zero(normal_ws));
    return textureSample(
        zr_environment_irradiance_cube,
        zr_environment_sampler,
        zr_environment_fix_cube_lookup_for_face_size(rotated, 0.0, 32.0),
    ).rgb
        * max(scene.environment_params.y, 0.0);
}

fn zr_environment_procedural_sky_color(direction: vec3<f32>) -> vec3<f32> {
    let normalized_direction = zr_environment_normalize_or_zero(direction);
    let sky_t = clamp(normalized_direction.y * 0.5 + 0.5, 0.0, 1.0);
    let ground_t = clamp(normalized_direction.y + 1.0, 0.0, 1.0);
    let sky = mix(scene.sky_horizon_color.rgb, scene.sky_zenith_color.rgb, sky_t);
    let ground = mix(scene.sky_ground_color.rgb, scene.sky_horizon_color.rgb, ground_t);
    var color = select(ground, sky, normalized_direction.y >= 0.0)
        * max(scene.environment_params.y, 0.0);
    let sun_direction_length = length(scene.sky_sun_direction.xyz);
    if (
        scene.sky_sun_direction.w >= 0.5
        && scene.sky_sun_params.x > 0.0
        && sun_direction_length > 0.000001
    ) {
        let sun_direction = scene.sky_sun_direction.xyz / sun_direction_length;
        let angular_radius = clamp(scene.sky_sun_color_radius.w, 0.0001, 1.5707963);
        let sun_mask = smoothstep(
            cos(angular_radius),
            cos(angular_radius * 0.72),
            dot(normalized_direction, sun_direction),
        );
        color += scene.sky_sun_color_radius.rgb * scene.sky_sun_params.x * sun_mask;
    }
    return color;
}

fn zr_environment_sky_color(direction: vec3<f32>) -> vec3<f32> {
    if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl()) {
        return zr_environment_source_cube_color_at_lod(direction, 0.0);
    }
    return zr_environment_procedural_sky_color(direction);
}

fn zr_environment_quat_rotate(rotation: vec4<f32>, value: vec3<f32>) -> vec3<f32> {
    let twice_cross = 2.0 * cross(rotation.xyz, value);
    return value + rotation.w * twice_cross + cross(rotation.xyz, twice_cross);
}

fn zr_environment_quat_rotate_inverse(rotation: vec4<f32>, value: vec3<f32>) -> vec3<f32> {
    return zr_environment_quat_rotate(vec4<f32>(-rotation.xyz, rotation.w), value);
}

fn zr_environment_probe_weight(
    probe: ZrGpuReflectionProbe,
    world_position: vec3<f32>,
) -> f32 {
    let local_position = zr_environment_quat_rotate_inverse(
        probe.rotation,
        world_position - probe.position_blend.xyz,
    );
    var edge_distance = 0.0;
    if (probe.box_max.w >= 0.5) {
        edge_distance = probe.box_max.x - length(local_position);
    } else {
        edge_distance = min(
            probe.box_max.x - abs(local_position.x),
            min(
                probe.box_max.y - abs(local_position.y),
                probe.box_max.z - abs(local_position.z),
            ),
        );
    }
    if (edge_distance <= 0.0) {
        return 0.0;
    }
    let blend_distance = probe.position_blend.w;
    if (blend_distance <= ZR_ENVIRONMENT_EPSILON) {
        return 1.0;
    }
    return clamp(edge_distance / blend_distance, 0.0, 1.0);
}

fn zr_environment_probe_is_better(
    weight: f32,
    priority: f32,
    index: u32,
    best_weight: f32,
    best_priority: f32,
    best_index: u32,
) -> bool {
    if (weight > best_weight + ZR_ENVIRONMENT_EPSILON) {
        return true;
    }
    if (abs(weight - best_weight) > ZR_ENVIRONMENT_EPSILON) {
        return false;
    }
    if (priority > best_priority + ZR_ENVIRONMENT_EPSILON) {
        return true;
    }
    if (abs(priority - best_priority) > ZR_ENVIRONMENT_EPSILON) {
        return false;
    }
    return best_index == ZR_ENVIRONMENT_INVALID_PROBE || index < best_index;
}

fn zr_environment_select_probes(world_position: vec3<f32>) -> ZrReflectionProbeSelection {
    var primary_index = ZR_ENVIRONMENT_INVALID_PROBE;
    var secondary_index = ZR_ENVIRONMENT_INVALID_PROBE;
    var primary_weight = 0.0;
    var secondary_weight = 0.0;
    var primary_priority = -3.402823e38;
    var secondary_priority = -3.402823e38;
    let probe_count = min(zr_env_probe_header.probe_count, arrayLength(&zr_env_probes));
    for (var probe_index = 0u; probe_index < probe_count; probe_index = probe_index + 1u) {
        let probe = zr_env_probes[probe_index];
        let weight = zr_environment_probe_weight(probe, world_position);
        if (weight <= 0.0) {
            continue;
        }
        let priority = probe.box_min.w;
        if (zr_environment_probe_is_better(
            weight,
            priority,
            probe_index,
            primary_weight,
            primary_priority,
            primary_index,
        )) {
            secondary_index = primary_index;
            secondary_weight = primary_weight;
            secondary_priority = primary_priority;
            primary_index = probe_index;
            primary_weight = weight;
            primary_priority = priority;
        } else if (zr_environment_probe_is_better(
            weight,
            priority,
            probe_index,
            secondary_weight,
            secondary_priority,
            secondary_index,
        )) {
            secondary_index = probe_index;
            secondary_weight = weight;
            secondary_priority = priority;
        }
    }
    primary_weight = clamp(primary_weight, 0.0, 1.0);
    secondary_weight = clamp(secondary_weight, 0.0, 1.0 - primary_weight);
    return ZrReflectionProbeSelection(
        primary_index,
        secondary_index,
        primary_weight,
        secondary_weight,
    );
}

fn zr_environment_box_project(
    reflection_direction: vec3<f32>,
    world_position: vec3<f32>,
    probe: ZrGpuReflectionProbe,
) -> vec3<f32> {
    if (probe.proj_params.w <= 0.5) {
        return reflection_direction;
    }
    let local_position = zr_environment_quat_rotate_inverse(
        probe.rotation,
        world_position - probe.position_blend.xyz,
    );
    let local_direction = zr_environment_quat_rotate_inverse(probe.rotation, reflection_direction);
    let extent = probe.proj_params.xyz;
    var distance = 3.402823e38;
    if (abs(local_direction.x) > ZR_ENVIRONMENT_EPSILON) {
        let plane = select(-extent.x, extent.x, local_direction.x > 0.0);
        let axis_distance = (plane - local_position.x) / local_direction.x;
        if (axis_distance >= 0.0) {
            distance = min(distance, axis_distance);
        }
    }
    if (abs(local_direction.y) > ZR_ENVIRONMENT_EPSILON) {
        let plane = select(-extent.y, extent.y, local_direction.y > 0.0);
        let axis_distance = (plane - local_position.y) / local_direction.y;
        if (axis_distance >= 0.0) {
            distance = min(distance, axis_distance);
        }
    }
    if (abs(local_direction.z) > ZR_ENVIRONMENT_EPSILON) {
        let plane = select(-extent.z, extent.z, local_direction.z > 0.0);
        let axis_distance = (plane - local_position.z) / local_direction.z;
        if (axis_distance >= 0.0) {
            distance = min(distance, axis_distance);
        }
    }
    if (distance >= 3.0e38) {
        return reflection_direction;
    }
    let local_hit = local_position + local_direction * distance;
    return zr_environment_quat_rotate(probe.rotation, local_hit);
}

fn zr_environment_probe_color(
    probe_index: u32,
    world_position: vec3<f32>,
    reflection_direction: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
    if (probe_index == ZR_ENVIRONMENT_INVALID_PROBE) {
        return vec3<f32>(0.0);
    }
    let probe = zr_env_probes[probe_index];
    let max_mip = max(probe.misc.y - 1.0, 0.0);
    let lod = zr_environment_mip_from_roughness(roughness, max_mip);
    let projected = zr_environment_box_project(reflection_direction, world_position, probe);
    let direction = zr_environment_fix_cube_lookup_for_face_size(
        zr_environment_normalize_or_zero(projected),
        lod,
        ZR_ENVIRONMENT_PROBE_FACE_SIZE,
    );
    return textureSampleLevel(
        zr_env_probe_cubemaps,
        zr_environment_sampler,
        direction,
        i32(probe.misc.z),
        lod,
    ).rgb * max(probe.misc.x, 0.0);
}

fn zr_environment_sky_reflection_color(
    normal: vec3<f32>,
    reflected: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
    if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl()) {
        let max_mip = max(scene.environment_sample_params.w - 1.0, 0.0);
        let lod = zr_environment_mip_from_roughness(roughness, max_mip);
        return zr_environment_specular_pmrem_color_at_lod(reflected, lod);
    }
    let sharp_reflection = zr_environment_sky_color(reflected);
    let rough_reflection = zr_environment_sky_color(normal);
    return mix(sharp_reflection, rough_reflection, roughness);
}

fn zr_environment_planar_reflection(
    world_position: vec3<f32>,
    roughness: f32,
) -> vec4<f32> {
    if (zr_env_planar.sample_params.w < 0.5) {
        return vec4<f32>(0.0);
    }
    let local = (zr_env_planar.local_from_world * vec4<f32>(world_position, 1.0)).xyz;
    if (any(local < zr_env_planar.bounds_min.xyz)
        || any(local > zr_env_planar.bounds_max.xyz)) {
        return vec4<f32>(0.0);
    }
    let clip = zr_env_planar.clip_from_world * vec4<f32>(world_position, 1.0);
    if (clip.w <= ZR_ENVIRONMENT_EPSILON) {
        return vec4<f32>(0.0);
    }
    let ndc = clip.xyz / clip.w;
    if (ndc.z < -ZR_PLANAR_NEAR_CLIP_EPSILON || ndc.z > 1.0
        || any(abs(ndc.xy) > vec2<f32>(1.0))) {
        return vec4<f32>(0.0);
    }
    let capture_uv = vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
    let uv = capture_uv * zr_env_planar.sample_params.xy;
    let max_mip = max(zr_env_planar.sample_params.z - 1.0, 0.0);
    let lod = zr_environment_mip_from_roughness(roughness, max_mip);
    let color = textureSampleLevel(
        zr_env_planar_reflection,
        zr_environment_sampler,
        uv,
        lod,
    ).rgb;
    return vec4<f32>(color, 1.0);
}

fn zr_environment_reflection_color(
    world_position: vec3<f32>,
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
    let normal = zr_environment_normalize_or_zero(normal_ws);
    let view_dir = zr_environment_normalize_or_zero(view_dir_ws);
    let reflected = reflect(-view_dir, normal);
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    let sky = zr_environment_sky_reflection_color(normal, reflected, clamped_roughness);
    let planar = zr_environment_planar_reflection(world_position, clamped_roughness);
    if (planar.a > 0.0) {
        return planar.rgb;
    }
    let selection = zr_environment_select_probes(world_position);
    let sky_weight = max(1.0 - selection.primary_weight - selection.secondary_weight, 0.0);
    return zr_environment_probe_color(
        selection.primary_index,
        world_position,
        reflected,
        clamped_roughness,
    ) * selection.primary_weight
        + zr_environment_probe_color(
            selection.secondary_index,
            world_position,
            reflected,
            clamped_roughness,
        ) * selection.secondary_weight
        + sky * sky_weight;
}

fn zr_environment_diffuse_color(normal_ws: vec3<f32>) -> vec3<f32> {
    if (zr_environment_is_source_cubemap()) {
        if (ZR_ENV_DIFFUSE_IEM) {
            return zr_environment_irradiance_cube_color(normal_ws);
        }
        return zr_environment_sh9_eval(normal_ws) * max(scene.environment_params.y, 0.0);
    }
    if (zr_environment_is_realtime_ibl()) {
        return zr_environment_sh9_eval(normal_ws) * max(scene.environment_params.y, 0.0);
    }
    return zr_environment_sky_color(normal_ws);
}

struct ZrEnvironmentPbrComponents {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
};

fn zr_environment_pbr_components(
    world_position: vec3<f32>,
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> ZrEnvironmentPbrComponents {
    if (!zr_environment_is_enabled() || !is_standard_pbr) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
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
    let reflection = zr_environment_reflection_color(
        world_position,
        normal,
        view_dir,
        clamped_roughness,
    );
    let specular_environment =
        reflection * zr_environment_env_brdf_lut(f0, clamped_roughness, no_v);
    return ZrEnvironmentPbrComponents(
        diffuse_environment * clamped_occlusion,
        specular_environment * clamped_occlusion,
    );
}

fn zr_environment_pbr_indirect(
    world_position: vec3<f32>,
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> vec3<f32> {
    let components = zr_environment_pbr_components(
        world_position,
        normal_ws,
        view_dir_ws,
        roughness,
        metallic,
        diffuse_color,
        base_color,
        occlusion,
        is_standard_pbr,
    );
    return components.diffuse + components.specular;
}
