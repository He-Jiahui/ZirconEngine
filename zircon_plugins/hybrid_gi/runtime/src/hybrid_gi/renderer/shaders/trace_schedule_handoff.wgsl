@group(0) @binding(0)
var<storage, read> hybrid_gi_scene_words: array<u32>;

@group(0) @binding(1)
var<storage, read_write> hybrid_gi_trace_words: array<u32>;

const HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC: u32 = 0x48474944u;
const HYBRID_GI_TRACE_SCHEDULE_MAGIC: u32 = 0x48474954u;
const DEPTH_Q24_MAX: u32 = 16777215u;
const TRACE_DEPTH_SOURCE_VALID_FLAG: u32 = 1u;

fn depth_unorm8_from_q24(depth_q24: u32) -> u32 {
    if (depth_q24 >= DEPTH_Q24_MAX) {
        return 255u;
    }
    return min(254u, (depth_q24 * 255u + (DEPTH_Q24_MAX / 2u)) / DEPTH_Q24_MAX);
}

fn pack_depth_rgba(depth_q24: u32, valid: bool) -> u32 {
    if (!valid) {
        return 0u;
    }
    let depth_u8 = depth_unorm8_from_q24(depth_q24);
    return depth_u8 | (depth_u8 << 8u) | (depth_u8 << 16u) | (255u << 24u);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x != 0u || global_id.y != 0u || global_id.z != 0u) {
        return;
    }

    let handoff_magic = hybrid_gi_scene_words[0];
    let width = hybrid_gi_scene_words[1];
    let height = hybrid_gi_scene_words[2];
    let depth_q24 = hybrid_gi_scene_words[3];
    let sample_count = hybrid_gi_scene_words[4];
    let valid_depth_source =
        handoff_magic == HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC &&
        width > 0u &&
        height > 0u &&
        depth_q24 < DEPTH_Q24_MAX;

    hybrid_gi_trace_words[0] = HYBRID_GI_TRACE_SCHEDULE_MAGIC;
    hybrid_gi_trace_words[1] = select(0u, 1u, valid_depth_source);
    hybrid_gi_trace_words[2] = width;
    hybrid_gi_trace_words[3] = height;
    hybrid_gi_trace_words[4] = depth_q24;
    hybrid_gi_trace_words[5] = sample_count;
    hybrid_gi_trace_words[6] = pack_depth_rgba(depth_q24, valid_depth_source);
    hybrid_gi_trace_words[7] = select(0u, TRACE_DEPTH_SOURCE_VALID_FLAG, valid_depth_source);
    hybrid_gi_trace_words[8] = 0u;
    hybrid_gi_trace_words[9] = handoff_magic;
}
