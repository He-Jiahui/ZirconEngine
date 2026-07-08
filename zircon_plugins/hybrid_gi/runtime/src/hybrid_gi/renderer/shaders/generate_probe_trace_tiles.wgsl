struct ProbeTraceTileGenerationParams {
    record_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<uniform> params: ProbeTraceTileGenerationParams;

@group(0) @binding(1)
var<storage, read> seed_tiles: array<u32>;

@group(0) @binding(2)
var<storage, read_write> output_tiles: array<u32>;

@group(0) @binding(3)
var<storage, read_write> indirect_args: array<u32>;

const WORDS_PER_RECORD: u32 = 4u;
const TRACE_THREADS_PER_GROUP: u32 = 128u;

@compute @workgroup_size(8, 8, 1)
fn cs_main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let local_index = local_id.y * 8u + local_id.x;
    let record_index = global_id.z;

    if (local_index == 0u && record_index == 0u) {
        indirect_args[0] = params.record_count;
        indirect_args[1] = TRACE_THREADS_PER_GROUP;
        indirect_args[2] = (params.record_count + TRACE_THREADS_PER_GROUP - 1u) / TRACE_THREADS_PER_GROUP;
        indirect_args[3] = 1u;
    }

    if (record_index >= params.record_count || local_index != 0u) {
        return;
    }

    let base = record_index * WORDS_PER_RECORD;
    output_tiles[base] = record_index;
    output_tiles[base + 1u] = seed_tiles[base + 1u];
    output_tiles[base + 2u] = seed_tiles[base + 2u];
    output_tiles[base + 3u] = max(seed_tiles[base + 3u], 1u);
}
