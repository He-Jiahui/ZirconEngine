const ZR_ENVIRONMENT_EPSILON: f32 = 0.000001;
const ZR_ENVIRONMENT_SOURCE_CUBEMAP_KIND: f32 = 3.0;
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

@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;
@group(0) @binding(2) var zr_environment_sampler: sampler;
@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;
@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;
@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;
@group(1) @binding(16) var<storage, read> zr_env_probes: array<ZrGpuReflectionProbe>;
@group(1) @binding(17) var<uniform> zr_env_probe_header: ZrReflectionProbeHeader;
@group(1) @binding(18) var zr_env_probe_cubemaps: texture_cube_array<f32>;

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

fn zr_environment_fix_cube_lookup(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    return zr_environment_fix_cube_lookup_for_face_size(
        direction,
        lod,
        scene.environment_sample_params.y,
    );
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
    if (zr_environment_is_source_cubemap()) {
        let max_mip = max(scene.environment_sample_params.w - 1.0, 0.0);
        let lod = zr_environment_mip_from_roughness(roughness, max_mip);
        return zr_environment_specular_pmrem_color_at_lod(reflected, lod);
    }
    let sharp_reflection = zr_environment_sky_color(reflected);
    let rough_reflection = zr_environment_sky_color(normal);
    return mix(sharp_reflection, rough_reflection, roughness);
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
    return zr_environment_sky_color(normal_ws);
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
    let reflection = zr_environment_reflection_color(
        world_position,
        normal,
        view_dir,
        clamped_roughness,
    );
    let specular_environment =
        reflection * zr_environment_env_brdf_lut(f0, clamped_roughness, no_v);
    return (diffuse_environment + specular_environment) * clamped_occlusion;
}
