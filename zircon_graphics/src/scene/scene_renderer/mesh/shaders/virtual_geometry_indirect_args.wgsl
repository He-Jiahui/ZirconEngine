struct VirtualGeometryIndirectSegmentInput {
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    page_id: u32,
    resident_slot: u32,
    state: u32,
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
    resident_slot: u32,
    state: u32,
) -> u32 {
    if (state != 0u || segment_triangle_count <= 1u || resident_slot < 4u) {
        return 0u;
    }

    let slot_band = 1u + ((min(resident_slot, 7u) - 4u) / 3u);
    return min(segment_triangle_count - 1u, slot_band);
}

fn page_id_cluster_offset(
    visible_triangle_count: u32,
    page_id: u32,
    resident_slot: u32,
    state: u32,
) -> u32 {
    if (state != 0u || visible_triangle_count <= 1u || resident_slot < 4u) {
        return 0u;
    }

    return min(visible_triangle_count - 1u, page_id & 1u);
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
            segment.resident_slot,
            segment.state,
        );
        let page_offset = page_id_cluster_offset(
            segment_triangle_count - resident_trim,
            segment.page_id,
            segment.resident_slot,
            segment.state,
        );
        first_index = (start_triangle + resident_trim + page_offset) * 3u;
        index_count =
            visible_triangle_count_for_state(
                segment_triangle_count - resident_trim - page_offset,
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
