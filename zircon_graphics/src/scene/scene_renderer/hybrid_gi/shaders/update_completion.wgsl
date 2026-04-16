struct HybridGiCompletionParams {
    resident_probe_count: u32,
    pending_probe_count: u32,
    probe_budget: u32,
    trace_region_count: u32,
    tracing_budget: u32,
    evictable_probe_count: u32,
    _padding0: u32,
    _padding1: u32,
};

@group(0) @binding(0)
var<uniform> params: HybridGiCompletionParams;

@group(0) @binding(1)
var<storage, read> pending_probe_updates: array<u32>;

@group(0) @binding(2)
var<storage, read> scheduled_trace_regions: array<u32>;

@group(0) @binding(3)
var<storage, read_write> completed_probe_updates: array<u32>;

@group(0) @binding(4)
var<storage, read_write> completed_trace_regions: array<u32>;

fn available_probe_completion_budget() -> u32 {
    let free_budget = max(params.probe_budget, params.resident_probe_count) - params.resident_probe_count;
    return free_budget + params.evictable_probe_count;
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let completed_probe_count = min(params.pending_probe_count, available_probe_completion_budget());
    let completed_trace_count = min(params.trace_region_count, params.tracing_budget);

    if (index == 0u) {
        completed_probe_updates[0] = completed_probe_count;
        completed_trace_regions[0] = completed_trace_count;
    }

    if (index < completed_probe_count) {
        completed_probe_updates[index + 1u] = pending_probe_updates[index];
    }

    if (index < completed_trace_count) {
        completed_trace_regions[index + 1u] = scheduled_trace_regions[index];
    }
}
