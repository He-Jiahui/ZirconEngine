struct VirtualGeometryUploaderParams {
    resident_count: u32,
    pending_count: u32,
    page_budget: u32,
    evictable_count: u32,
};

@group(0) @binding(0)
var<uniform> params: VirtualGeometryUploaderParams;

@group(0) @binding(1)
var<storage, read> pending_requests: array<u32>;

@group(0) @binding(2)
var<storage, read_write> completed_pages: array<u32>;

fn available_completion_budget() -> u32 {
    let free_budget = max(params.page_budget, params.resident_count) - params.resident_count;
    return free_budget + params.evictable_count;
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let completion_budget = min(params.pending_count, available_completion_budget());

    if (index == 0u) {
        completed_pages[0] = completion_budget;
    }

    if (index >= params.pending_count || index >= completion_budget) {
        return;
    }

    completed_pages[index + 1u] = pending_requests[index];
}
