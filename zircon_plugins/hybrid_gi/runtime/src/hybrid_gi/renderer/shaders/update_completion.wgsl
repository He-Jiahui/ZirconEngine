struct HybridGiCompletionParams {
    resident_probe_count: u32,
    pending_probe_count: u32,
    probe_budget: u32,
    trace_region_count: u32,
    scene_card_capture_request_count: u32,
    scene_voxel_clipmap_count: u32,
    scene_voxel_cell_count: u32,
    tracing_budget: u32,
    evictable_probe_count: u32,
    scene_light_seed_rgb: u32,
    scene_light_strength_q: u32,
    _padding1: u32,
};

struct ResidentProbeInput {
    probe_id: u32,
    slot: u32,
    ray_budget: u32,
    lineage_trace_support_q: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    previous_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_weight_q: u32,
    skip_scene_prepare_for_irradiance_q: u32,
    lineage_trace_lighting_rgb: u32,
    skip_scene_prepare_for_trace_q: u32,
    parent_probe_id: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
};

struct PendingProbeInput {
    probe_id: u32,
    logical_index: u32,
    ray_budget: u32,
    lineage_trace_support_q: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    runtime_hierarchy_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_weight_q: u32,
    skip_scene_prepare_for_irradiance_q: u32,
    lineage_trace_lighting_rgb: u32,
    skip_scene_prepare_for_trace_q: u32,
    parent_probe_id: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
};

struct TraceRegionInput {
    region_id: u32,
    center_x_q: u32,
    center_y_q: u32,
    center_z_q: u32,
    radius_q: u32,
    coverage_q: u32,
    rt_lighting_rgb: u32,
    _padding1: u32,
};

struct ScenePrepareDescriptor {
    descriptor_kind: u32,
    primary_id: u32,
    secondary_id: u32,
    tertiary_id: u32,
    quaternary_id: u32,
    scalar0: u32,
    scalar1: u32,
    scalar2: u32,
    scalar3: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
};

@group(0) @binding(0)
var<uniform> params: HybridGiCompletionParams;

@group(0) @binding(1)
var<storage, read> resident_probe_inputs: array<ResidentProbeInput>;

@group(0) @binding(2)
var<storage, read> pending_probe_updates: array<PendingProbeInput>;

@group(0) @binding(3)
var<storage, read> scheduled_trace_regions: array<TraceRegionInput>;

@group(0) @binding(4)
var<storage, read> scene_prepare_descriptors: array<ScenePrepareDescriptor>;

@group(0) @binding(5)
var<storage, read_write> completed_probe_updates: array<u32>;

@group(0) @binding(6)
var<storage, read_write> completed_trace_regions: array<u32>;

@group(0) @binding(7)
var<storage, read_write> probe_irradiance_updates: array<u32>;

@group(0) @binding(8)
var<storage, read_write> probe_trace_lighting_updates: array<u32>;

const NO_PARENT_PROBE_ID: u32 = 0xffffffffu;
const SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE: u32 = 1u;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CLIPMAP: u32 = 2u;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL: u32 = 3u;

fn available_probe_completion_budget() -> u32 {
    let free_budget = max(params.probe_budget, params.resident_probe_count) - params.resident_probe_count;
    return free_budget + params.evictable_probe_count;
}

fn active_trace_count() -> u32 {
    return min(params.trace_region_count, params.tracing_budget);
}

fn abs_diff_u32(a: u32, b: u32) -> u32 {
    if (a >= b) {
        return a - b;
    }
    return b - a;
}

fn pack_rgb8(rgb: vec3<u32>) -> u32 {
    return rgb.x | (rgb.y << 8u) | (rgb.z << 16u);
}

fn unpack_rgb8(packed: u32) -> vec3<u32> {
    return vec3<u32>(
        packed & 0xffu,
        (packed >> 8u) & 0xffu,
        (packed >> 16u) & 0xffu,
    );
}

fn scene_prepare_card_capture_base_rgb(descriptor: ScenePrepareDescriptor) -> vec3<u32> {
    if (descriptor._padding1 != 0u) {
        return unpack_rgb8(descriptor._padding0);
    }
    return vec3<u32>(
        96u + ((descriptor.primary_id * 17u + descriptor.secondary_id * 5u + descriptor.quaternary_id * 3u) % 96u),
        72u + ((descriptor.secondary_id * 13u + descriptor.tertiary_id * 7u + descriptor.scalar3) % 80u),
        40u + ((descriptor.primary_id * 11u + descriptor.scalar0 + descriptor.scalar2) % 56u),
    );
}

fn scene_prepare_voxel_owner_fallback_rgb(descriptor: ScenePrepareDescriptor) -> vec3<u32> {
    let owner_id = descriptor._padding0;
    return vec3<u32>(
        80u + ((owner_id * 17u + descriptor.primary_id * 13u + descriptor.secondary_id * 7u + descriptor.scalar0) % 96u),
        72u + ((owner_id * 11u + descriptor.primary_id * 5u + descriptor.secondary_id * 13u + descriptor.scalar1) % 104u),
        64u + ((owner_id * 19u + descriptor.primary_id * 7u + descriptor.secondary_id * 11u + descriptor.scalar2) % 120u),
    );
}

fn scene_prepare_owned_card_capture_base_rgb(owner_id: u32) -> vec4<u32> {
    let descriptor_count = scene_prepare_descriptor_count();
    for (var descriptor_index = 0u; descriptor_index < descriptor_count; descriptor_index = descriptor_index + 1u) {
        let descriptor = scene_prepare_descriptors[descriptor_index];
        if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE
            && descriptor.primary_id == owner_id) {
            let base_rgb = scene_prepare_card_capture_base_rgb(descriptor);
            return vec4<u32>(base_rgb.x, base_rgb.y, base_rgb.z, 1u);
        }
    }
    return vec4<u32>(0u, 0u, 0u, 0u);
}

fn temporal_update_weight(ray_budget: u32, tracing_budget: u32) -> u32 {
    return min(224u, 48u + min(ray_budget, 192u) / 2u + min(tracing_budget, 4u) * 12u);
}

fn apply_scene_light_seed(base_rgb: vec3<u32>) -> vec3<u32> {
    let seed_rgb = unpack_rgb8(params.scene_light_seed_rgb);
    let seeded_rgb = vec3<u32>(
        min(255u, (base_rgb.x * max(seed_rgb.x, 1u) + 127u) / 255u),
        min(255u, (base_rgb.y * max(seed_rgb.y, 1u) + 127u) / 255u),
        min(255u, (base_rgb.z * max(seed_rgb.z, 1u) + 127u) / 255u),
    );
    let strength_q = max(params.scene_light_strength_q, 1u);
    return vec3<u32>(
        min(255u, (seeded_rgb.x * strength_q + 127u) / 255u),
        min(255u, (seeded_rgb.y * strength_q + 127u) / 255u),
        min(255u, (seeded_rgb.z * strength_q + 127u) / 255u),
    );
}

fn scene_prepare_descriptor_count() -> u32 {
    return params.scene_card_capture_request_count
        + params.scene_voxel_clipmap_count
        + params.scene_voxel_cell_count;
}

fn scene_prepare_descriptor_base_rgb(descriptor: ScenePrepareDescriptor) -> vec3<u32> {
    var base_rgb = vec3<u32>(0u);
    if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE) {
        base_rgb = scene_prepare_card_capture_base_rgb(descriptor);
    } else if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CLIPMAP) {
        base_rgb = vec3<u32>(
            40u + ((descriptor.primary_id * 5u + descriptor.scalar0 + descriptor.scalar3) % 56u),
            88u + ((descriptor.primary_id * 11u + descriptor.scalar1 + descriptor.scalar3) % 88u),
            104u + ((descriptor.primary_id * 19u + descriptor.scalar2 + descriptor.scalar3) % 104u),
        );
    } else if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL) {
        if (descriptor._padding1 != 0u) {
            base_rgb = unpack_rgb8(descriptor.quaternary_id);
        } else if (descriptor._padding0 != 0u) {
            let owner_card_base_rgb = scene_prepare_owned_card_capture_base_rgb(descriptor._padding0);
            if (owner_card_base_rgb.w != 0u) {
                base_rgb = owner_card_base_rgb.xyz;
            } else {
                base_rgb = scene_prepare_voxel_owner_fallback_rgb(descriptor);
            }
        } else {
            base_rgb = vec3<u32>(
                120u + ((descriptor.primary_id * 13u + descriptor.secondary_id * 7u + descriptor.tertiary_id * 17u + descriptor.scalar0) % 96u),
                104u + ((descriptor.primary_id * 11u + descriptor.secondary_id * 5u + descriptor.tertiary_id * 9u + descriptor.scalar1) % 96u),
                88u + ((descriptor.primary_id * 19u + descriptor.secondary_id * 3u + descriptor.tertiary_id * 13u + descriptor.scalar2) % 96u),
            );
        }
    }
    return apply_scene_light_seed(base_rgb);
}

fn scene_prepare_descriptor_weight(
    distance_to_descriptor: u32,
    max_distance: u32,
    ray_budget: u32,
    descriptor_extent_q: u32,
    base_strength: u32,
) -> u32 {
    let proximity = max_distance - distance_to_descriptor;
    let proximity_weight = min(255u, (proximity * 255u) / max(max_distance, 1u));
    let descriptor_strength = min(
        255u,
        base_strength + min(ray_budget, 160u) / 2u + min(descriptor_extent_q, 192u) / 2u,
    );
    return (proximity_weight * descriptor_strength + 127u) / 255u;
}

fn scene_prepare_contribution_rgb(
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    ray_budget: u32,
) -> u32 {
    let descriptor_count = scene_prepare_descriptor_count();
    if (descriptor_count == 0u) {
        return 0u;
    }

    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    for (var descriptor_index = 0u; descriptor_index < descriptor_count; descriptor_index = descriptor_index + 1u) {
        let descriptor = scene_prepare_descriptors[descriptor_index];
        let distance_to_descriptor =
            abs_diff_u32(position_x_q, descriptor.scalar0)
            + abs_diff_u32(position_y_q, descriptor.scalar1)
            + abs_diff_u32(position_z_q, descriptor.scalar2);
        var max_distance = 0u;
        var descriptor_weight = 0u;

        if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE) {
            let descriptor_radius_q = max(descriptor.scalar3, 1u);
            let reach = max(radius_q + descriptor_radius_q + 1u, 1u);
            max_distance = max(reach * 3u, 1u);
            if (distance_to_descriptor >= max_distance) {
                continue;
            }
            descriptor_weight = scene_prepare_descriptor_weight(
                distance_to_descriptor,
                max_distance,
                ray_budget,
                descriptor_radius_q,
                92u,
            );
        } else if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CLIPMAP) {
            let descriptor_half_extent_q = max(descriptor.scalar3, 1u);
            max_distance = max(radius_q + descriptor_half_extent_q * 4u + 1u, 1u);
            if (distance_to_descriptor >= max_distance) {
                continue;
            }
            descriptor_weight = scene_prepare_descriptor_weight(
                distance_to_descriptor,
                max_distance,
                ray_budget,
                descriptor_half_extent_q,
                72u,
            );
        } else if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL) {
            let descriptor_cell_half_extent_q = max(descriptor.scalar3, 1u);
            let occupancy_count = max(descriptor.tertiary_id, 1u);
            max_distance = max(radius_q + descriptor_cell_half_extent_q * 5u + 1u, 1u);
            if (distance_to_descriptor >= max_distance) {
                continue;
            }
            descriptor_weight = scene_prepare_descriptor_weight(
                distance_to_descriptor,
                max_distance,
                ray_budget,
                descriptor_cell_half_extent_q,
                128u + min(occupancy_count, 8u) * 16u,
            );
        } else {
            continue;
        }

        let descriptor_rgb = scene_prepare_descriptor_base_rgb(descriptor);
        weighted_rgb = vec3<u32>(
            weighted_rgb.x + descriptor_rgb.x * descriptor_weight,
            weighted_rgb.y + descriptor_rgb.y * descriptor_weight,
            weighted_rgb.z + descriptor_rgb.z * descriptor_weight,
        );
        total_weight = total_weight + descriptor_weight;
    }

    if (total_weight == 0u) {
        return 0u;
    }

    return pack_rgb8(vec3<u32>(
        min(255u, (weighted_rgb.x + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.y + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.z + total_weight / 2u) / total_weight),
    ));
}

fn blend_traced_with_scene_prepare(
    traced_packed: u32,
    scene_prepare_packed: u32,
    ray_budget: u32,
) -> u32 {
    if (scene_prepare_packed == 0u) {
        return traced_packed;
    }
    if (traced_packed == 0u) {
        return scene_prepare_packed;
    }

    let scene_prepare_weight = min(
        224u,
        56u + min(ray_budget, 160u) / 2u + min(scene_prepare_descriptor_count(), 8u) * 12u,
    );
    let traced_rgb = unpack_rgb8(traced_packed);
    let scene_prepare_rgb = unpack_rgb8(scene_prepare_packed);
    return pack_rgb8(vec3<u32>(
        min(255u, traced_rgb.x + (scene_prepare_rgb.x * scene_prepare_weight + 127u) / 255u),
        min(255u, traced_rgb.y + (scene_prepare_rgb.y * scene_prepare_weight + 127u) / 255u),
        min(255u, traced_rgb.z + (scene_prepare_rgb.z * scene_prepare_weight + 127u) / 255u),
    ));
}

fn trace_region_base_rgb(region: TraceRegionInput) -> vec3<u32> {
    var base_rgb = unpack_rgb8(region.rt_lighting_rgb);
    if (region.rt_lighting_rgb == 0u) {
        base_rgb = vec3<u32>(
            32u + ((region.region_id * 17u + region.center_x_q + region.coverage_q) % 160u),
            32u + ((region.region_id * 11u + region.center_y_q + region.radius_q) % 160u),
            32u + ((region.region_id * 7u + region.center_z_q + region.coverage_q * 3u) % 160u),
        );
    }
    return apply_scene_light_seed(base_rgb);
}

fn trace_region_contribution_weight(
    distance_to_region: u32,
    max_distance: u32,
    ray_budget: u32,
    coverage_q: u32,
) -> u32 {
    let proximity = max_distance - distance_to_region;
    let proximity_weight = min(255u, (proximity * 255u) / max(max_distance, 1u));
    let trace_strength =
        min(255u, 32u + min(ray_budget, 160u) / 2u + min(coverage_q, 160u) / 2u + min(params.tracing_budget, 4u) * 40u);
    return (proximity_weight * trace_strength + 127u) / 255u;
}

fn resident_probe_gather_weight(
    distance_to_probe: u32,
    max_distance: u32,
    ray_budget: u32,
    resident_ray_budget: u32,
) -> u32 {
    let proximity = max_distance - distance_to_probe;
    let proximity_weight = min(255u, (proximity * 255u) / max(max_distance, 1u));
    let gather_strength =
        min(255u, 24u + min(ray_budget, 160u) / 3u + min(resident_ray_budget, 160u) / 3u
            + min(params.tracing_budget, 4u) * 20u);
    return (proximity_weight * gather_strength + 127u) / 255u;
}

fn hierarchy_probe_gather_boost(
    probe_id: u32,
    parent_probe_id: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
    resident_probe_id: u32,
    resident_parent_probe_id: u32,
) -> u32 {
    var boost = 0u;
    if (parent_probe_id != NO_PARENT_PROBE_ID && resident_probe_id == parent_probe_id) {
        boost = max(boost, 160u);
    }
    if (resident_ancestor_probe_id != NO_PARENT_PROBE_ID
        && resident_ancestor_depth > 1u
        && resident_probe_id == resident_ancestor_probe_id) {
        boost = max(
            boost,
            max(72u, 152u - min(resident_ancestor_depth - 1u, 3u) * 24u),
        );
    }
    if (resident_secondary_ancestor_probe_id != NO_PARENT_PROBE_ID
        && resident_secondary_ancestor_depth > 1u
        && resident_probe_id == resident_secondary_ancestor_probe_id) {
        boost = max(
            boost,
            max(56u, 132u - min(resident_secondary_ancestor_depth - 1u, 5u) * 16u),
        );
    }
    if (resident_tertiary_ancestor_probe_id != NO_PARENT_PROBE_ID
        && resident_tertiary_ancestor_depth > 1u
        && resident_probe_id == resident_tertiary_ancestor_probe_id) {
        boost = max(
            boost,
            max(44u, 116u - min(resident_tertiary_ancestor_depth - 1u, 7u) * 12u),
        );
    }
    if (resident_quaternary_ancestor_probe_id != NO_PARENT_PROBE_ID
        && resident_quaternary_ancestor_depth > 1u
        && resident_probe_id == resident_quaternary_ancestor_probe_id) {
        boost = max(
            boost,
            max(36u, 104u - min(resident_quaternary_ancestor_depth - 1u, 9u) * 10u),
        );
    }
    if (resident_parent_probe_id != NO_PARENT_PROBE_ID && resident_parent_probe_id == probe_id) {
        boost = max(boost, 96u);
    }
    return boost;
}

fn traced_contribution_rgb_for_trace_count(
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    ray_budget: u32,
    trace_count: u32,
) -> u32 {
    if (trace_count == 0u) {
        return 0u;
    }

    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    for (var trace_index = 0u; trace_index < trace_count; trace_index = trace_index + 1u) {
        let region = scheduled_trace_regions[trace_index];
        let reach = max(radius_q + region.radius_q + 1u, 1u);
        let max_distance = max(reach * 3u, 1u);
        let distance_to_region =
            abs_diff_u32(position_x_q, region.center_x_q)
            + abs_diff_u32(position_y_q, region.center_y_q)
            + abs_diff_u32(position_z_q, region.center_z_q);
        if (distance_to_region >= max_distance) {
            continue;
        }

        let contribution_weight = trace_region_contribution_weight(
            distance_to_region,
            max_distance,
            ray_budget,
            region.coverage_q,
        );
        let base_rgb = trace_region_base_rgb(region);
        weighted_rgb = vec3<u32>(
            weighted_rgb.x + base_rgb.x * contribution_weight,
            weighted_rgb.y + base_rgb.y * contribution_weight,
            weighted_rgb.z + base_rgb.z * contribution_weight,
        );
        total_weight = total_weight + contribution_weight;
    }

    if (total_weight == 0u) {
        return 0u;
    }

    return pack_rgb8(vec3<u32>(
        min(255u, (weighted_rgb.x + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.y + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.z + total_weight / 2u) / total_weight),
    ));
}

fn traced_contribution_rgb(
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    ray_budget: u32,
) -> u32 {
    return traced_contribution_rgb_for_trace_count(
        position_x_q,
        position_y_q,
        position_z_q,
        radius_q,
        ray_budget,
        active_trace_count(),
    );
}

fn gathered_resident_rgb(
    exclude_probe_id: u32,
    parent_probe_id: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    ray_budget: u32,
) -> u32 {
    if (params.resident_probe_count == 0u) {
        return 0u;
    }

    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    for (var resident_index = 0u; resident_index < params.resident_probe_count; resident_index = resident_index + 1u) {
        let resident = resident_probe_inputs[resident_index];
        if (resident.probe_id == exclude_probe_id || resident.previous_irradiance_rgb == 0u) {
            continue;
        }

        let max_distance = max(radius_q + resident.radius_q + 1u, 1u);
        let distance_to_probe =
            abs_diff_u32(position_x_q, resident.position_x_q)
            + abs_diff_u32(position_y_q, resident.position_y_q)
            + abs_diff_u32(position_z_q, resident.position_z_q);
        if (distance_to_probe >= max_distance) {
            continue;
        }

        let gather_weight = resident_probe_gather_weight(
            distance_to_probe,
            max_distance,
            ray_budget,
            resident.ray_budget,
        );
        let hierarchy_weight = hierarchy_probe_gather_boost(
            exclude_probe_id,
            parent_probe_id,
            resident_ancestor_probe_id,
            resident_ancestor_depth,
            resident_secondary_ancestor_probe_id,
            resident_secondary_ancestor_depth,
            resident_tertiary_ancestor_probe_id,
            resident_tertiary_ancestor_depth,
            resident_quaternary_ancestor_probe_id,
            resident_quaternary_ancestor_depth,
            resident.probe_id,
            resident.parent_probe_id,
        );
        let combined_weight = min(255u, gather_weight + hierarchy_weight);
        let resident_rgb = unpack_rgb8(resident.previous_irradiance_rgb);
        weighted_rgb = vec3<u32>(
            weighted_rgb.x + resident_rgb.x * combined_weight,
            weighted_rgb.y + resident_rgb.y * combined_weight,
            weighted_rgb.z + resident_rgb.z * combined_weight,
        );
        total_weight = total_weight + combined_weight;
    }

    let lineage_rgb = gathered_lineage_resident_rgb(
        resident_ancestor_probe_id,
        resident_ancestor_depth,
        resident_secondary_ancestor_probe_id,
        resident_secondary_ancestor_depth,
        resident_tertiary_ancestor_probe_id,
        resident_tertiary_ancestor_depth,
        resident_quaternary_ancestor_probe_id,
        resident_quaternary_ancestor_depth,
    );
    if (total_weight == 0u) {
        return lineage_rgb;
    }

    let spatial_rgb = pack_rgb8(vec3<u32>(
        min(255u, (weighted_rgb.x + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.y + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.z + total_weight / 2u) / total_weight),
    ));
    if (lineage_rgb == 0u) {
        return spatial_rgb;
    }
    return temporal_update_rgb(spatial_rgb, lineage_rgb, 224u);
}

fn gathered_lineage_resident_rgb(
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
) -> u32 {
    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;

    let primary_lineage_weight =
        lineage_resident_gather_weight(resident_ancestor_depth, 112u, 12u);
    if (primary_lineage_weight > 0u) {
        let primary_index = find_resident_probe_index(resident_ancestor_probe_id);
        if (primary_index != NO_PARENT_PROBE_ID) {
            let primary_rgb = unpack_rgb8(resident_probe_inputs[primary_index].previous_irradiance_rgb);
            weighted_rgb = vec3<u32>(
                weighted_rgb.x + primary_rgb.x * primary_lineage_weight,
                weighted_rgb.y + primary_rgb.y * primary_lineage_weight,
                weighted_rgb.z + primary_rgb.z * primary_lineage_weight,
            );
            total_weight = total_weight + primary_lineage_weight;
        }
    }

    let secondary_lineage_weight =
        lineage_resident_gather_weight(resident_secondary_ancestor_depth, 208u, 20u);
    if (secondary_lineage_weight > 0u) {
        let secondary_index = find_resident_probe_index(resident_secondary_ancestor_probe_id);
        if (secondary_index != NO_PARENT_PROBE_ID) {
            let secondary_rgb =
                unpack_rgb8(resident_probe_inputs[secondary_index].previous_irradiance_rgb);
            weighted_rgb = vec3<u32>(
                weighted_rgb.x + secondary_rgb.x * secondary_lineage_weight,
                weighted_rgb.y + secondary_rgb.y * secondary_lineage_weight,
                weighted_rgb.z + secondary_rgb.z * secondary_lineage_weight,
            );
            total_weight = total_weight + secondary_lineage_weight;
        }
    }

    let tertiary_lineage_weight =
        lineage_resident_gather_weight(resident_tertiary_ancestor_depth, 152u, 24u);
    if (tertiary_lineage_weight > 0u) {
        let tertiary_index = find_resident_probe_index(resident_tertiary_ancestor_probe_id);
        if (tertiary_index != NO_PARENT_PROBE_ID) {
            let tertiary_rgb =
                unpack_rgb8(resident_probe_inputs[tertiary_index].previous_irradiance_rgb);
            weighted_rgb = vec3<u32>(
                weighted_rgb.x + tertiary_rgb.x * tertiary_lineage_weight,
                weighted_rgb.y + tertiary_rgb.y * tertiary_lineage_weight,
                weighted_rgb.z + tertiary_rgb.z * tertiary_lineage_weight,
            );
            total_weight = total_weight + tertiary_lineage_weight;
        }
    }

    let quaternary_lineage_weight =
        lineage_resident_gather_weight(resident_quaternary_ancestor_depth, 160u, 24u);
    if (quaternary_lineage_weight > 0u) {
        let quaternary_index = find_resident_probe_index(resident_quaternary_ancestor_probe_id);
        if (quaternary_index != NO_PARENT_PROBE_ID) {
            let quaternary_rgb =
                unpack_rgb8(resident_probe_inputs[quaternary_index].previous_irradiance_rgb);
            weighted_rgb = vec3<u32>(
                weighted_rgb.x + quaternary_rgb.x * quaternary_lineage_weight,
                weighted_rgb.y + quaternary_rgb.y * quaternary_lineage_weight,
                weighted_rgb.z + quaternary_rgb.z * quaternary_lineage_weight,
            );
            total_weight = total_weight + quaternary_lineage_weight;
        }
    }

    if (total_weight == 0u) {
        return 0u;
    }

    return pack_rgb8(vec3<u32>(
        min(255u, (weighted_rgb.x + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.y + total_weight / 2u) / total_weight),
        min(255u, (weighted_rgb.z + total_weight / 2u) / total_weight),
    ));
}

fn lineage_resident_gather_weight(
    resident_ancestor_depth: u32,
    base_weight: u32,
    falloff_step: u32,
) -> u32 {
    if (resident_ancestor_depth == 0u) {
        return 0u;
    }

    let attenuation = min(resident_ancestor_depth - 1u, 5u) * falloff_step;
    if (attenuation >= base_weight) {
        return 64u;
    }
    return max(64u, base_weight - attenuation);
}

fn find_resident_probe_index(probe_id: u32) -> u32 {
    for (var resident_index = 0u; resident_index < params.resident_probe_count; resident_index = resident_index + 1u) {
        if (resident_probe_inputs[resident_index].probe_id == probe_id) {
            return resident_index;
        }
    }
    return NO_PARENT_PROBE_ID;
}

fn resident_ancestor_trace_inheritance_weight(resident_ancestor_depth: u32) -> u32 {
    if (resident_ancestor_depth <= 1u) {
        return 0u;
    }

    var weight = 96u;
    for (var depth = 2u; depth < resident_ancestor_depth; depth = depth + 1u) {
        weight = max(28u, (weight * 184u + 127u) / 256u);
    }
    return weight;
}

fn lineage_trace_support_weight(lineage_trace_support_q: u32, ray_budget: u32) -> u32 {
    let support_weight = min(lineage_trace_support_q, 255u);
    let budget_weight = min(ray_budget, 192u) / 2u;
    return min(192u, 24u + support_weight / 2u + budget_weight / 3u);
}

fn apply_lineage_trace_lighting_continuation(
    traced_packed: u32,
    lineage_trace_lighting_rgb: u32,
    lineage_trace_support_q: u32,
    ray_budget: u32,
) -> u32 {
    if (lineage_trace_lighting_rgb == 0u || lineage_trace_support_q == 0u) {
        return traced_packed;
    }

    let continuation_weight =
        lineage_trace_support_weight(lineage_trace_support_q, ray_budget);
    if (traced_packed == 0u) {
        return lineage_trace_lighting_rgb;
    }
    return temporal_update_rgb(
        traced_packed,
        lineage_trace_lighting_rgb,
        continuation_weight,
    );
}

fn blend_traced_contribution_with_resident_ancestor(
    traced_packed: u32,
    ray_budget: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
) -> u32 {
    if (resident_ancestor_probe_id == NO_PARENT_PROBE_ID || resident_ancestor_depth <= 1u) {
        return traced_packed;
    }

    let resident_index = find_resident_probe_index(resident_ancestor_probe_id);
    if (resident_index == NO_PARENT_PROBE_ID) {
        return traced_packed;
    }

    let ancestor_probe = resident_probe_inputs[resident_index];
    let ancestor_traced = traced_contribution_rgb_for_trace_count(
        ancestor_probe.position_x_q,
        ancestor_probe.position_y_q,
        ancestor_probe.position_z_q,
        ancestor_probe.radius_q,
        max(ray_budget, ancestor_probe.ray_budget),
        params.trace_region_count,
    );
    if (ancestor_traced == 0u) {
        return traced_packed;
    }

    let inheritance_weight = resident_ancestor_trace_inheritance_weight(resident_ancestor_depth);
    if (inheritance_weight == 0u) {
        return traced_packed;
    }
    return temporal_update_rgb(traced_packed, ancestor_traced, inheritance_weight);
}

fn traced_contribution_rgb_with_resident_ancestors(
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    ray_budget: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
    skip_scene_prepare_q: u32,
) -> u32 {
    var traced = traced_contribution_rgb(
        position_x_q,
        position_y_q,
        position_z_q,
        radius_q,
        ray_budget,
    );
    traced = blend_traced_contribution_with_resident_ancestor(
        traced,
        ray_budget,
        resident_ancestor_probe_id,
        resident_ancestor_depth,
    );
    traced = blend_traced_contribution_with_resident_ancestor(
        traced,
        ray_budget,
        resident_secondary_ancestor_probe_id,
        resident_secondary_ancestor_depth,
    );
    traced = blend_traced_contribution_with_resident_ancestor(
        traced,
        ray_budget,
        resident_tertiary_ancestor_probe_id,
        resident_tertiary_ancestor_depth,
    );
    traced = blend_traced_contribution_with_resident_ancestor(
        traced,
        ray_budget,
        resident_quaternary_ancestor_probe_id,
        resident_quaternary_ancestor_depth,
    );
    if (skip_scene_prepare_q == 0u) {
        let scene_prepare = scene_prepare_contribution_rgb(
            position_x_q,
            position_y_q,
            position_z_q,
            radius_q,
            ray_budget,
        );
        traced = blend_traced_with_scene_prepare(traced, scene_prepare, ray_budget);
    }
    return traced;
}

fn blend_channel(previous: u32, contribution: u32, weight: u32) -> u32 {
    let clamped_weight = min(weight, 255u);
    let inverse_weight = 255u - clamped_weight;
    return (previous * inverse_weight + contribution * clamped_weight + 127u) / 255u;
}

fn temporal_update_rgb(previous_packed: u32, contribution_packed: u32, weight: u32) -> u32 {
    let previous = unpack_rgb8(previous_packed);
    let contribution = unpack_rgb8(contribution_packed);
    let blended = vec3<u32>(
        blend_channel(previous.x, contribution.x, weight),
        blend_channel(previous.y, contribution.y, weight),
        blend_channel(previous.z, contribution.z, weight),
    );
    return blended.x | (blended.y << 8u) | (blended.z << 16u);
}

fn combine_traced_and_gathered_rgb(
    traced_packed: u32,
    gathered_packed: u32,
    lineage_trace_support_q: u32,
    ray_budget: u32,
) -> u32 {
    if (traced_packed == 0u || gathered_packed == 0u) {
        return traced_packed;
    }

    let gather_weight = min(
        192u,
        24u
            + min(ray_budget, 160u) / 3u
            + min(params.tracing_budget, 4u) * 16u
            + min(lineage_trace_support_q, 128u) / 2u,
    );
    return temporal_update_rgb(traced_packed, gathered_packed, gather_weight);
}

fn apply_runtime_hierarchy_irradiance_continuation(
    gathered_packed: u32,
    runtime_hierarchy_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_weight_q: u32,
    lineage_trace_support_q: u32,
    ray_budget: u32,
) -> u32 {
    if (runtime_hierarchy_irradiance_rgb == 0u || runtime_hierarchy_irradiance_weight_q == 0u) {
        return gathered_packed;
    }

    if (gathered_packed == 0u) {
        return runtime_hierarchy_irradiance_rgb;
    }

    let continuation_weight = min(
        224u,
        24u
            + min(runtime_hierarchy_irradiance_weight_q, 192u) / 2u
            + min(ray_budget, 160u) / 3u
            + min(lineage_trace_support_q, 128u) / 3u,
    );
    return temporal_update_rgb(
        gathered_packed,
        runtime_hierarchy_irradiance_rgb,
        continuation_weight,
    );
}

fn combine_traced_and_gathered_with_runtime_hierarchy_fallback(
    traced_packed: u32,
    gathered_packed: u32,
    runtime_hierarchy_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_weight_q: u32,
    lineage_trace_support_q: u32,
    ray_budget: u32,
) -> u32 {
    let continued_gathered = apply_runtime_hierarchy_irradiance_continuation(
        gathered_packed,
        runtime_hierarchy_irradiance_rgb,
        runtime_hierarchy_irradiance_weight_q,
        lineage_trace_support_q,
        ray_budget,
    );
    if (traced_packed == 0u) {
        return select(
            0u,
            continued_gathered,
            runtime_hierarchy_irradiance_rgb != 0u
                && runtime_hierarchy_irradiance_weight_q != 0u,
        );
    }

    return combine_traced_and_gathered_rgb(
        traced_packed,
        continued_gathered,
        lineage_trace_support_q,
        ray_budget,
    );
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let completed_probe_count = min(params.pending_probe_count, available_probe_completion_budget());
    let completed_trace_count = min(params.trace_region_count, params.tracing_budget);
    let irradiance_count = params.resident_probe_count + completed_probe_count;

    if (index == 0u) {
        completed_probe_updates[0] = completed_probe_count;
        completed_trace_regions[0] = completed_trace_count;
        probe_irradiance_updates[0] = irradiance_count;
        probe_trace_lighting_updates[0] = irradiance_count;
    }

    if (index < params.resident_probe_count) {
        let probe = resident_probe_inputs[index];
        let entry_offset = 1u + index * 2u;
        let traced = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_trace_q,
        );
        let continued_traced = apply_lineage_trace_lighting_continuation(
            traced,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let traced_for_irradiance = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_irradiance_q,
        );
        let continued_traced_for_irradiance = apply_lineage_trace_lighting_continuation(
            traced_for_irradiance,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let gathered = gathered_resident_rgb(
            probe.probe_id,
            probe.parent_probe_id,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
        );
        let contribution = combine_traced_and_gathered_with_runtime_hierarchy_fallback(
            continued_traced_for_irradiance,
            gathered,
            probe.runtime_hierarchy_irradiance_rgb,
            probe.runtime_hierarchy_irradiance_weight_q,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        probe_irradiance_updates[entry_offset] = probe.probe_id;
        probe_irradiance_updates[entry_offset + 1u] = select(
            temporal_update_rgb(
                probe.previous_irradiance_rgb,
                contribution,
                temporal_update_weight(probe.ray_budget, params.tracing_budget),
            ),
            probe.previous_irradiance_rgb,
            contribution == 0u,
        );
        probe_trace_lighting_updates[entry_offset] = probe.probe_id;
        probe_trace_lighting_updates[entry_offset + 1u] = continued_traced;
    }

    if (index < completed_probe_count) {
        let probe = pending_probe_updates[index];
        completed_probe_updates[index + 1u] = probe.probe_id;
        let entry_index = params.resident_probe_count + index;
        let entry_offset = 1u + entry_index * 2u;
        let traced = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_trace_q,
        );
        let continued_traced = apply_lineage_trace_lighting_continuation(
            traced,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let traced_for_irradiance = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_irradiance_q,
        );
        let continued_traced_for_irradiance = apply_lineage_trace_lighting_continuation(
            traced_for_irradiance,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let gathered = gathered_resident_rgb(
            probe.probe_id,
            probe.parent_probe_id,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
        );
        probe_irradiance_updates[entry_offset] = probe.probe_id;
        probe_irradiance_updates[entry_offset + 1u] =
            combine_traced_and_gathered_with_runtime_hierarchy_fallback(
                continued_traced_for_irradiance,
                gathered,
                probe.runtime_hierarchy_irradiance_rgb,
                probe.runtime_hierarchy_irradiance_weight_q,
                probe.lineage_trace_support_q,
                probe.ray_budget,
            );
        probe_trace_lighting_updates[entry_offset] = probe.probe_id;
        probe_trace_lighting_updates[entry_offset + 1u] = continued_traced;
    }

    if (index < completed_trace_count) {
        completed_trace_regions[index + 1u] = scheduled_trace_regions[index].region_id;
    }
}
