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

fn scene_prepare_owned_card_capture_radiance(owner_id: u32) -> vec4<u32> {
    let descriptor_count = scene_prepare_descriptor_count();
    for (var descriptor_index = 0u; descriptor_index < descriptor_count; descriptor_index = descriptor_index + 1u) {
        let descriptor = scene_prepare_descriptors[descriptor_index];
        if (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE
            && descriptor.primary_id == owner_id
            && descriptor._padding1 != 0u) {
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

fn scene_prepare_descriptor_has_authoritative_radiance(
    descriptor: ScenePrepareDescriptor,
) -> bool {
    return descriptor._padding1 != 0u
        && (descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE
            || descriptor.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL);
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
            let owner_card_radiance = scene_prepare_owned_card_capture_radiance(descriptor._padding0);
            if (owner_card_radiance.w != 0u) {
                return owner_card_radiance.xyz;
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
    if (scene_prepare_descriptor_has_authoritative_radiance(descriptor)) {
        return base_rgb;
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
    if (region.rt_lighting_rgb != 0u) {
        return unpack_rgb8(region.rt_lighting_rgb);
    }
    let base_rgb = vec3<u32>(
        32u + ((region.region_id * 17u + region.center_x_q + region.coverage_q) % 160u),
        32u + ((region.region_id * 11u + region.center_y_q + region.radius_q) % 160u),
        32u + ((region.region_id * 7u + region.center_z_q + region.coverage_q * 3u) % 160u),
    );
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
