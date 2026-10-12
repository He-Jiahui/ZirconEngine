struct ClusterVertex {
    clip_position: vec4<f32>,
    cluster_id: u32,
    padding: array<u32, 3>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) cluster_id: u32,
}

@group(0) @binding(0) var<storage, read> visible_clusters: array<u32>;
@group(0) @binding(1) var<storage, read> cluster_vertices: array<ClusterVertex>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let vertex = cluster_vertices[vertex_index];
    var output: VertexOutput;
    output.position = vertex.clip_position;
    output.cluster_id = vertex.cluster_id;
    return output;
}

fn cluster_is_visible(cluster_id: u32) -> bool {
    let count = min(visible_clusters[0], arrayLength(&visible_clusters) - 1u);
    for (var index = 0u; index < count; index += 1u) {
        if visible_clusters[index + 1u] == cluster_id {
            return true;
        }
    }
    return false;
}

@fragment
fn fs_depth(input: VertexOutput) {
    if (!cluster_is_visible(input.cluster_id)) {
        discard;
    }
}

@fragment
fn fs_color(input: VertexOutput) -> @location(0) vec4<f32> {
    if (!cluster_is_visible(input.cluster_id)) {
        discard;
    }
    let tint = f32(input.cluster_id % 7u) / 6.0;
    return vec4<f32>(0.15 + tint * 0.75, 0.85 - tint * 0.45, 0.2, 0.35);
}
