@group(0) @binding(0)
var<storage, read> hybrid_gi_trace_words: array<u32>;

const HYBRID_GI_TRACE_SCHEDULE_MAGIC: u32 = 0x48474954u;
const TRACE_DEPTH_SOURCE_VALID_FLAG: u32 = 1u;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xffu) / 255.0;
    let g = f32((packed >> 8u) & 0xffu) / 255.0;
    let b = f32((packed >> 16u) & 0xffu) / 255.0;
    let a = f32((packed >> 24u) & 0xffu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    let trace_magic = hybrid_gi_trace_words[0];
    let packet_count = hybrid_gi_trace_words[1];
    let packed_depth_rgba = hybrid_gi_trace_words[6];
    let valid_flag = hybrid_gi_trace_words[7];
    let valid_depth_source =
        trace_magic == HYBRID_GI_TRACE_SCHEDULE_MAGIC &&
        packet_count > 0u &&
        valid_flag == TRACE_DEPTH_SOURCE_VALID_FLAG;

    if (!valid_depth_source) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    return unpack_rgba8(packed_depth_rgba);
}
