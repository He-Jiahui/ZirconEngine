struct ClusterVertex {
    clip_position: vec4<f32>,
    cluster_id: u32,
    candidate_index: u32,
    authority_index: u32,
    instance_index: u32,
    lineage_depth: u32,
    frontier_rank: u32,
    padding: array<u32, 2>,
}

struct AuthorityRecords {
    words: array<u32>,
}

@group(0) @binding(0) var<storage, read> visible_candidate_indices: array<u32>;
@group(0) @binding(1) var<storage, read> cluster_vertices: array<ClusterVertex>;
@group(0) @binding(2) var<storage, read> authority_records: AuthorityRecords;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) cluster_id: u32,
    @location(1) @interpolate(flat) candidate_index: u32,
    @location(2) @interpolate(flat) authority_index: u32,
    @location(3) @interpolate(flat) authority_valid: u32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let vertex = cluster_vertices[vertex_index];
    let authority_offset = vertex.authority_index * 14u;
    let authority_valid = authority_offset + 13u < arrayLength(&authority_records.words)
        && authority_records.words[authority_offset] == vertex.instance_index
        && instance_index == ((authority_records.words[authority_offset + 8u] & 0xffffu) << 16u
            | (authority_records.words[authority_offset + 9u] & 0xffffu))
        && authority_records.words[authority_offset + 4u] == vertex.cluster_id
        && authority_records.words[authority_offset + 11u] == vertex.candidate_index
        && authority_records.words[authority_offset + 12u] == vertex.lineage_depth
        && authority_records.words[authority_offset + 13u] == vertex.frontier_rank;
    var output: VertexOutput;
    output.position = vertex.clip_position;
    output.cluster_id = vertex.cluster_id;
    output.candidate_index = vertex.candidate_index;
    output.authority_index = vertex.authority_index;
    output.authority_valid = select(0u, 1u, authority_valid);
    return output;
}

fn candidate_is_visible(candidate_index: u32) -> bool {
    let count = min(visible_candidate_indices[0], arrayLength(&visible_candidate_indices) - 1u);
    for (var index = 0u; index < count; index += 1u) {
        if visible_candidate_indices[index + 1u] == candidate_index {
            return true;
        }
    }
    return false;
}

@fragment
fn fs_depth(input: VertexOutput) {
    if input.authority_valid == 0u || !candidate_is_visible(input.candidate_index) {
        discard;
    }
    let authority_offset = input.authority_index * 14u;
    if authority_records.words[authority_offset + 4u] != input.cluster_id {
        discard;
    }
}
