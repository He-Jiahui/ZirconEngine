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

struct ResidentProbeInput {
    probe_id: u32,
    slot: u32,
    ray_budget: u32,
    _padding: u32,
};

struct PendingProbeInput {
    probe_id: u32,
    logical_index: u32,
    ray_budget: u32,
    _padding: u32,
};

@group(0) @binding(0)
var<uniform> params: HybridGiCompletionParams;

@group(0) @binding(1)
var<storage, read> resident_probe_inputs: array<ResidentProbeInput>;

@group(0) @binding(2)
var<storage, read> pending_probe_updates: array<PendingProbeInput>;

@group(0) @binding(3)
var<storage, read> scheduled_trace_regions: array<u32>;

@group(0) @binding(4)
var<storage, read_write> completed_probe_updates: array<u32>;

@group(0) @binding(5)
var<storage, read_write> completed_trace_regions: array<u32>;

@group(0) @binding(6)
var<storage, read_write> probe_irradiance_updates: array<u32>;

fn available_probe_completion_budget() -> u32 {
    let free_budget = max(params.probe_budget, params.resident_probe_count) - params.resident_probe_count;
    return free_budget + params.evictable_probe_count;
}

fn rotate_left_u32(value: u32, amount: u32) -> u32 {
    let shift = amount & 31u;
    if (shift == 0u) {
        return value;
    }
    return (value << shift) | (value >> (32u - shift));
}

fn irradiance_channel(seed: u32, bias: u32) -> u32 {
    return 48u + ((seed + bias) % 160u);
}

fn packed_irradiance_rgb(
    probe_id: u32,
    slot_or_index: u32,
    ray_budget: u32,
    pending_completion: bool,
) -> u32 {
    let pending_bias = select(13u, 97u, pending_completion);
    let seed =
        probe_id * 17u
        + slot_or_index * 31u
        + ray_budget * 7u
        + params.trace_region_count * 19u
        + params.tracing_budget * 23u
        + pending_bias;
    let r = irradiance_channel(seed, 0x1fu);
    let g = irradiance_channel(rotate_left_u32(seed, 7u), 0x3du);
    let b = irradiance_channel(rotate_left_u32(seed, 13u), 0x59u);
    return r | (g << 8u) | (b << 16u);
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let completed_probe_count = min(params.pending_probe_count, available_probe_completion_budget());
    let completed_trace_count = min(params.trace_region_count, params.tracing_budget);
    let irradiance_count = params.resident_probe_count + completed_probe_count;

    if (index == 0u) {
        completed_probe_updates[0] = completed_probe_count;
        completed_trace_regions[0] = completed_trace_count;
        probe_irradiance_updates[0] = irradiance_count;
    }

    if (index < params.resident_probe_count) {
        let probe = resident_probe_inputs[index];
        let entry_offset = 1u + index * 2u;
        probe_irradiance_updates[entry_offset] = probe.probe_id;
        probe_irradiance_updates[entry_offset + 1u] =
            packed_irradiance_rgb(probe.probe_id, probe.slot, probe.ray_budget, false);
    }

    if (index < completed_probe_count) {
        let probe = pending_probe_updates[index];
        completed_probe_updates[index + 1u] = probe.probe_id;
        let entry_index = params.resident_probe_count + index;
        let entry_offset = 1u + entry_index * 2u;
        probe_irradiance_updates[entry_offset] = probe.probe_id;
        probe_irradiance_updates[entry_offset + 1u] =
            packed_irradiance_rgb(probe.probe_id, probe.logical_index, probe.ray_budget, true);
    }

    if (index < completed_trace_count) {
        completed_trace_regions[index + 1u] = scheduled_trace_regions[index];
    }
}
