struct VirtualGeometryIndirectSegmentInput {
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    page_id: u32,
    submission_slot: u32,
    state: u32,
    lineage_depth: u32,
    lod_level: u32,
    frontier_rank: u32,
};

struct VirtualGeometryIndirectDrawRefInput {
    mesh_index_count: u32,
    segment_index: u32,
};

struct IndexedIndirectArgs {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct SegmentBuffer {
    values: array<VirtualGeometryIndirectSegmentInput>,
};

struct DrawRefBuffer {
    values: array<VirtualGeometryIndirectDrawRefInput>,
};

struct OutputBuffer {
    values: array<IndexedIndirectArgs>,
};

@group(0) @binding(0)
var<storage, read> segment_buffer: SegmentBuffer;

@group(0) @binding(1)
var<storage, read> draw_ref_buffer: DrawRefBuffer;

@group(0) @binding(2)
var<storage, read_write> output_buffer: OutputBuffer;

fn visible_triangle_count_for_state(segment_triangle_count: u32, state: u32) -> u32 {
    if (segment_triangle_count == 0u) {
        return 0u;
    }

    if (state == 0u) {
        return segment_triangle_count;
    }

    if (state == 1u) {
        let pending_count = u32(ceil(f32(segment_triangle_count) * 0.45));
        return clamp(pending_count, 1u, segment_triangle_count);
    }

    return 0u;
}

fn resident_slot_cluster_trim(
    segment_triangle_count: u32,
    submission_slot: u32,
    state: u32,
) -> u32 {
    if (state != 0u || segment_triangle_count <= 1u || submission_slot < 4u) {
        return 0u;
    }

    let slot_band = 1u + ((min(submission_slot, 7u) - 4u) / 3u);
    return min(segment_triangle_count - 1u, slot_band);
}

fn page_id_cluster_offset(
    visible_triangle_count: u32,
    page_id: u32,
    submission_slot: u32,
    state: u32,
) -> u32 {
    if (state != 0u || visible_triangle_count <= 1u || submission_slot < 4u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, page_id & 1u);
}

fn lod_level_cluster_trim(
    visible_triangle_count: u32,
    lod_level: u32,
    state: u32,
) -> u32 {
    if (state != 0u || visible_triangle_count <= 1u || lod_level == 0u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, min(lod_level, 3u));
}

fn lineage_depth_cluster_offset(
    visible_triangle_count: u32,
    lineage_depth: u32,
    state: u32,
) -> u32 {
    if (state != 0u || visible_triangle_count <= 1u || lineage_depth == 0u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, min(lineage_depth, 4u));
}

fn frontier_rank_cluster_trim(
    visible_triangle_count: u32,
    frontier_rank: u32,
) -> u32 {
    if (visible_triangle_count <= 1u || frontier_rank == 0u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, min(frontier_rank, 3u));
}

fn pending_submission_slot_cluster_trim(
    visible_triangle_count: u32,
    submission_slot: u32,
    state: u32,
) -> u32 {
    if (state != 1u || visible_triangle_count <= 1u || submission_slot < 3u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, min(submission_slot / 3u, 2u));
}

fn pending_submission_slot_cluster_offset(
    visible_triangle_count: u32,
    submission_slot: u32,
    state: u32,
) -> u32 {
    if (state != 1u || visible_triangle_count <= 1u || submission_slot == 0u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, submission_slot & 1u);
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&draw_ref_buffer.values)) {
        return;
    }

    let draw_ref = draw_ref_buffer.values[index];
    let segment = segment_buffer.values[draw_ref.segment_index];
    let triangle_count = draw_ref.mesh_index_count / 3u;
    var first_index = 0u;
    var index_count = draw_ref.mesh_index_count;

    if (triangle_count > 0u) {
        let segment_count = max(1u, min(segment.cluster_total_count, triangle_count));
        let segment_ordinal = segment.cluster_start_ordinal % segment_count;
        let start_triangle = (triangle_count * segment_ordinal) / segment_count;
        let end_segment_ordinal =
            min(segment_ordinal + max(segment.cluster_span_count, 1u), segment_count);
        var end_triangle = (triangle_count * end_segment_ordinal) / segment_count;
        if (end_triangle <= start_triangle) {
            end_triangle = min(start_triangle + 1u, triangle_count);
        }

        let segment_triangle_count = end_triangle - start_triangle;
        let resident_trim = resident_slot_cluster_trim(
            segment_triangle_count,
            segment.submission_slot,
            segment.state,
        );
        let page_offset = page_id_cluster_offset(
            segment_triangle_count - resident_trim,
            segment.page_id,
            segment.submission_slot,
            segment.state,
        );
        let lineage_offset = lineage_depth_cluster_offset(
            segment_triangle_count - resident_trim - page_offset,
            segment.lineage_depth,
            segment.state,
        );
        let frontier_trim = frontier_rank_cluster_trim(
            segment_triangle_count - resident_trim - page_offset - lineage_offset,
            segment.frontier_rank,
        );
        let lod_trim = lod_level_cluster_trim(
            segment_triangle_count
                - resident_trim
                - page_offset
                - lineage_offset
                - frontier_trim,
            segment.lod_level,
            segment.state,
        );
        let base_remaining_triangle_count =
            segment_triangle_count
                - resident_trim
                - page_offset
                - lineage_offset
                - frontier_trim
                - lod_trim;
        let pending_submission_trim = pending_submission_slot_cluster_trim(
            base_remaining_triangle_count,
            segment.submission_slot,
            segment.state,
        );
        let submission_remaining_triangle_count =
            base_remaining_triangle_count - pending_submission_trim;
        let pending_submission_offset = pending_submission_slot_cluster_offset(
            submission_remaining_triangle_count,
            segment.submission_slot,
            segment.state,
        );
        first_index =
            (start_triangle + resident_trim + page_offset + lineage_offset + pending_submission_offset)
                * 3u;
        index_count =
            visible_triangle_count_for_state(
                submission_remaining_triangle_count - pending_submission_offset,
                segment.state,
            ) * 3u;
    }

    output_buffer.values[index] = IndexedIndirectArgs(
        index_count,
        1u,
        first_index,
        0,
        0u,
    );
}
