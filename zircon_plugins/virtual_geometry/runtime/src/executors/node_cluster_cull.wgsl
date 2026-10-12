@group(0) @binding(0) var<storage, read> page_requests: array<u32>;

struct ClusterCandidate {
    cluster_id: u32,
    page_id: u32,
}

struct VisibleClusters {
    count: atomic<u32>,
    cluster_ids: array<u32>,
}

@group(0) @binding(1) var<storage, read> cluster_candidates: array<ClusterCandidate>;
@group(0) @binding(2) var<storage, read_write> visible_clusters: VisibleClusters;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let candidate_count = min(cluster_candidates[0].cluster_id, arrayLength(&cluster_candidates) - 1u);
    if global_id.x >= candidate_count {
        return;
    }
    let candidate = cluster_candidates[global_id.x + 1u];
    let page_count = min(page_requests[0], arrayLength(&page_requests) - 1u);
    var resident = false;
    for (var index = 0u; index < page_count; index += 1u) {
        if page_requests[index + 1u] == candidate.page_id {
            resident = true;
            break;
        }
    }
    if resident {
        let output_index = atomicAdd(&visible_clusters.count, 1u);
        if output_index < arrayLength(&visible_clusters.cluster_ids) {
            visible_clusters.cluster_ids[output_index] = candidate.cluster_id;
        }
    }
}
