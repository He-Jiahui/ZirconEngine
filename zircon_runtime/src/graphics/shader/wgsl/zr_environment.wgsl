const ZR_PLANAR_NEAR_CLIP_EPSILON: f32 = 0.001;
const ZR_ENVIRONMENT_PROBE_FACE_SIZE: f32 = 128.0;
const ZR_ENVIRONMENT_INVALID_PROBE: u32 = 0xffffffffu;

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

struct ZrEnvironmentPbrPreparedInputs {
    has_global_environment: bool,
    is_active: bool,
    clamped_metallic: f32,
    clamped_roughness: f32,
    clamped_occlusion: f32,
};

@group(1) @binding(16) var<storage, read> zr_env_probes: array<ZrGpuReflectionProbe>;
@group(1) @binding(17) var<uniform> zr_env_probe_header: ZrReflectionProbeHeader;
@group(1) @binding(18) var zr_env_probe_cubemaps: texture_cube_array<f32>;
@group(1) @binding(29) var zr_env_planar_reflection: texture_2d<f32>;
@group(1) @binding(30) var<uniform> zr_env_planar: ZrPlanarReflection;

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
    let position_delta = world_position - probe.position_blend.xyz;
    var edge_distance = 0.0;
    if (probe.box_max.w >= 0.5) {
        // Sphere influence is rotation-invariant, so keep this path in world space.
        edge_distance = probe.box_max.x - length(position_delta);
    } else {
        let local_position = zr_environment_quat_rotate_inverse(probe.rotation, position_delta);
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
        if (!(probe.misc.x > 0.0)) {
            continue;
        }
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
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    let planar = zr_environment_planar_reflection(world_position, clamped_roughness);
    if (planar.a > 0.0) {
        return planar.rgb;
    }
    let has_global_environment = zr_environment_is_enabled()
        && scene.environment_params.y > 0.0;
    if (zr_env_probe_header.probe_count == 0u && !has_global_environment) {
        return vec3<f32>(0.0);
    }
    return zr_environment_reflection_color_after_planar(
        world_position,
        zr_environment_normalize_or_zero(normal_ws),
        zr_environment_normalize_or_zero(view_dir_ws),
        clamped_roughness,
        has_global_environment,
    );
}

fn zr_environment_reflection_color_normalized(
    world_position: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    clamped_roughness: f32,
    has_global_environment: bool,
) -> vec3<f32> {
    let planar = zr_environment_planar_reflection(world_position, clamped_roughness);
    if (planar.a > 0.0) {
        return planar.rgb;
    }
    return zr_environment_reflection_color_after_planar(
        world_position,
        normal,
        view_dir,
        clamped_roughness,
        has_global_environment,
    );
}

fn zr_environment_reflection_color_after_planar(
    world_position: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    clamped_roughness: f32,
    has_global_environment: bool,
) -> vec3<f32> {
    if (zr_env_probe_header.probe_count == 0u && !has_global_environment) {
        return vec3<f32>(0.0);
    }
    let reflected = reflect(-view_dir, normal);
    if (zr_env_probe_header.probe_count == 0u) {
        return zr_environment_sky_reflection_color(reflected, clamped_roughness);
    }
    let selection = zr_environment_select_probes(world_position);
    let sky_weight = max(1.0 - selection.primary_weight - selection.secondary_weight, 0.0);
    var sky = vec3<f32>(0.0);
    if (sky_weight > 0.0 && has_global_environment) {
        sky = zr_environment_sky_reflection_color(reflected, clamped_roughness);
    }
    var primary = vec3<f32>(0.0);
    if (selection.primary_weight > 0.0) {
        primary = zr_environment_probe_color(
            selection.primary_index,
            world_position,
            reflected,
            clamped_roughness,
        );
    }
    var secondary = vec3<f32>(0.0);
    if (selection.secondary_weight > 0.0) {
        secondary = zr_environment_probe_color(
            selection.secondary_index,
            world_position,
            reflected,
            clamped_roughness,
        );
    }
    return primary * selection.primary_weight
        + secondary * selection.secondary_weight
        + sky * sky_weight;
}

fn zr_environment_pbr_prepared_inputs(
    roughness: f32,
    metallic: f32,
    occlusion: f32,
    is_standard_pbr: bool,
) -> ZrEnvironmentPbrPreparedInputs {
    if (!is_standard_pbr) {
        return ZrEnvironmentPbrPreparedInputs(false, false, 0.0, 0.0, 0.0);
    }
    let environment_intensity = max(scene.environment_params.y, 0.0);
    let has_global_environment = zr_environment_is_enabled()
        && environment_intensity > 0.0;
    if (!has_global_environment
        && zr_env_probe_header.probe_count == 0u
        && zr_env_planar.sample_params.w < 0.5)
    {
        return ZrEnvironmentPbrPreparedInputs(
            has_global_environment,
            false,
            0.0,
            0.0,
            0.0,
        );
    }
    let clamped_metallic = clamp(metallic, 0.0, 1.0);
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    let clamped_occlusion = clamp(occlusion, 0.0, 1.0);
    if (clamped_occlusion <= 0.0) {
        return ZrEnvironmentPbrPreparedInputs(
            has_global_environment,
            false,
            clamped_metallic,
            clamped_roughness,
            clamped_occlusion,
        );
    }
    return ZrEnvironmentPbrPreparedInputs(
        has_global_environment,
        true,
        clamped_metallic,
        clamped_roughness,
        clamped_occlusion,
    );
}

fn zr_environment_pbr_components_with_prepared_inputs(
    world_position: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    prepared: ZrEnvironmentPbrPreparedInputs,
) -> ZrEnvironmentPbrComponents {
    if (all(normal == vec3<f32>(0.0)) || all(view_dir == vec3<f32>(0.0))) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let reflection = zr_environment_reflection_color_normalized(
        world_position,
        normal,
        view_dir,
        prepared.clamped_roughness,
        prepared.has_global_environment,
    );
    return zr_environment_pbr_components_from_reflection(
        normal,
        view_dir,
        prepared.clamped_roughness,
        prepared.clamped_metallic,
        prepared.clamped_occlusion,
        diffuse_color,
        base_color,
        prepared.has_global_environment,
        reflection,
    );
}

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
    let prepared = zr_environment_pbr_prepared_inputs(
        roughness,
        metallic,
        occlusion,
        is_standard_pbr,
    );
    if (!prepared.is_active) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    return zr_environment_pbr_components_with_prepared_inputs(
        world_position,
        zr_environment_normalize_or_zero(normal_ws),
        zr_environment_normalize_or_zero(view_dir_ws),
        diffuse_color,
        base_color,
        prepared,
    );
}

fn zr_environment_pbr_components_normalized(
    world_position: vec3<f32>,
    normal_normalized: vec3<f32>,
    view_dir_normalized: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> ZrEnvironmentPbrComponents {
    let prepared = zr_environment_pbr_prepared_inputs(
        roughness,
        metallic,
        occlusion,
        is_standard_pbr,
    );
    if (!prepared.is_active) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    return zr_environment_pbr_components_with_prepared_inputs(
        world_position,
        normal_normalized,
        view_dir_normalized,
        diffuse_color,
        base_color,
        prepared,
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

fn zr_environment_pbr_indirect_normalized(
    world_position: vec3<f32>,
    normal_normalized: vec3<f32>,
    view_dir_normalized: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> vec3<f32> {
    let components = zr_environment_pbr_components_normalized(
        world_position,
        normal_normalized,
        view_dir_normalized,
        roughness,
        metallic,
        diffuse_color,
        base_color,
        occlusion,
        is_standard_pbr,
    );
    return components.diffuse + components.specular;
}
