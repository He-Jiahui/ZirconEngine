// Resident-page candidate filtering. CPU policy supplies candidates; this pass does not traverse a hierarchy.
@group(0) @binding(0) var<storage, read> resident_pages: array<u32>;

struct ClusterCandidate {
    cluster_id: u32,
    page_id: u32,
    candidate_index: u32,
    instance_index: u32,
    lod_level: u32,
}

struct IdStream {
    count: u32,
    ids: array<u32>,
}

@group(0) @binding(1) var<storage, read> cluster_candidates: array<ClusterCandidate>;
@group(0) @binding(2) var<storage, read_write> visible_clusters: IdStream;
@group(0) @binding(3) var<storage, read_write> visible_candidates: IdStream;
@group(0) @binding(4) var<storage, read_write> requested_pages: IdStream;
@group(0) @binding(5) var<storage, read_write> visbuffer64: IdStream;
@group(0) @binding(6) var<storage, read> cull_input: array<u32>;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // One ordered writer keeps duplicate requests and candidate identities deterministic.
    if global_id.x != 0u { return; }
    let candidate_count = min(min(cluster_candidates[0].cluster_id, arrayLength(&cluster_candidates) - 1u), cull_input[3]);
    let page_count = min(resident_pages[0], arrayLength(&resident_pages) - 1u);
    let request_budget = cull_input[1] - min(cull_input[1], cull_input[8]);
    for (var index = 0u; index < candidate_count; index += 1u) {
        let candidate = cluster_candidates[index + 1u];
        var resident = false;
        for (var page = 0u; page < page_count; page += 1u) {
            if resident_pages[page + 1u] == candidate.page_id { resident = true; break; }
        }
        if resident {
            let output_index = visible_clusters.count;
            if output_index < arrayLength(&visible_clusters.ids) && output_index < arrayLength(&visible_candidates.ids) && output_index * 2u + 1u < arrayLength(&visbuffer64.ids) {
                visible_clusters.ids[output_index] = candidate.cluster_id;
                visible_candidates.ids[output_index] = candidate.candidate_index;
                // Same neutral 20/20/16/6/2 encoding, produced on the GPU as two u32 words.
                visbuffer64.ids[output_index * 2u] = (candidate.cluster_id & 0xfffffu) | ((candidate.page_id & 0xfffu) << 20u);
                visbuffer64.ids[output_index * 2u + 1u] = ((candidate.page_id >> 12u) & 0xffu) | ((candidate.instance_index & 0xffffu) << 8u) | ((candidate.lod_level & 0x3fu) << 24u);
                visible_clusters.count += 1u;
                visible_candidates.count += 1u;
                visbuffer64.count += 1u;
            }
        } else {
            var duplicate = false;
            for (var request = 0u; request < requested_pages.count; request += 1u) {
                if requested_pages.ids[request] == candidate.page_id { duplicate = true; break; }
            }
            if !duplicate && requested_pages.count < min(request_budget, arrayLength(&requested_pages.ids)) {
                requested_pages.ids[requested_pages.count] = candidate.page_id;
                requested_pages.count += 1u;
            }
        }
    }
}
