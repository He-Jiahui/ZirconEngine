const TRACE_LIGHTING_MASK_NEUTRAL_AMBIENT: u32 = 1u;
const TRACE_LIGHTING_MASK_SURFACE_CACHE: u32 = 2u;
const TRACE_LIGHTING_MASK_PROBE_LINEAGE: u32 = 4u;
const TRACE_LIGHTING_MASK_VOXEL_RADIANCE: u32 = 8u;
const TRACE_MAX_TILES_PER_PROBE: u32 = 16u;

fn dominant_intersection_source(
    surface_weight: u32,
    global_sdf_weight: u32,
    voxel_weight: u32,
) -> u32 {
    var source = 0u;
    var best_weight = 0u;
    if (surface_weight > best_weight) {
        source = TRACE_SOURCE_SURFACE_CACHE;
        best_weight = surface_weight;
    }
    if (global_sdf_weight > best_weight) {
        source = TRACE_SOURCE_GLOBAL_SDF;
        best_weight = global_sdf_weight;
    }
    if (voxel_weight > best_weight) {
        source = TRACE_SOURCE_VOXEL_CLIPMAP;
    }
    return source;
}

fn dominant_lighting_source(
    neutral_weight: u32,
    surface_weight: u32,
    lineage_weight: u32,
    voxel_weight: u32,
) -> u32 {
    var source = 0u;
    var best_weight = neutral_weight;
    if (surface_weight > best_weight) {
        source = TRACE_LIGHTING_SURFACE_CACHE;
        best_weight = surface_weight;
    }
    if (lineage_weight > best_weight) {
        source = TRACE_LIGHTING_PROBE_LINEAGE;
        best_weight = lineage_weight;
    }
    if (voxel_weight > best_weight) {
        source = TRACE_LIGHTING_VOXEL_RADIANCE;
    }
    return source;
}

fn tile_trace(
    probe_id: u32,
    ray_budget: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    lineage_trace_lighting_rgb: u32,
) -> ProbeTraceResult {
    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    var surface_weight = 0u;
    var global_sdf_weight = 0u;
    var voxel_weight = 0u;
    var surface_distance_sum = 0.0;
    var global_sdf_distance_sum = 0.0;
    var voxel_distance_sum = 0.0;
    var surface_confidence_sum = 0.0;
    var global_sdf_confidence_sum = 0.0;
    var voxel_confidence_sum = 0.0;
    var neutral_lighting_weight = 0u;
    var surface_lighting_weight = 0u;
    var lineage_lighting_weight = 0u;
    var voxel_lighting_weight = 0u;
    var intersection_backend_mask = 0u;
    var lighting_source_mask = 0u;
    var used_fallback = false;
    var texture_samples = 0u;
    var page_tests = 0u;
    var sdf_steps = 0u;
    var voxel_candidates = 0u;
    let position_hash = position_x_q ^ (position_y_q << 3u) ^ (position_z_q << 6u);
    let probe_position = vec3<f32>(
        dequantize_probe_position(position_x_q),
        dequantize_probe_position(position_y_q),
        dequantize_probe_position(position_z_q),
    );
    let trace_tile_count = min(params.tile_count, TRACE_MAX_TILES_PER_PROBE);
    let trace_tile_start = (probe_id ^ position_hash) % params.tile_count;
    let trace_tile_stride = max(1u, params.tile_count / trace_tile_count);

    for (var local_tile_index = 0u; local_tile_index < trace_tile_count; local_tile_index += 1u) {
        let tile_index = (trace_tile_start + local_tile_index * trace_tile_stride) % params.tile_count;
        let base = tile_index * WORDS_PER_TILE;
        let tile_id = probe_trace_tiles[base];
        let tile_probe_id = probe_trace_tiles[base + 1u];
        let tile_sample_id = probe_trace_tiles[base + 2u];
        let ray_count = max(probe_trace_tiles[base + 3u], 1u);
        let weight = min(255u, 24u + min(ray_budget, 192u) / 2u + min(ray_count, 128u));
        var tile_sample = surface_cache_tile_sample(tile_sample_id, ray_count);
        texture_samples = texture_samples + tile_sample.texture_samples;
        if (tile_sample.valid == 0u) {
            used_fallback = true;
            tile_sample = global_sdf_tile_sample(
                probe_position,
                tile_sample_id,
                ray_count,
                lineage_trace_lighting_rgb,
            );
            page_tests = page_tests + tile_sample.page_tests;
            sdf_steps = sdf_steps + tile_sample.sdf_steps;
        }
        if (tile_sample.valid == 0u) {
            tile_sample = voxel_fallback_tile_sample(
                probe_position,
                tile_probe_id,
                tile_sample_id,
            );
            voxel_candidates = voxel_candidates + tile_sample.voxel_candidates;
        }

        var tile_rgb = tile_sample.rgb;
        if (tile_sample.valid == 0u) {
            tile_rgb = fallback_tile_rgb(
                probe_id,
                position_hash,
                tile_id,
                tile_probe_id,
                tile_sample_id,
                ray_count,
            );
            neutral_lighting_weight = neutral_lighting_weight + weight * 3u;
            lighting_source_mask = lighting_source_mask | TRACE_LIGHTING_MASK_NEUTRAL_AMBIENT;
        } else {
            if (tile_sample.intersection_source == TRACE_SOURCE_SURFACE_CACHE) {
                surface_weight = surface_weight + weight;
                surface_distance_sum = surface_distance_sum + tile_sample.distance * f32(weight);
                surface_confidence_sum = surface_confidence_sum + tile_sample.confidence * f32(weight);
                intersection_backend_mask = intersection_backend_mask | TRACE_BACKEND_SURFACE_CACHE;
            } else if (tile_sample.intersection_source == TRACE_SOURCE_GLOBAL_SDF) {
                global_sdf_weight = global_sdf_weight + weight;
                global_sdf_distance_sum = global_sdf_distance_sum + tile_sample.distance * f32(weight);
                global_sdf_confidence_sum = global_sdf_confidence_sum + tile_sample.confidence * f32(weight);
                intersection_backend_mask = intersection_backend_mask | TRACE_BACKEND_GLOBAL_SDF;
            } else if (tile_sample.intersection_source == TRACE_SOURCE_VOXEL_CLIPMAP) {
                voxel_weight = voxel_weight + weight;
                voxel_distance_sum = voxel_distance_sum + tile_sample.distance * f32(weight);
                voxel_confidence_sum = voxel_confidence_sum + tile_sample.confidence * f32(weight);
                intersection_backend_mask = intersection_backend_mask | TRACE_BACKEND_VOXEL_CLIPMAP;
            }

            if (tile_sample.lighting_source == TRACE_LIGHTING_SURFACE_CACHE) {
                surface_lighting_weight = surface_lighting_weight + weight * 3u;
                lighting_source_mask = lighting_source_mask | TRACE_LIGHTING_MASK_SURFACE_CACHE;
            } else if (tile_sample.lighting_source == TRACE_LIGHTING_PROBE_LINEAGE) {
                lineage_lighting_weight = lineage_lighting_weight + weight * 3u;
                lighting_source_mask = lighting_source_mask | TRACE_LIGHTING_MASK_PROBE_LINEAGE;
            } else if (tile_sample.lighting_source == TRACE_LIGHTING_VOXEL_RADIANCE) {
                voxel_lighting_weight = voxel_lighting_weight + weight * 3u;
                lighting_source_mask = lighting_source_mask | TRACE_LIGHTING_MASK_VOXEL_RADIANCE;
            } else {
                neutral_lighting_weight = neutral_lighting_weight + weight * 3u;
                lighting_source_mask = lighting_source_mask | TRACE_LIGHTING_MASK_NEUTRAL_AMBIENT;
            }
        }

        weighted_rgb = vec3<u32>(
            weighted_rgb.x + tile_rgb.x * weight,
            weighted_rgb.y + tile_rgb.y * weight,
            weighted_rgb.z + tile_rgb.z * weight,
        );
        total_weight = total_weight + weight;
    }

    var traced = lineage_trace_lighting_rgb;
    if (total_weight != 0u) {
        traced = pack_rgb8(vec3<u32>(
            (weighted_rgb.x + total_weight / 2u) / total_weight,
            (weighted_rgb.y + total_weight / 2u) / total_weight,
            (weighted_rgb.z + total_weight / 2u) / total_weight,
        ));
    }
    if (total_weight != 0u && lineage_trace_lighting_rgb != 0u) {
        let lineage = unpack_rgb8(lineage_trace_lighting_rgb);
        let traced_rgb = unpack_rgb8(traced);
        traced = pack_rgb8(vec3<u32>(
            (traced_rgb.x * 3u + lineage.x + 2u) / 4u,
            (traced_rgb.y * 3u + lineage.y + 2u) / 4u,
            (traced_rgb.z * 3u + lineage.z + 2u) / 4u,
        ));
        lineage_lighting_weight = lineage_lighting_weight + total_weight;
        lighting_source_mask = lighting_source_mask | TRACE_LIGHTING_MASK_PROBE_LINEAGE;
    }

    let intersection_source = dominant_intersection_source(
        surface_weight,
        global_sdf_weight,
        voxel_weight,
    );
    let lighting_source = dominant_lighting_source(
        neutral_lighting_weight,
        surface_lighting_weight,
        lineage_lighting_weight,
        voxel_lighting_weight,
    );
    var distance = bitcast<f32>(0x7f800000u);
    var confidence = 0.0;
    if (intersection_source == TRACE_SOURCE_SURFACE_CACHE) {
        distance = surface_distance_sum / f32(max(surface_weight, 1u));
        confidence = surface_confidence_sum / f32(max(surface_weight, 1u));
    } else if (intersection_source == TRACE_SOURCE_GLOBAL_SDF) {
        distance = global_sdf_distance_sum / f32(max(global_sdf_weight, 1u));
        confidence = global_sdf_confidence_sum / f32(max(global_sdf_weight, 1u));
    } else if (intersection_source == TRACE_SOURCE_VOXEL_CLIPMAP) {
        distance = voxel_distance_sum / f32(max(voxel_weight, 1u));
        confidence = voxel_confidence_sum / f32(max(voxel_weight, 1u));
    }
    var fallback_reason = 0u;
    if (used_fallback) {
        fallback_reason = select(
            params.fallback_reason,
            TRACE_FALLBACK_INTERSECTION_MISS,
            params.fallback_reason == 0u,
        );
    }

    return ProbeTraceResult(
        traced,
        intersection_source,
        lighting_source,
        intersection_backend_mask,
        lighting_source_mask,
        distance,
        confidence,
        fallback_reason,
        texture_samples,
        page_tests,
        sdf_steps,
        voxel_candidates,
    );
}
